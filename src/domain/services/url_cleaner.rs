use super::super::model::Link;
use url::Url;

pub trait UrlCleaner {
    fn clean(&self, link: &Link) -> Link;
}

pub struct TrackerParamCleaner;

impl TrackerParamCleaner {
    pub fn new() -> Self {
        Self {}
    }

    fn is_tracker_param(&self, param: &str) -> bool {
        param.starts_with("utm_")
            || param == "fbclid"
            || param == "gclid"
            || param == "msclkid"
            || param == "ref"
            || param == "source"
    }
}

impl UrlCleaner for TrackerParamCleaner {
    fn clean(&self, link: &Link) -> Link {
        let mut cleaned_link = link.clone();

        if let Ok(mut parsed_url) = Url::parse(&link.url) {
            let has_trackers = parsed_url
                .query_pairs()
                .any(|(k, _)| self.is_tracker_param(&k));

            if has_trackers {
                let mut pairs: Vec<(String, String)> = parsed_url
                    .query_pairs()
                    .filter(|(k, _)| !self.is_tracker_param(k))
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect();

                parsed_url.set_query(None);

                if !pairs.is_empty() {
                    pairs.sort_by(|(a, _), (b, _)| a.cmp(b));
                    parsed_url.set_query(Some(
                        &pairs
                            .iter()
                            .map(|(k, v)| format!("{}={}", k, v))
                            .collect::<Vec<String>>()
                            .join("&"),
                    ));
                }

                cleaned_link.url = parsed_url.to_string();
            }
        }

        cleaned_link
    }
}

pub struct YouTubeUrlCleaner;

impl YouTubeUrlCleaner {
    pub fn new() -> Self {
        Self {}
    }

    fn extract_video_id(&self, url_str: &str) -> Option<String> {
        if let Ok(url) = Url::parse(url_str) {
            // Check if it's a YouTube domain
            if let Some(host) = url.host_str() {
                if host.contains("youtube.com") || host == "youtu.be" {
                    // Handle youtube.com/watch?v=VIDEO_ID
                    if host.contains("youtube.com") {
                        if url.path() == "/watch" {
                            return url
                                .query_pairs()
                                .find(|(key, _)| key == "v")
                                .map(|(_, value)| value.to_string());
                        } else if url.path().starts_with("/embed/") || url.path().starts_with("/v/")
                        {
                            // Handle youtube.com/embed/VIDEO_ID or youtube.com/v/VIDEO_ID
                            let path_segments: Vec<&str> = url.path().split('/').collect();
                            return path_segments.get(2).map(|&id| id.to_string());
                        }
                    } else if host == "youtu.be" {
                        // Handle youtu.be/VIDEO_ID
                        let path = url.path();
                        if path.len() > 1 {
                            return Some((path[1..]).to_string());
                        }
                    }
                }
            }
        }
        None
    }
}

impl UrlCleaner for YouTubeUrlCleaner {
    fn clean(&self, link: &Link) -> Link {
        let mut cleaned_link = link.clone();

        if let Some(video_id) = self.extract_video_id(&link.url) {
            cleaned_link.url = format!("https://www.youtube.com/watch?v={}", video_id);
        }

        cleaned_link
    }
}

pub struct CompositeUrlCleaner {
    cleaners: Vec<Box<dyn UrlCleaner>>,
}

impl CompositeUrlCleaner {
    pub fn new() -> Self {
        Self {
            cleaners: Vec::new(),
        }
    }

    pub fn add_cleaner(&mut self, cleaner: Box<dyn UrlCleaner>) -> &mut Self {
        self.cleaners.push(cleaner);
        self
    }
}

impl UrlCleaner for CompositeUrlCleaner {
    fn clean(&self, link: &Link) -> Link {
        let mut result = link.clone();

        for cleaner in &self.cleaners {
            result = cleaner.clean(&result);
        }

        result
    }
}
