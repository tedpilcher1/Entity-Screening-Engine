use std::{collections::HashMap, env};

use reqwest::Client;

use super::types::FlagSearchResponse;

const FLAG_SEARCH_URL: &str = "https://api.opensanctions.org/search/default";

pub struct OpenSanctionsClient {
    client: Client,
}

impl OpenSanctionsClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    pub async fn get_flags(
        &self,
        individual_name: String,
    ) -> Result<FlagSearchResponse, failure::Error> {
        let mut params = HashMap::new();
        params.insert(
            "api_key",
            env::var("OPEN_SANCTIONS_API_KEY").expect("API KEY should be set"),
        );
        params.insert("q", individual_name);

        let response = self
            .client
            .get(FLAG_SEARCH_URL)
            .query(&params)
            .send()
            .await?;
        let search_result: FlagSearchResponse = response.json().await?;

        Ok(search_result)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use dotenv::dotenv;

    #[tokio::test]
    async fn test1() {
        dotenv().ok();
        let client = OpenSanctionsClient::new();
        let result = client.get_flags("boris johnson".to_string()).await.unwrap();

        if let Some(entity) = result.results.first() {
            for (key, value) in entity.properties.to_owned().into_iter() {
                println!("{:?} : {:?}", key, value)
            }
        }
    }
}
