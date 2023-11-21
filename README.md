# candle-ocr-server

This is a small server and web app that leverages the open source transformer based OCR model `microsoft/trocr-base-handwritten` for optical character recognition from freeform text and photos. 

This app is built on `candle`, `poem`, `svelte` and `fly`.

### TLDR; 🕹️ Play with it now

Try it out at https://ocr.dholtz.com

**Note: the app performance may be impacted by other users since the model is running on a small CPU bound VM.

# oooo, show me what it does 🚀

## Detect from freeform

Note: the model has been trained on single line text and works best with similar input

<img width="700px" src="https://github.com/drbh/candle-ocr-server/assets/9896130/eacc1eac-e6f0-4630-b644-964379d8970a" alt="handwritedetect" />

## Select handwriting subset

Using select mode we can can analyze specific sections of handwriting

<img width="700px" src="https://github.com/drbh/candle-ocr-server/assets/9896130/49f8119d-9713-4ccb-b2a2-f11dade94873" alt="handwriteselect" />

## Drop in a photo

In a more practical example we can drag in a screenshot.

<img width="700px" src="https://github.com/drbh/candle-ocr-server/assets/9896130/a67528b9-1b11-4cdd-85bb-60851e5d9f8a" alt="dropfileselect" />

## Select sections

and now we can select sections of text to run detection on

<img width="700px" src="https://github.com/drbh/candle-ocr-server/assets/9896130/e89d8299-c7f7-4d95-b1a5-834de6ed656e" alt="selectscreen" />

