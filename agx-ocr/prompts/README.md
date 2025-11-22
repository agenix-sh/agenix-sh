# Prompt Templates for agx-ocr

This directory contains example prompts for different OCR use cases.

## Usage

### Method 1: Direct prompt string
```bash
cat chart.png | agx-ocr "<image>\nExtract chart data as JSON" --model-path ~/models/deepseek-ocr
```

### Method 2: Positional argument
```bash
cat chart.png | agx-ocr "$(cat prompts/chart-to-json.txt)" --model-path ~/models/deepseek-ocr
```

### Method 3: --prompt flag
```bash
cat chart.png | agx-ocr --model-path ~/models/deepseek-ocr --prompt "$(cat prompts/chart-to-json.txt)"
```

## Available Prompts

### chart-to-json.txt
Extract structured data from charts (bar, line, pie) as JSON.

**Best for:**
- Bar charts with categories and values
- Line charts with data points
- Pie charts with percentages

**Example:**
```bash
cat sales-chart.png | agx-ocr "$(cat prompts/chart-to-json.txt)" --model-path ~/models/deepseek-ocr
```

### table-to-structured.txt
Convert visual tables to structured text or JSON.

**Best for:**
- Financial tables
- Data grids
- Spreadsheet screenshots

**Example:**
```bash
cat table.png | agx-ocr "$(cat prompts/table-to-structured.txt)" --model-path ~/models/deepseek-ocr
```

## Creating Custom Prompts

All prompts must include the `<image>` token to indicate where the image should be placed.

**Template:**
```
<image>
[Your specific instructions here]
[Desired output format]
```

**Tips:**
1. Be specific about what data to extract
2. Specify the output format (JSON, CSV, bullet points, etc.)
3. Include units or formatting requirements
4. Use structured prompts for charts and tables

## Examples

### Extract only numbers
```
<image>
List all numbers found in this image, one per line.
```

### Invoice data extraction
```
<image>
Extract the following from this invoice:
- Invoice number
- Date
- Total amount
- Line items with descriptions and prices

Format as JSON.
```

### Chart with specific focus
```
<image>
This is a quarterly sales chart.
Extract Q1, Q2, Q3, and Q4 sales figures.
Include the year if shown.
Format as: Q1: $X, Q2: $Y, etc.
```
