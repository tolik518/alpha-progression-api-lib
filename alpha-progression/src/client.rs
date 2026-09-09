pub(crate) const API_BASE: &str = "https://alphaprogression.com/api/v3";
pub(crate) const CLOUDANT_BASE: &str =
    "https://8e4fc8f1-3681-4dc2-8215-45ad5f9233e6-bluemix.cloudant.com";

pub struct AlphaProgressionClient {
    pub(crate) http: reqwest::Client,
}

impl AlphaProgressionClient {
    pub fn new() -> Self {
        Self {
            http: reqwest::Client::new(),
        }
    }
}

impl Default for AlphaProgressionClient {
    fn default() -> Self {
        Self::new()
    }
}
