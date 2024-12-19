use std::num::NonZero;

use governor::{
    clock::{QuantaClock, QuantaInstant},
    middleware::NoOpMiddleware,
    state::{InMemoryState, NotKeyed},
    Quota, RateLimiter,
};
use log::info;
use pulsar::{
    consumer::{DeadLetterPolicy, Message},
    producer, proto, Consumer, Producer, Pulsar, SubType, TokioExecutor,
};
use uuid::Uuid;

use crate::{
    jobs::jobs::{Job, JobKind},
    postgres::Database,
};

const PULSAR_ADDR: &str = "pulsar://localhost:6650";
const MAX_JOB_RETRY: usize = 3;

pub struct PulsarClient {
    internal_client: Pulsar<TokioExecutor>,
}

impl PulsarClient {
    pub async fn new() -> Self {
        Self {
            internal_client: Pulsar::builder(PULSAR_ADDR, TokioExecutor)
                .build()
                .await
                .expect("Should be able to create new pulsar client builder"),
        }
    }

    pub async fn create_producer(
        &self,
        topic: &str,
        rate_limit_per_min: Option<u32>,
        max_jobs_per_check: Option<usize>,
    ) -> PulsarProducer {
        let id = Uuid::new_v4();

        // TODO: This will limit each worker's producer to x per min, in reality we want all workers' producers
        // to be limited to x per min, i.e each producer limited to x / n per min, where n is number of workers
        // TODO: This could be done via an env var which describes num of workers
        let rate_limiter = match rate_limit_per_min {
            Some(rate_limit_per_min) => Some(RateLimiter::direct(
                Quota::per_minute(
                    NonZero::new(rate_limit_per_min)
                        .expect("entity relation producer limit should be set"),
                )
                .allow_burst(NonZero::new(1).unwrap()),
            )),
            None => None,
        };

        PulsarProducer {
            id,
            internal_producer: self
                .internal_client
                .producer()
                .with_topic(topic)
                .with_name("PRODUCER_".to_owned() + &id.to_string())
                .with_options(producer::ProducerOptions {
                    schema: Some(proto::Schema {
                        r#type: proto::schema::Type::String as i32, // Or appropriate type for Job
                        ..Default::default()
                    }),

                    ..Default::default()
                })
                .build()
                .await
                .expect("Should be able to create producer"),
            rate_limiter,
            max_jobs_per_check,
        }
    }

    pub async fn create_consumer(
        &self,
        topic: &str,
        subscription_type: SubType,
        subscription: &str,
    ) -> PulsarConsumer {
        let id = Uuid::new_v4();
        PulsarConsumer {
            id,
            internal_consumer: self
                .internal_client
                .consumer()
                .with_topic(topic)
                .with_consumer_name("CONSUMER_".to_owned() + &id.to_string())
                .with_subscription_type(subscription_type) // exclusive for current testing
                // .with_subscription("SUB_".to_owned() + &id.to_string())
                .with_subscription(subscription)
                .with_dead_letter_policy(DeadLetterPolicy {
                    max_redeliver_count: MAX_JOB_RETRY,
                    dead_letter_topic: format!("{}-DLQ", topic),
                })
                .build()
                .await
                .expect("Should be able to create consumer"),
        }
    }
}

pub struct PulsarProducer {
    id: Uuid,
    internal_producer: Producer<TokioExecutor>,
    rate_limiter:
        Option<RateLimiter<NotKeyed, InMemoryState, QuantaClock, NoOpMiddleware<QuantaInstant>>>,
    max_jobs_per_check: Option<usize>,
}

impl PulsarProducer {
    async fn produce_message(&mut self, job: Job) -> Result<(), failure::Error> {
        self.internal_producer.send(job).await?;
        Ok(())
    }

    pub async fn enqueue_job(
        &mut self,
        database: &mut Database,
        check_id: Option<Uuid>,
        job_kind: JobKind,
    ) -> Result<(), failure::Error> {
        // if max jobs per check reached, gracefully terminate
        // TODO: this is a bit inefficient was we are executing sql query each
        // time we enqueue rather than per job, but fine for now
        if let Some(rate_limiter) = &self.rate_limiter {
            rate_limiter.until_ready().await;
        }

        let job_id = match check_id {
            Some(check_id) => {
                if let Some(max_jobs) = self.max_jobs_per_check {
                    if database.get_num_of_jobs(&check_id)? >= max_jobs {
                        println!("Check with ID: {:?} reached job limit", check_id);
                        info!("Check with ID: {:?} reached job limit", check_id);
                        return Ok(());
                    }
                }
                database.add_job_with_check(check_id)?
            }
            None => database.add_job()?,
        };

        self.produce_message(Job {
            id: job_id,
            job_kind,
        })
        .await?;
        Ok(())
    }
}

pub struct PulsarConsumer {
    id: Uuid,
    pub internal_consumer: Consumer<Job, TokioExecutor>,
}

impl PulsarConsumer {
    pub async fn ack(&mut self, msg: &Message<Job>) {
        if let Err(_) = self.internal_consumer.ack(msg).await {
            println!("Failed to ack message with id: {:?}", msg.message_id());
            info!("Failed to ack message with id: {:?}", msg.message_id())
        }
    }

    pub async fn nack(&mut self, msg: &Message<Job>) {
        if let Err(_) = self.internal_consumer.nack(msg).await {
            println!("Failed to nack message with id: {:?}", msg.message_id());
            info!("Failed to nack message with id: {:?}", msg.message_id())
        }
    }
}
