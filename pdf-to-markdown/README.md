# PDF to Markdown Converter

A Rust command-line tool that converts PDF files to Markdown format.

## Features

- Extracts text from PDF files
- Converts to clean Markdown format
- Detects and formats headings
- Preserves list items
- Handles paragraphs intelligently
- Cleans up spacing and punctuation

## Installation

```bash
cargo build --release
```

## Usage

```bash
# Basic usage (creates input.md from input.pdf)
./target/release/pdf-to-markdown input.pdf

# Specify output file
./target/release/pdf-to-markdown input.pdf -o output.md

# Verbose mode
./target/release/pdf-to-markdown input.pdf -v
```

## Command Line Options

- `input` - Path to the PDF file (required)
- `-o, --output` - Output markdown file (default: input_file.md)
- `-v, --verbose` - Enable verbose output
- `-h, --help` - Print help information

## Example

```bash
./target/release/pdf-to-markdown document.pdf -o document.md
```

This will convert `document.pdf` to `document.md` with:
- Headings detected and formatted as Markdown headers
- Lists converted to Markdown bullet points
- Paragraphs properly separated
- Clean formatting

## Limitations

- Works best with text-based PDFs
- May not preserve complex formatting or tables
- Does not extract images from PDFs