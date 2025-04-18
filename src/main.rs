mod domain {
    pub mod model;
    pub mod services {
        pub mod url_cleaner;
        pub mod url_processor;
    }
}

mod application {
    pub mod link_extractor;
}

mod infrastructure {
    pub mod cli;
    pub mod output;
}

use application::link_extractor::LinkExtractorService;
use infrastructure::cli::args;
use infrastructure::output::OutputFormatterFactory;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command-line arguments
    let cli_args = args::parse_args()?;

    // Create application service
    let link_extractor = LinkExtractorService::new();

    // Extract and process links
    let processed_links = link_extractor.extract_and_process_links(
        &cli_args.input_dir,
        cli_args.filter_domain.clone(),
        cli_args.filter_protocol.clone(),
    )?;

    // Get appropriate formatter from factory and format output
    let formatter = OutputFormatterFactory::create_formatter(&cli_args.output_format);
    let output_path = cli_args.output_path.as_deref();
    formatter.format(&processed_links, output_path)?;

    Ok(())
}
