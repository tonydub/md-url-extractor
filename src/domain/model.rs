use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct Link {
    pub url: String,
    pub source_file: PathBuf,
    pub link_text: String,
}
