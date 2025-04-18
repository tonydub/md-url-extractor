use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = env!("CARGO_PKG_NAME"),
    version = env!("CARGO_PKG_VERSION"),
    author = env!("CARGO_PKG_AUTHORS"),
    about = env!("CARGO_PKG_DESCRIPTION"),
)]
pub struct CliArgs {
    /// Directory containing Markdown files
    #[arg(value_parser = clap::value_parser!(PathBuf))]
    pub input_dir: PathBuf,

    /// Output format (stdout, text, csv, html)
    #[arg(
        short = 'f',
        long = "format",
        value_parser = ["stdout", "text", "csv", "html"],
        default_value = "stdout"
    )]
    pub output_format: String,

    /// Output file path
    #[arg(short = 'o', long = "output", value_parser = clap::value_parser!(PathBuf))]
    pub output_path: Option<PathBuf>,

    /// Filter URLs by domain
    #[arg(long = "domain")]
    pub filter_domain: Option<String>,

    /// Filter URLs by protocol (http, https, etc.)
    #[arg(
        long = "protocol",
        value_parser = ["http", "https", "ftp", "file", "mailto"],
        default_values = ["http", "https"],
        action = clap::ArgAction::Append,
    )]
    pub filter_protocol: Vec<String>,
}

pub fn parse_args() -> Result<CliArgs, Box<dyn std::error::Error>> {
    Ok(CliArgs::parse())
}
