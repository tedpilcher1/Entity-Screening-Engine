use std::env;

use futures::StreamExt;
use reqwest::{header, Client};
use lazy_static::lazy_static;


const COMPANY_STREAMING_URL: &str = "https://stream.companieshouse.gov.uk/companies";

lazy_static! {
    static ref API_KEY: String = env::var("COMPANY_HOUSE_STREAMING_API_KEY").expect("Streaming API KEY should be set");
}

pub struct CompanyHouseStreamingClient {
    client: Client,
}

impl CompanyHouseStreamingClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    pub async fn connect_to_company_stream(&self) -> Result<(), failure::Error> {
        
        let mut headers = header::HeaderMap::new();
        headers.insert(
            "Authorization",
            header::HeaderValue::from_str(&format!("{}", API_KEY.as_str()))?,
        );
        
        let mut stream = self.client
            .get(COMPANY_STREAMING_URL)
            .headers(headers)
            .send()
            .await?
            .bytes_stream();

        while let Some(item) = stream.next().await {
            println!("Chunk: {:?}", item?);
        }

        Ok(())
    }
}