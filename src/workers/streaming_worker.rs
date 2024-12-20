use bytes::Bytes;
use futures::StreamExt;
use log::{info, warn};

use crate::{
    company_house::{
        company_house_streaming_client::CompanyHouseStreamingClient,
        company_house_streaming_types::{CompanyStreamingResponse, OfficerStreamingResponse},
    },
    jobs::{
        jobs::JobKind,
        streaming_update_jobs::{StreamingUpdateJob, UpdateKind},
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
    kind: StreamingKind,
}

#[derive(Clone)]
pub enum StreamingKind {
    Company,
    Officer,
    Shareholder,
}

impl StreamingWorker {
    pub async fn new(kind: StreamingKind) -> Result<Self, failure::Error> {
        let pulsar_client = PulsarClient::new().await;

        let topic = match kind {
            StreamingKind::Company => COMPANY_STREAMING_TOPIC,
            StreamingKind::Officer => OFFICER_STREAMING_TOPIC,
            StreamingKind::Shareholder => SHAREHOLDER_STREAMING_TOPIC,
        };

        Ok(Self {
            database: Database::connect()?,
            update_event_producer: pulsar_client.create_producer(topic, None, None).await,
            streaming_client: CompanyHouseStreamingClient::new(kind.clone()),
            kind,
        })
    }

    // TODO: handle reconnection when disconnected
    pub async fn do_work(&mut self) -> Result<(), failure::Error> {
        let timepoint = self
            .database
            .get_last_processed_timepoint((&self.kind).into())?;
        let mut stream = self.streaming_client.connect_to_stream(timepoint).await?;

        let mut buffer: Vec<Vec<u8>> = Vec::new();
        while let Some(bytes_result) = stream.next().await {
            if let Ok(bytes) = bytes_result {
                match self.process_bytes(bytes, &mut buffer).await {
                    Ok(_) => info!("Successfully processed bytes"),
                    Err(e) => {
                        warn!("Failed to process bytes, error: {:?}", e);
                        println!("Failed to process bytes, error: {:?}", e)
                    }
                }
            }
        }

        Ok(())
    }

    async fn process_bytes(
        &mut self,
        bytes: Bytes,
        buffer: &mut Vec<Vec<u8>>,
    ) -> Result<(), failure::Error> {
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
                self.process_chunk(buffer.concat()).await?;
                buffer.clear();
            }
        }
        Ok(())
    }

    async fn process_chunk(&mut self, chunk: Vec<u8>) -> Result<(), failure::Error> {
        let update = match self.kind {
            StreamingKind::Company => {
                let streaming_response: CompanyStreamingResponse = serde_json::from_slice(&chunk)?;

                match (streaming_response.data, streaming_response.event) {
                    (Some(data), Some(event)) => Some((UpdateKind::Company(data), event)),
                    _ => None,
                }
            }
            StreamingKind::Officer => {
                let streaming_response: OfficerStreamingResponse = serde_json::from_slice(&chunk)?;

                match (streaming_response.data, streaming_response.event) {
                    (Some(data), Some(event)) => Some((UpdateKind::Officer(data), event)),
                    _ => None,
                }
            },
            StreamingKind::Shareholder => unimplemented!(),
        };

        if let Some((kind, event)) = update {
            let update_job = JobKind::StreamingUpdateJob(StreamingUpdateJob { event, kind });
            self.update_event_producer
                .enqueue_job(&mut self.database, None, update_job)
                .await?;
        }

        Ok(())
    }
}
