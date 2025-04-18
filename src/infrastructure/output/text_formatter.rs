use super::formatter::OutputFormatter;
use crate::domain::model::Link;
use std::fs::File;
use std::io::{self, Write};
use std::path::Path;

pub struct TextFormatter;

impl OutputFormatter for TextFormatter {
    fn format(
        &self,
        links: &[Link],
        output_path: Option<&Path>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let output_path = output_path.ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "Output path required for text format",
            )
        })?;

        let mut file = File::create(output_path)?;
        for link in links {
            writeln!(file, "{}", link.url)?;
        }
        Ok(())
    }
}
