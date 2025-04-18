use super::formatter::OutputFormatter;
use crate::domain::model::Link;
use std::fs::File;
use std::io::{self, Write};
use std::path::Path;

pub struct HtmlFormatter;

impl OutputFormatter for HtmlFormatter {
    fn format(
        &self,
        links: &[Link],
        output_path: Option<&Path>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let output_path = output_path.ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "Output path required for HTML format",
            )
        })?;

        let mut file = File::create(output_path)?;

        writeln!(file, "<!DOCTYPE NETSCAPE-Bookmark-file-1>")?;
        writeln!(file, "<!-- This is an automatically generated file. -->")?;
        writeln!(
            file,
            "<META HTTP-EQUIV=\"Content-Type\" CONTENT=\"text/html; charset=UTF-8\">"
        )?;
        writeln!(file, "<TITLE>Bookmarks</TITLE>")?;
        writeln!(file, "<H1>Bookmarks</H1>")?;
        writeln!(file, "<DL><p>")?;

        for link in links {
            let title = if link.link_text.is_empty() {
                &link.url
            } else {
                &link.link_text
            };
            writeln!(file, "    <DT><A HREF=\"{}\">{}</A>", link.url, title)?;
            writeln!(
                file,
                "    <DD>Source: {}",
                link.source_file.to_string_lossy()
            )?;
        }

        writeln!(file, "</DL><p>")?;
        Ok(())
    }
}
