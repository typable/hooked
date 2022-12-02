use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Payload {
    pub repository: Repository,
}

#[derive(Debug, Deserialize)]
pub struct Repository {
    #[serde(rename = "full_name")]
    pub id: String,
}
