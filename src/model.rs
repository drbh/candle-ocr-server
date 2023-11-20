use anyhow::Error as E;
use candle_core::{DType, Device, Result, Tensor};
use candle_examples::token_output_stream::TokenOutputStream;
use candle_nn::VarBuilder;
use candle_transformers::{generation::LogitsProcessor, models::trocr, models::trocr::TrOCRModel};
use hf_hub::api::sync::Api;
use serde_json::json;
use tokenizers::Tokenizer;

#[derive(Debug, Clone)]
pub struct ModelArgs {
    pub model: Option<String>,
    pub tokenizer: Option<String>,
    pub cpu: bool,
    pub quantized: bool,
}

pub struct ModelResources {
    pub model: TrOCRModel,
    pub tokenizer: TokenOutputStream,
    pub logit_processor: LogitsProcessor,
}

impl ModelResources {
    pub fn new(args: ModelArgs) -> anyhow::Result<Self> {
        let (model, tokenizer, logit_processor) = model(args)?;
        Ok(Self {
            model,
            tokenizer,
            logit_processor,
        })
    }
}
// Read image from bytes and resize it to 384x384 and return a tensor with shape (1, 3, 384, 384).
pub fn load_image_from_bytes(bytes: &[u8]) -> Result<Tensor> {
    let img = image::load_from_memory(bytes).map_err(candle_core::Error::wrap)?;
    let img = img.resize_exact(384, 384, image::imageops::FilterType::Triangle);

    let img = img.to_rgb8();
    let data = img.into_raw();

    let data = Tensor::from_vec(data, (384, 384, 3), &Device::Cpu)?.permute((2, 0, 1))?;
    // let data = data.unsqueeze(0)?; // Add an extra dimension to make the shape [1, 3, 384, 384]

    let mean = Tensor::new(&[0.5f32, 0.5, 0.5], &Device::Cpu)?.reshape((3, 1, 1))?;
    let std = Tensor::new(&[0.5f32, 0.5, 0.5], &Device::Cpu)?.reshape((3, 1, 1))?;

    let data = (data.to_dtype(candle_core::DType::F32)? / 255.)?
        .broadcast_sub(&mean)?
        .broadcast_div(&std)?;

    data.unsqueeze(0)
}

pub fn model(args: ModelArgs) -> anyhow::Result<(TrOCRModel, TokenOutputStream, LogitsProcessor)> {
    println!("Downloading model...");
    let tokenizer_dec = {
        let tokenizer = Api::new()?
            .model(String::from("ToluClassics/candle-trocr-tokenizer"))
            .get("tokenizer.json")?;

        Tokenizer::from_file(&tokenizer).map_err(E::msg)?
    };
    let tokenizer = TokenOutputStream::new(tokenizer_dec);
    let device = candle_examples::device(args.cpu)?;
    let vb = {
        let model = Api::new()?
            .repo(hf_hub::Repo::with_revision(
                "microsoft/trocr-base-handwritten".to_string(),
                hf_hub::RepoType::Model,
                "refs/pr/3".to_string(),
            ))
            .get("model.safetensors")?;
        println!("model: {:?}", model);
        unsafe { VarBuilder::from_mmaped_safetensors(&[model], DType::F32, &device)? }
    };
    let encoder_config =
        candle_transformers::models::vit::Config::microsoft_trocr_base_handwritten();
    let decoder_config = trocr::TrOCRConfig::default();
    let model = trocr::TrOCRModel::new(&encoder_config, &decoder_config, vb)?;
    let logits_processor = candle_transformers::generation::LogitsProcessor::new(1337, None, None);
    Ok((model, tokenizer, logits_processor))
}

pub async fn detect(
    data: &mut ModelResources,
    image: &[u8],
    sender: tokio::sync::mpsc::Sender<std::string::String>,
) -> anyhow::Result<String> {
    let device = Device::Cpu;

    println!("Loading image...");
    crate::send_and_wait(
        sender.clone(),
        json!({"status": "Loading model..."}).to_string(),
    )
    .await;
    let image = load_image_from_bytes(image)?.to_device(&device)?;

    println!("Generating caption...");
    crate::send_and_wait(
        sender.clone(),
        json!({"status": "Loading image..."}).to_string(),
    )
    .await;

    let model = &mut data.model;
    let tokenizer = &mut data.tokenizer;
    let logits_processor = &mut data.logit_processor;

    println!("Embedding image...");
    crate::send_and_wait(
        sender.clone(),
        json!({"status": "Generating caption..."}).to_string(),
    )
    .await;

    let decoder_config = trocr::TrOCRConfig::default();
    let encoder_xs = model.encoder().forward(&image)?;

    println!("Passed image through encoder...");
    crate::send_and_wait(
        sender.clone(),
        json!({"status": "Passed image through encoder..."}).to_string(),
    )
    .await;

    let mut token_ids: Vec<u32> = vec![decoder_config.decoder_start_token_id];
    for index in 0..1000 {
        let context_size = if index >= 1 { 1 } else { token_ids.len() };
        let start_pos = token_ids.len().saturating_sub(context_size);
        let input_ids = Tensor::new(&token_ids[start_pos..], &device)?.unsqueeze(0)?;

        let logits = model.decode(&input_ids, &encoder_xs, start_pos)?;

        let logits = logits.squeeze(0)?;
        let logits = logits.get(logits.dim(0)? - 1)?;
        let token = logits_processor.sample(&logits)?;
        token_ids.push(token);

        if let Some(t) = tokenizer.next_token(token)? {
            println!(
                "Sending token: {} at time {}",
                t,
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis()
            );
            sender
                .send(
                    json!({
                        "status": "token",
                        "token": t,
                    })
                    .to_string(),
                )
                .await
                .unwrap();
        }
        tokio::time::sleep(std::time::Duration::from_millis(1)).await;
        if token == decoder_config.eos_token_id {
            break;
        }
    }

    if let Some(rest) = tokenizer.decode_rest().map_err(E::msg)? {
        print!("{rest}");
    }
    println!();

    model.reset_kv_cache();
    let final_output = token_ids
        .iter()
        .filter_map(|&x| tokenizer.next_token(x).ok().unwrap())
        .collect::<Vec<_>>()
        .join("");

    Ok(final_output)
}
