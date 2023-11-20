use futures_util::StreamExt;
use poem::http::StatusCode;
use poem::{
    get, handler, listener::TcpListener, middleware::Tracing, post, web::Data, web::Path,
    EndpointExt, Route, Server,
};
use poem::{
    web::{
        sse::{Event, SSE},
        Html, Multipart,
    },
    Response,
};
use serde_json::json;
use std::sync::Arc;
use tokio::{
    sync::{mpsc, Mutex},
    time::Duration,
};
use tokio_stream::wrappers::ReceiverStream;

mod model;

const SEND_END: bool = false;

// Shared data between requests (it holds a mutex to the model)
#[derive(Clone)]
struct Shared {
    blip: Arc<Mutex<model::ModelResources>>,
}

impl Shared {
    fn new(quantized: bool) -> Self {
        Self {
            blip: Arc::new(Mutex::new(
                model::ModelResources::new(model::ModelArgs {
                    model: None,
                    tokenizer: None,
                    cpu: true,
                    quantized,
                })
                .unwrap(),
            )),
        }
    }
}

pub async fn send_and_wait(sender: mpsc::Sender<String>, msg: String) {
    sender.send(msg).await.unwrap();
    tokio::time::sleep(std::time::Duration::from_millis(1)).await;
}

#[handler]
async fn index() -> Html<String> {
    let filename = "index.html";
    let directory = "dist";
    let cwd = std::env::current_dir().unwrap();
    let cwd_as_str = cwd.to_str().unwrap();
    let full_pattern = format!("{}/app/{}/{}", cwd_as_str, directory, filename);
    println!("full pattern: --{}--", full_pattern);

    let contents = tokio::fs::read_to_string(full_pattern)
        .await
        .unwrap_or_else(|_| "".to_string());
    Html(contents)
}

// get css or js files assets
#[handler]
async fn assets(Path(filename): Path<String>) -> Response {
    if !filename.starts_with("index-") {
        let response = Response::from(StatusCode::NOT_FOUND);
        return response;
    }

    let directory = "dist/assets";
    let cwd = std::env::current_dir().unwrap();
    let cwd_as_str = cwd.to_str().unwrap();
    let full_pattern = format!("{}/app/{}/{}", cwd_as_str, directory, filename);

    let contents = std::fs::read_to_string(full_pattern).unwrap();
    match filename.ends_with(".js") {
        true => {
            let mut response = Response::from(contents);
            // set mime type
            response
                .headers_mut()
                .insert("content-type", "application/javascript".parse().unwrap());
            return response;
        }
        false => {
            let mut response = Response::from(contents);
            // set mime type
            response
                .headers_mut()
                .insert("content-type", "text/css".parse().unwrap());
            return response;
        }
    }
}

#[handler]
async fn caption(mut multipart: Multipart, shared: Data<&Shared>) -> SSE {
    let (sender, receiver) = mpsc::channel::<String>(100); // buffer size of 100
    let stream = ReceiverStream::new(receiver).map(Event::message); // Map the received message into an SSE event
    let sse = SSE::new(stream).keep_alive(Duration::from_secs(2));

    send_and_wait(
        sender.clone(),
        json!({"status": "Loading model..."}).to_string(),
    )
    .await;

    // read multipart form data
    let mut image_bytes: Option<Vec<u8>> = None;
    while let Ok(Some(field)) = multipart.next_field().await {
        let file_name = field.file_name().map(ToString::to_string);
        if let Ok(bytes) = field.bytes().await {
            // the file must end in .jpg
            if let Some(file_name) = file_name {
                println!("Got file: {}", file_name);
                image_bytes = Some(bytes);
            }
        }
    }

    if let Some(bytes) = image_bytes {
        println!("Got image bytes");
        let shared2 = shared.clone();
        tokio::spawn(async move {
            send_and_wait(
                sender.clone(),
                json!({"status": "Loading model..."}).to_string(),
            )
            .await;

            let mut shared_model = shared2.blip.lock().await;
            send_and_wait(
                sender.clone(),
                json!({"status": "Loading image..."}).to_string(),
            )
            .await;

            println!("Got image bytes");

            let caption = match model::detect(&mut shared_model, &bytes, sender.clone()).await {
                Ok(output) => output,
                Err(e) => {
                    println!("Error: {:?}", e);
                    format!("Error: {:?}", e)
                }
            };

            drop(shared_model); // unlock the mutex
            if SEND_END {
                send_and_wait(sender.clone(), json!({"caption": caption}).to_string()).await;
            }
            send_and_wait(sender.clone(), json!({"status": "Done"}).to_string()).await;
        });
    } else {
        send_and_wait(
            sender.clone(),
            json!({"status": "Error: no image found"}).to_string(),
        )
        .await;
        return sse;
    }
    sse
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "poem=debug");
    }
    tracing_subscriber::fmt::init();
    let shared_data = Shared::new(false);
    let new_app = || {
        Route::new()
            .at("/", get(index))
            .at("/assets/:filename", get(assets))
            .at("/api/cap", post(caption))
            .at("/cap", post(caption))
            .data(shared_data)
            .with(Tracing)
    };
    Server::new(TcpListener::bind("0.0.0.0:3000"))
        .name("hello-world")
        .run(new_app())
        .await
}
