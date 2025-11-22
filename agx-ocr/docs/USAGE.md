# Usage

## Basic OCR

```bash
cat invoice.png | agx-ocr --model-path /models/deepseek-ocr-q4.gguf > out.json
```

If no `--model-path` is supplied and `$MODEL_PATH` is unset, `agx-ocr` will exit
with a non-zero code and write a helpful error message to `stderr`.

## Describe contract

To obtain a machine-readable model card (for `agx` tool registry or other AUs):

```bash
agx-ocr --describe
```

This prints JSON following the AGEniX `describe.schema.json` contract, suitable
for ingestion into the central registry or planner.
