use crate::domain::model::Link;
use std::path::Path;

pub trait OutputFormatter {
    fn format(
        &self,
        links: &[Link],
        output_path: Option<&Path>,
    ) -> Result<(), Box<dyn std::error::Error>>;
}
