use pulsar::{producer, DeserializeMessage, Error as PulsarError, SerializeMessage};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub enum Job {
    TestJob(TestJob)
}

impl SerializeMessage for Job {
    fn serialize_message(input: Self) -> Result<producer::Message, PulsarError> {
       let payload = serde_json::to_vec(&input).map_err(|e| PulsarError::Custom(e.to_string()))?;
       Ok(producer::Message {
            payload,
            ..Default::default()
       })
    }
}

impl DeserializeMessage for Job {
    type Output = Result<Job, serde_json::Error>;
    
    fn deserialize_message(payload: &pulsar::Payload) -> Self::Output {
        serde_json::from_slice(&payload.data)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TestJob {
    id: Uuid,
}

impl TestJob {
    pub fn do_work() -> Result<(), failure::Error>{
        todo!()
    }
}