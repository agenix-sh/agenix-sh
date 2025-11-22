# Chart Extraction & Custom Prompts

## DeepSeek-OCR Chart Capabilities

### What DeepSeek-OCR Can Do 🎯

DeepSeek-OCR is specifically designed to handle **chart interpretation and data extraction**:

**Supported Chart Types:**
- ✅ Bar charts - Extract values from bars
- ✅ Line charts - Read data points
- ✅ Pie charts - Parse percentages and labels
- ✅ Tables - Parse structured data
- ✅ Financial charts - Extract trading data
- ✅ Diagrams - Understand visual relationships

**Capabilities:**
1. **Deep Chart Parsing** - Extracts data points, labels, and values
2. **Table Recognition** - Converts visual tables to structured data
3. **Multi-modal Understanding** - Text + visual structure
4. **Data Extraction** - Reads numeric values from visualizations

### Performance

According to research:
- **Accuracy**: 97% on chart extraction tasks
- **Output Format**: Structured (typically HTML tables or JSON)
- **Compression**: 10-20x token compression vs raw OCR

### Limitations

**Verbosity**: Raw output can be verbose with many HTML tags (`<td>`, `<tr>`)
**Complex Charts**: May duplicate entries or lose higher-level structure on very complex visualizations
**Better for**: Clean, well-labeled charts vs hand-drawn or low-quality images

---

## Current Implementation Status

### What's Implemented ✅

Our current `agx-ocr` uses a **fixed, generic prompt**:

```rust
// src/ocr.rs:121
let prompt = "<image>\nExtract all text from this image.";
```

This works for general OCR but doesn't leverage DeepSeek's chart-specific capabilities.

### What's NOT Implemented ❌

- ❌ **Custom prompts** via CLI arguments
- ❌ **Chart-specific prompts** for data extraction
- ❌ **Structured output modes** (e.g., JSON, CSV for charts)
- ❌ **Prompt templates** for different use cases

---

## Proposal: Add Prompt Support

### 1. Add CLI Arguments

Update `src/main.rs`:

```rust
#[derive(Parser, Debug)]
struct Cli {
    /// Path to DeepSeek OCR model
    #[arg(long = "model-path", env = "MODEL_PATH")]
    model_path: Option<PathBuf>,

    /// Print AU model description
    #[arg(long = "describe")]
    describe: bool,

    /// Custom prompt (use <image> token for image placement)
    #[arg(long = "prompt")]
    prompt: Option<String>,

    /// Prompt file path (overrides --prompt)
    #[arg(long = "prompt-file")]
    prompt_file: Option<PathBuf>,
}
```

### 2. Update OCR Function Signature

Modify `src/ocr.rs`:

```rust
pub fn run_ocr(
    image_bytes: &[u8],
    cfg: &ModelConfig,
    prompt: Option<&str>  // NEW: Optional custom prompt
) -> Result<OcrResult>
```

### 3. Default Prompts for Different Use Cases

```rust
// Default prompts
const DEFAULT_PROMPT: &str = "<image>\nExtract all text from this image.";
const CHART_PROMPT: &str = "<image>\nExtract all data from this chart. Include labels, values, and units. Format as structured data.";
const TABLE_PROMPT: &str = "<image>\nConvert this table to structured format. Preserve all rows, columns, and headers.";
const FINANCIAL_PROMPT: &str = "<image>\nExtract all financial data including numbers, dates, and metrics from this document.";
```

### 4. Example Usage

```bash
# Generic OCR (default)
cat image.png | agx-ocr --model-path ~/models/deepseek-ocr

# Chart extraction
cat chart.png | agx-ocr \
  --model-path ~/models/deepseek-ocr \
  --prompt "<image>\nExtract all bar chart data as JSON with labels and values."

# Table extraction
cat table.png | agx-ocr \
  --model-path ~/models/deepseek-ocr \
  --prompt-file prompts/table-to-csv.txt

# Financial document
cat invoice.png | agx-ocr \
  --model-path ~/models/deepseek-ocr \
  --prompt "<image>\nExtract invoice number, date, total amount, and line items."
```

---

## Example Chart Extraction Prompts

### Bar Chart Data Extraction

**Prompt:**
```
<image>
Extract all data from this bar chart.
For each bar, provide:
- Label/category
- Value
- Units (if shown)

Format as JSON array.
```

**Expected Output:**
```json
{
  "text": "[{\"label\": \"Q1\", \"value\": 125000, \"unit\": \"$\"}, ...]",
  "regions": [],
  "model": "deepseek-ocr"
}
```

### Line Chart Trend Analysis

**Prompt:**
```
<image>
Analyze this line chart and extract:
1. All data points with x and y coordinates
2. Trend direction (increasing/decreasing)
3. Min and max values
4. Legend labels
```

### Pie Chart Breakdown

**Prompt:**
```
<image>
Extract all segments from this pie chart.
For each segment provide:
- Label
- Percentage
- Value (if shown)

Sort by percentage descending.
```

### Financial Chart

**Prompt:**
```
<image>
Extract all trading data from this financial chart:
- Date/time axis values
- Price levels (open, high, low, close)
- Volume bars (if present)
- Any technical indicators shown

Format as CSV.
```

---

## Implementation Plan

### Phase 1: Basic Prompt Support (Quick Win)

**Tasks:**
1. Add `--prompt` CLI argument
2. Pass prompt to `run_ocr()`
3. Use custom prompt if provided, else default
4. Update tests

**Time**: ~1 hour
**Complexity**: Low
**Value**: High - Enables chart extraction immediately

### Phase 2: Prompt Templates

**Tasks:**
1. Create `prompts/` directory with templates
2. Add `--prompt-file` support
3. Create pre-built templates:
   - `chart-to-json.txt`
   - `table-to-csv.txt`
   - `financial-extract.txt`
4. Document template format

**Time**: ~2 hours
**Complexity**: Low
**Value**: Medium - Better UX

### Phase 3: Structured Output Modes

**Tasks:**
1. Add `--output-format` flag (json/csv/markdown)
2. Post-process DeepSeek output
3. Parse HTML tables to CSV
4. Add schema validation

**Time**: ~4 hours
**Complexity**: Medium
**Value**: High - Production-ready outputs

---

## Chart Extraction Experiment

### Immediate Experiment (No Code Changes)

You can test chart extraction **right now** by modifying the prompt in `src/ocr.rs:121`:

```rust
// Change this line:
let prompt = "<image>\nExtract all text from this image.";

// To this for chart extraction:
let prompt = "<image>\nExtract all data from this chart including labels, values, and units. Format as structured data.";
```

Then rebuild and test:
```bash
cargo build --release
cat my-chart.png | ./target/release/agx-ocr --model-path ~/models/deepseek-ocr
```

### Test Images to Try

1. **Simple bar chart** - Sales by quarter
2. **Line chart** - Stock prices over time
3. **Pie chart** - Market share breakdown
4. **Financial chart** - Trading data with indicators
5. **Complex table** - Financial statements

### Expected Results

**Before (generic prompt):**
```json
{
  "text": "Q1 $125K Q2 $150K Q3 $175K Q4 $200K",
  ...
}
```

**After (chart-specific prompt):**
```json
{
  "text": "[\n  {\"quarter\": \"Q1\", \"sales\": 125000, \"currency\": \"USD\"},\n  {\"quarter\": \"Q2\", \"sales\": 150000, \"currency\": \"USD\"},\n  ...\n]",
  ...
}
```

---

## Recommended Next Steps

### 1. Quick Experiment (5 minutes)
```bash
# Edit src/ocr.rs line 121 to use chart prompt
# Rebuild
cargo build --release

# Test with a chart image
cat test-chart.png | ./target/release/agx-ocr --model-path ~/models/deepseek-ocr
```

### 2. If Successful: Add Prompt Support (1 hour)

Implement Phase 1 to make prompts configurable via CLI.

### 3. Create Prompt Library

Build a collection of tested prompts for common use cases.

### 4. Integrate into AGEniX

Add prompt passing to the AU contract so orchestrators can specify extraction behavior.

---

## AU Contract Enhancement

### Current Contract
```bash
cat image.png | agx-ocr --model-path /models/deepseek-ocr
```

### Proposed Enhanced Contract
```bash
cat image.png | agx-ocr \
  --model-path /models/deepseek-ocr \
  --prompt "Extract chart data as JSON" \
  --context "Financial Q4 report"
```

### Benefits for AGEniX
1. **Flexible**: Same binary, different extraction modes
2. **Composable**: Orchestrator controls behavior
3. **Traceable**: Prompts logged in AU output
4. **Reusable**: Share prompts across AUs

---

## Answers to Your Questions

### Q1: Can we experiment with chart extraction?

**✅ YES!** DeepSeek-OCR has excellent chart parsing capabilities.

**How:**
1. **Quick test**: Edit `src/ocr.rs:121` to use a chart-specific prompt
2. **Proper implementation**: Add `--prompt` CLI argument (Phase 1)

**What to expect:**
- Structured data extraction from charts
- Labels, values, and units parsed
- Works well on clean, well-labeled charts
- May be verbose on complex visualizations

### Q2: Do we support prompts/context via CLI?

**❌ NOT YET** - but it's easy to add!

**Current state:**
- Fixed prompt hardcoded in `src/ocr.rs:121`
- No CLI argument for custom prompts

**To implement:**
- Add `--prompt` and `--prompt-file` arguments (like deepseek-ocr-cli has)
- Pass through to the `decode()` function
- 1-2 hours of work maximum

**Would it help?**
- **YES!** Essential for chart extraction
- Allows task-specific instructions
- Makes AU more flexible and powerful
- Aligns with AGEniX composability principles

---

## Summary

✅ **Chart Extraction**: Fully supported by DeepSeek-OCR model
❌ **Custom Prompts**: Not yet implemented in agx-ocr CLI
⚡ **Quick Win**: Add `--prompt` argument (~1 hour)
🎯 **High Value**: Unlocks chart/table/diagram parsing capabilities

The model has the capability - we just need to expose it via the CLI interface!
