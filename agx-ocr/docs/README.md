# agx-ocr

`agx-ocr` is an AGEniX Agentic Unit (AU) that performs OCR using locally hosted DeepSeek GGUF models.

- Reads **binary image data** from `stdin` (PNG, JPEG).
- Writes **structured JSON** to `stdout`.
- Logs diagnostics and errors to `stderr`.
- Requires a **local GGUF model path**; it does not download models automatically.

This AU is designed to be orchestrated by `agx` (the AGEniX planner) and executed
on workers (`agw`) in a zero-trust environment.
