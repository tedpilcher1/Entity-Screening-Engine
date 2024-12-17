use std::env;

use bytes::Bytes;
use futures::Stream;
use lazy_static::lazy_static;
use reqwest::{header, Client};

const COMPANY_STREAMING_URL: &str = "https://stream.companieshouse.gov.uk/companies";

lazy_static! {
    static ref API_KEY: String =
        env::var("COMPANY_HOUSE_STREAMING_API_KEY").expect("Streaming API KEY should be set");
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

    pub async fn connect_to_company_stream(
        &self,
    ) -> Result<impl Stream<Item = Result<Bytes, reqwest::Error>>, failure::Error> {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            "Authorization",
            header::HeaderValue::from_str(&format!("{}", API_KEY.as_str()))?,
        );

        let stream = self
            .client
            .get(COMPANY_STREAMING_URL)
            .headers(headers)
            .send()
            .await?
            .bytes_stream();

        Ok(stream)
    }
}
