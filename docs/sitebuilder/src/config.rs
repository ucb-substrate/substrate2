use serde::Deserialize;

#[derive(Deserialize)]
pub struct SiteConfig {
    pub versions: Vec<String>,
}
