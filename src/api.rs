use serde::de::DeserializeOwned;

pub(crate) const BASE_URL: &str = "https://users.rust-lang.org";

pub struct Api;

impl Api {
    pub async fn get<T: DeserializeOwned>(path: &str) -> Result<T, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let url = format!("{}/{}", BASE_URL, path);
        let body = client.get(url).send().await?.text().await?;

        serde_json::from_str::<T>(body.as_str()).map_err(|e| e.into())
    }
}
