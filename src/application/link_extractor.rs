use indicatif::{ProgressBar, ProgressStyle};
use pulldown_cmark::{Event, Parser, Tag};
use rayon::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use walkdir::WalkDir;

use crate::domain::model::Link;
use crate::domain::services::url_cleaner::{
    CompositeUrlCleaner, TrackerParamCleaner, YouTubeUrlCleaner,
};
use crate::domain::services::url_processor::{UrlProcessor, UrlProcessorConfig};

/// Service that orchestrates link extraction and processing workflow
pub struct LinkExtractorService;

impl LinkExtractorService {
    pub fn new() -> Self {
        Self
    }

    /// Main orchestration method to extract and process links from markdown files
    pub fn extract_and_process_links(
        &self,
        input_dir: &Path,
        filter_domain: Option<String>,
        filter_protocol: Vec<String>,
    ) -> Result<Vec<Link>, Box<dyn std::error::Error>> {
        // Find markdown files in directory
        let markdown_files = self.scan_directory(input_dir)?;

        // Extract links from the files
        let links = self.extract_links_from_files(markdown_files)?;

        // Create URL processor with cleaners
        let url_processor = self.create_url_processor(UrlProcessorConfig {
            filter_domain,
            filter_protocol,
        });

        // Process and clean the links
        let processed_links = url_processor.process_urls(links)?;

        Ok(processed_links)
    }

    /// Scan directory recursively for markdown files
    pub fn scan_directory(&self, dir: &Path) -> Result<Vec<PathBuf>, std::io::Error> {
        let mut markdown_files = Vec::new();

        for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_file() && path.extension().is_some_and(|ext| ext == "md") {
                markdown_files.push(path.to_path_buf());
            }
        }

        Ok(markdown_files)
    }

    /// Extract links from multiple files in parallel
    pub fn extract_links_from_files(
        &self,
        files: Vec<PathBuf>,
    ) -> Result<Vec<Link>, Box<dyn std::error::Error>> {
        let progress_bar = ProgressBar::new(files.len() as u64);
        progress_bar.set_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} files")?
                .progress_chars("##-"),
        );

        let links = Arc::new(Mutex::new(Vec::new()));

        files.par_iter().for_each(|file| {
            let file_content = match fs::read_to_string(file) {
                Ok(content) => content,
                Err(err) => {
                    eprintln!("Error reading file {:?}: {}", file, err);
                    return;
                }
            };

            let file_links = self.extract_links_from_markdown(&file_content, file);

            if !file_links.is_empty() {
                if let Ok(mut infos) = links.lock() {
                    infos.extend(file_links);
                }
            }

            progress_bar.inc(1);
        });

        progress_bar.finish_with_message("URL extraction complete");

        let result = match Arc::try_unwrap(links) {
            Ok(mutex) => mutex.into_inner().map_err(|_| "Failed to unwrap Mutex")?,
            Err(_) => return Err("Failed to unwrap Arc".into()),
        };
        Ok(result)
    }

    /// Extract links from a single markdown file
    fn extract_links_from_markdown(&self, content: &str, source_file: &Path) -> Vec<Link> {
        let mut links = Vec::new();
        let parser = Parser::new(content);

        let mut current_link: Option<(String, String)> = None;

        for event in parser {
            match event {
                Event::Start(Tag::Link(_, destination, _)) => {
                    current_link = Some((destination.to_string(), String::new()));
                }
                Event::Text(text) if current_link.is_some() => {
                    if let Some((_, ref mut link_text)) = current_link {
                        link_text.push_str(text.as_ref());
                    }
                }
                Event::End(Tag::Link(_, _, _)) => {
                    if let Some((url, link_text)) = current_link.take() {
                        links.push(Link {
                            url,
                            source_file: source_file.to_path_buf(),
                            link_text,
                        });
                    }
                }
                _ => {}
            }
        }

        links
    }

    /// Create URL processor with all needed cleaners
    fn create_url_processor(&self, config: UrlProcessorConfig) -> UrlProcessor {
        let mut composite_cleaner = CompositeUrlCleaner::new();
        composite_cleaner
            .add_cleaner(Box::new(TrackerParamCleaner::new()))
            .add_cleaner(Box::new(YouTubeUrlCleaner::new()));

        UrlProcessor::new(config, Box::new(composite_cleaner))
    }
}
