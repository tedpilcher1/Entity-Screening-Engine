use std::{collections::HashMap, env};

use bytes::Bytes;
use futures::Stream;
use lazy_static::lazy_static;
use log::info;
use reqwest::{header, Client};

use crate::workers::streaming_worker::StreamingKind;

const COMPANY_STREAMING_URL: &str = "https://stream.companieshouse.gov.uk/companies";
const OFFICER_STREAMING_URL: &str = "https://stream.companieshouse.gov.uk/officers";
const SHAREHOLDER_STREAMING_URL: &str =
    "https://stream.companieshouse.gov.uk/persons-with-significant-control";

lazy_static! {
    static ref API_KEY: String =
        env::var("COMPANY_HOUSE_STREAMING_API_KEY").expect("Streaming API KEY should be set");
}

pub struct CompanyHouseStreamingClient {
    client: Client,
    kind: StreamingKind,
}

impl CompanyHouseStreamingClient {
    pub fn new(kind: StreamingKind) -> Self {
        Self {
            client: Client::new(),
            kind,
        }
    }

    pub async fn connect_to_stream(
        &self,
        timepoint: Option<i32>,
    ) -> Result<impl Stream<Item = Result<Bytes, reqwest::Error>>, failure::Error> {
        let url = match self.kind {
            StreamingKind::Company => COMPANY_STREAMING_URL,
            StreamingKind::Officer => OFFICER_STREAMING_URL,
            StreamingKind::Shareholder => SHAREHOLDER_STREAMING_URL,
        };

        let mut params = HashMap::new();
        if let Some(timepoint) = timepoint {
            params.insert("timepoint", timepoint);
            println!("Resuming stream from timepoint: {:?}", timepoint);
            info!("Resuming stream from timepoint: {:?}", timepoint)
        }

        let mut headers = header::HeaderMap::new();
        headers.insert(
            "Authorization",
            header::HeaderValue::from_str(&format!("{}", API_KEY.as_str()))?,
        );

        Ok(self
            .client
            .get(url)
            .headers(headers)
            .query(&params)
            .send()
            .await?
            .bytes_stream())
    }
}
