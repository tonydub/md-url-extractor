use super::formatter::OutputFormatter;
use crate::domain::model::Link;
use std::path::Path;

pub struct StdoutFormatter;

impl OutputFormatter for StdoutFormatter {
    fn format(
        &self,
        links: &[Link],
        _output_path: Option<&Path>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for link in links {
            println!("{}", link.url);
        }
        Ok(())
    }
}
