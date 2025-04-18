use std::collections::HashMap;
use url::Url;

use crate::domain::model::Link;
use crate::domain::services::url_cleaner::UrlCleaner;

pub struct UrlProcessorConfig {
    pub filter_domain: Option<String>,
    pub filter_protocol: Vec<String>,
}

pub struct UrlProcessor {
    config: UrlProcessorConfig,
    url_cleaner: Box<dyn UrlCleaner>,
}

impl UrlProcessor {
    pub fn new(config: UrlProcessorConfig, url_cleaner: Box<dyn UrlCleaner>) -> Self {
        Self {
            config,
            url_cleaner,
        }
    }

    pub fn process_urls(
        &self,
        url_infos: Vec<Link>,
    ) -> Result<Vec<Link>, Box<dyn std::error::Error>> {
        let mut unique_urls = HashMap::new();
        let base_url = Url::parse("http://base.example.com/")?;

        for info in url_infos {
            // Skip pure fragment/anchor links
            if info.url.starts_with('#') {
                continue;
            }

            // Try to parse the URL to validate and check against filters
            let url_str = &info.url;
            let parsed_url = Url::parse(url_str).or_else(|_| base_url.join(url_str));

            // Skip malformed URLs with a warning
            let parsed_url = match parsed_url {
                Ok(url) => url,
                Err(err) => {
                    eprintln!(
                        "Warning: Error parsing URL: {} from file {:?}: {}",
                        url_str, info.source_file, err
                    );
                    continue;
                }
            };

            // Apply domain filter if specified
            if let Some(ref domain) = self.config.filter_domain {
                if let Some(host) = parsed_url.host_str() {
                    if !host.contains(domain) {
                        continue;
                    }
                } else {
                    continue;
                }
            }

            // Apply protocol filter if specified
            if !self.config.filter_protocol.is_empty()
                && !self
                    .config
                    .filter_protocol
                    .contains(&parsed_url.scheme().to_string())
            {
                continue;
            }

            // Clean the URL
            let cleaned_info = self.url_cleaner.clean(&info);

            // Keep first occurrence of each URL (deduplicate)
            if !unique_urls.contains_key(&cleaned_info.url) {
                unique_urls.insert(cleaned_info.url.clone(), cleaned_info);
            }
        }

        Ok(unique_urls.into_values().collect())
    }
}
