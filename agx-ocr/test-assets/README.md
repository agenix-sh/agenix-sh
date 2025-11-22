# Test Assets for agx-ocr

This directory contains test images for validating the OCR functionality.

## Creating Test Images

You can create simple test images using ImageMagick:

```bash
# Install ImageMagick (if not already installed)
brew install imagemagick  # macOS
# or
sudo apt-get install imagemagick  # Ubuntu

# Create a simple text image
convert -size 800x200 -background white -fill black \
  -font Arial -pointsize 48 -gravity center \
  label:"Hello World OCR Test" \
  test-assets/sample-text.png

# Create a more complex multi-line text image
convert -size 800x400 -background white -fill black \
  -font Arial -pointsize 24 \
  label:"Line 1: Invoice Number\nLine 2: Customer Details\nLine 3: Total Amount: \$1,234.56" \
  test-assets/sample-invoice.png

# Create an image with mixed content
convert -size 1000x600 -background white -fill black \
  -font Arial -pointsize 18 \
  label:"Address: 123 Main St\nCity: San Francisco, CA 94102\nPhone: (555) 123-4567\nEmail: test@example.com" \
  test-assets/sample-mixed.png
```

## Using Test Images

Test images can be piped to agx-ocr:

```bash
# Test with a sample image
cat test-assets/sample-text.png | ./target/debug/agx-ocr --model-path ~/models/deepseek-ocr

# Expected output (JSON):
# {
#   "text": "Hello World OCR Test",
#   "regions": [],
#   "model": "deepseek-ocr (...)"
# }
```

## Sample Images

You can also download real-world test images:

```bash
# Download a sample receipt image
curl -o test-assets/receipt.jpg "https://raw.githubusercontent.com/tesseract-ocr/tesseract/main/testing/phototest.tif"

# Download a sample document
curl -o test-assets/document.png "https://raw.githubusercontent.com/tesseract-ocr/tesseract/main/testing/eurotext.tif"
```

## Test Image Requirements

For best OCR results, test images should:
- Be high resolution (at least 300 DPI for scanned documents)
- Have good contrast between text and background
- Use supported formats: PNG, JPEG, TIFF
- Have clear, readable text

## Programmatic Test Image Generation

You can also use Python with PIL/Pillow:

```python
from PIL import Image, ImageDraw, ImageFont

# Create a white background
img = Image.new('RGB', (800, 200), color='white')
d = ImageDraw.Draw(img)

# Add text
font = ImageFont.truetype('/System/Library/Fonts/Helvetica.ttc', 48)
d.text((50, 50), "Test OCR Image", fill='black', font=font)

# Save
img.save('test-assets/python-test.png')
```
