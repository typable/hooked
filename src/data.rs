use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Payload {
    #[serde(rename = "ref")]
    pub branch: String,
    pub repository: Repository,
}

#[derive(Debug, Deserialize)]
pub struct Repository {
    #[serde(rename = "full_name")]
    pub id: String,
}
