use std::io::Read;

use bytes::Bytes;
use futures::{future, io::BufReader, sink::Buffer, StreamExt};
use reqwest::Error;
use tokio_util::io::StreamReader;

use crate::{
    company_house::{
        company_house_streaming_client::CompanyHouseStreamingClient,
        company_house_streaming_types::CompanyStreamingResponse,
    },
    postgres::Database,
    pulsar::{PulsarClient, PulsarProducer},
};

pub const COMPANY_STREAMING_TOPIC: &str = "non-persistent://public/default/company-streaming";
pub const OFFICER_STREAMING_TOPIC: &str = "non-persistent://public/default/officer-streaming";
pub const SHAREHOLDER_STREAMING_TOPIC: &str =
    "non-persistent://public/default/shareholder-streaming";

pub struct StreamingWorker {
    database: Database,
    update_event_producer: PulsarProducer,
    streaming_client: CompanyHouseStreamingClient,
    kind: StreamingWorkerKind,
}

pub enum StreamingWorkerKind {
    Company,
    Officer,
    Shareholder,
}

impl StreamingWorker {
    pub async fn new(kind: StreamingWorkerKind) -> Result<Self, failure::Error> {
        let pulsar_client = PulsarClient::new().await;

        let topic = match kind {
            StreamingWorkerKind::Company => COMPANY_STREAMING_TOPIC,
            StreamingWorkerKind::Officer => OFFICER_STREAMING_TOPIC,
            StreamingWorkerKind::Shareholder => SHAREHOLDER_STREAMING_TOPIC,
        };

        Ok(Self {
            database: Database::connect()?,
            update_event_producer: pulsar_client.create_producer(topic, None, None).await,
            streaming_client: CompanyHouseStreamingClient::new(),
            kind,
        })
    }

    fn process_chunk(&self, buffer: &Vec<&[u8]>, worker_kind: &StreamingWorkerKind) -> Result<(), failure::Error> {

        let chunk = buffer.concat();
        
        match worker_kind {
            StreamingWorkerKind::Company => {
                let company_streaming_response: CompanyStreamingResponse =
                serde_json::from_slice(&chunk)?;
                // println!("{:?}", company_streaming_response)
            }
            StreamingWorkerKind::Officer => unimplemented!(),
            StreamingWorkerKind::Shareholder => unimplemented!(),
        }

        Ok(())
    }

    fn process_bytes(&self, bytes: String, buffer: &mut Vec<&str>) -> Result<(), failure::Error> {

        let split_bytes: Vec<&str> = bytes.split_inclusive('\n').collect();

        for bytes in split_bytes {
            // buffer.push(bytes);
            if bytes.starts_with("b") {
                println!("Starts with b: {:?}", bytes)
            }
        }

        Ok(())
    }

    // TODO: handle reconnection when disconnected
    pub async fn do_work(&self) -> Result<(), failure::Error> {
        let mut stream = match self.kind {
            StreamingWorkerKind::Company => {
                self.streaming_client.connect_to_company_stream().await?
            }
            StreamingWorkerKind::Officer => unimplemented!(),
            StreamingWorkerKind::Shareholder => unimplemented!(),
        };

        let mut buffer: Vec<Vec<u8>> = Vec::new();
        while let Some(bytes_result) = stream.next().await {
            if let Ok(bytes) = bytes_result {
                let chunks: Vec<&[u8]> = bytes.split_inclusive(|byte| byte == &b'\n').collect();
                for chunk in chunks {

                    // skip heartbeat 
                    if chunk == &[10] {
                        println!("Skipping heartbeat");
                        continue;
                    }

                    let owned_chunk = chunk.to_owned();
                    buffer.push(owned_chunk);
                    if chunk.ends_with(&[10]) {
                        let completed_chunk = buffer.concat();
                        match serde_json::from_slice::<CompanyStreamingResponse>(&completed_chunk) {
                            Ok(company_streaming_response) => {
                                println!("{:?}", company_streaming_response)
                            },
                            Err(e) => {
                                println!("Failed to convert chunk into response, error: {:?}", e);
                            },
                        }
                        buffer.clear();
                    }
                }
            }
        }

        Ok(())
    }
}
