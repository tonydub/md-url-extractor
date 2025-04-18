use super::formatter::OutputFormatter;
use crate::domain::model::Link;
use std::io;
use std::path::Path;

pub struct CsvFormatter;

impl OutputFormatter for CsvFormatter {
    fn format(
        &self,
        links: &[Link],
        output_path: Option<&Path>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let output_path = output_path.ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "Output path required for CSV format",
            )
        })?;

        let mut wtr = csv::Writer::from_path(output_path)?;
        wtr.write_record(["URL", "Source File", "Link Text"])?;

        for link in links {
            wtr.write_record([
                &link.url,
                &link.source_file.to_string_lossy().to_string(),
                &link.link_text,
            ])?;
        }

        wtr.flush()?;
        Ok(())
    }
}
