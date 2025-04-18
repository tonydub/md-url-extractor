use super::csv_formatter::CsvFormatter;
use super::formatter::OutputFormatter;
use super::html_formatter::HtmlFormatter;
use super::stdout_formatter::StdoutFormatter;
use super::text_formatter::TextFormatter;

pub struct OutputFormatterFactory;

impl OutputFormatterFactory {
    pub fn create_formatter(format: &str) -> Box<dyn OutputFormatter> {
        match format {
            "text" => Box::new(TextFormatter),
            "csv" => Box::new(CsvFormatter),
            "html" => Box::new(HtmlFormatter),
            _ => Box::new(StdoutFormatter), // Default to stdout
        }
    }
}
