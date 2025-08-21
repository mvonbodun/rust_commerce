use anyhow::{Context, Result};
use clap::Parser;
use pdf_extract::extract_text_from_mem;
use std::fs;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(help = "Path to the PDF file")]
    input: PathBuf,

    #[arg(short, long, help = "Output markdown file (default: input_file.md)")]
    output: Option<PathBuf>,

    #[arg(short, long, help = "Verbose output")]
    verbose: bool,
}

fn pdf_to_markdown(input_path: &PathBuf, output_path: &PathBuf, verbose: bool) -> Result<()> {
    if verbose {
        println!("Reading PDF from: {}", input_path.display());
    }

    let bytes = fs::read(input_path)
        .with_context(|| format!("Failed to read PDF file: {}", input_path.display()))?;

    let output_string =
        extract_text_from_mem(&bytes).with_context(|| "Failed to extract text from PDF")?;

    let markdown = convert_to_markdown(&output_string);

    if verbose {
        println!("Writing Markdown to: {}", output_path.display());
    }

    fs::write(output_path, markdown)
        .with_context(|| format!("Failed to write output file: {}", output_path.display()))?;

    println!("✓ Successfully converted PDF to Markdown");
    println!("  Input:  {}", input_path.display());
    println!("  Output: {}", output_path.display());

    Ok(())
}

fn convert_to_markdown(text: &str) -> String {
    let mut markdown = String::new();
    let mut in_paragraph = false;
    let mut paragraph_buffer = String::new();

    for line in text.lines() {
        let trimmed = line.trim();

        if trimmed.is_empty() {
            if in_paragraph && !paragraph_buffer.is_empty() {
                markdown.push_str(&paragraph_buffer);
                markdown.push_str("\n\n");
                paragraph_buffer.clear();
            }
            in_paragraph = false;
            continue;
        }

        if looks_like_heading(trimmed) {
            if in_paragraph && !paragraph_buffer.is_empty() {
                markdown.push_str(&paragraph_buffer);
                markdown.push_str("\n\n");
                paragraph_buffer.clear();
                in_paragraph = false;
            }

            if trimmed.chars().all(|c| {
                c.is_uppercase() || c.is_whitespace() || c.is_numeric() || c.is_ascii_punctuation()
            }) && trimmed.len() < 100
                && !trimmed.ends_with('.')
            {
                markdown.push_str("## ");
                markdown.push_str(trimmed);
                markdown.push_str("\n\n");
            } else if is_numbered_heading(trimmed) {
                markdown.push_str("### ");
                markdown.push_str(trimmed);
                markdown.push_str("\n\n");
            } else {
                in_paragraph = true;
                if !paragraph_buffer.is_empty() {
                    paragraph_buffer.push(' ');
                }
                paragraph_buffer.push_str(trimmed);
            }
        } else if looks_like_list_item(trimmed) {
            if in_paragraph && !paragraph_buffer.is_empty() {
                markdown.push_str(&paragraph_buffer);
                markdown.push_str("\n\n");
                paragraph_buffer.clear();
                in_paragraph = false;
            }

            markdown.push_str("- ");
            let content = get_list_item_content(trimmed);
            markdown.push_str(content);
            markdown.push('\n');
        } else {
            in_paragraph = true;
            if !paragraph_buffer.is_empty() {
                paragraph_buffer.push(' ');
            }
            paragraph_buffer.push_str(trimmed);
        }
    }

    if !paragraph_buffer.is_empty() {
        markdown.push_str(&paragraph_buffer);
        markdown.push('\n');
    }

    markdown = post_process_markdown(markdown);

    markdown
}

fn looks_like_heading(line: &str) -> bool {
    if line.len() > 200 {
        return false;
    }

    let uppercase_ratio = line
        .chars()
        .filter(|c| c.is_alphabetic())
        .filter(|c| c.is_uppercase())
        .count() as f32
        / line.chars().filter(|c| c.is_alphabetic()).count().max(1) as f32;

    uppercase_ratio > 0.7 && !line.ends_with('.')
}

fn is_numbered_heading(line: &str) -> bool {
    let patterns = [
        r"^\d+\.",
        r"^\d+\.\d+",
        r"^[IVXLCDM]+\.",
        r"^[ivxlcdm]+\.",
        r"^[A-Z]\.",
        r"^[a-z]\.",
    ];

    for _pattern in patterns {
        if line.starts_with(char::is_numeric) || line.starts_with(char::is_alphabetic) {
            let parts: Vec<&str> = line.splitn(2, ' ').collect();
            if parts.len() == 2 {
                let first = parts[0];
                if first.ends_with('.') || first.ends_with(')') {
                    return true;
                }
            }
        }
    }

    false
}

fn looks_like_list_item(line: &str) -> bool {
    line.starts_with("• ")
        || line.starts_with("- ")
        || line.starts_with("* ")
        || line.starts_with("· ")
        || (line.starts_with("o ") && line.len() > 2)
}

fn get_list_item_content(line: &str) -> &str {
    // Handle multi-byte UTF-8 characters properly
    if line.starts_with("• ") {
        &line["• ".len()..]
    } else if line.starts_with("- ") {
        &line["- ".len()..]
    } else if line.starts_with("* ") {
        &line["* ".len()..]
    } else if line.starts_with("· ") {
        &line["· ".len()..]
    } else if line.starts_with("o ") && line.len() > 2 {
        &line["o ".len()..]
    } else {
        line
    }
}

fn post_process_markdown(mut markdown: String) -> String {
    markdown = markdown.replace("  ", " ");

    markdown = markdown.replace(" .", ".");
    markdown = markdown.replace(" ,", ",");
    markdown = markdown.replace(" ;", ";");
    markdown = markdown.replace(" :", ":");
    markdown = markdown.replace(" !", "!");
    markdown = markdown.replace(" ?", "?");

    while markdown.contains("\n\n\n") {
        markdown = markdown.replace("\n\n\n", "\n\n");
    }

    markdown = emphasize_special_text(markdown);

    markdown.trim().to_string()
}

fn emphasize_special_text(text: String) -> String {
    let mut result = String::new();
    let chars = text.chars().peekable();
    let mut in_quotes = false;

    for ch in chars {
        if ch == '"' {
            if in_quotes {
                result.push('"');
                in_quotes = false;
            } else {
                result.push('"');
                in_quotes = true;
            }
        } else if ch == '\'' || ch == '\u{2018}' || ch == '\u{2019}' {
            result.push('\'');
        } else {
            result.push(ch);
        }
    }

    result
}

fn main() -> Result<()> {
    let args = Args::parse();

    let output_path = args.output.unwrap_or_else(|| {
        let mut path = args.input.clone();
        path.set_extension("md");
        path
    });

    pdf_to_markdown(&args.input, &output_path, args.verbose)?;

    Ok(())
}
