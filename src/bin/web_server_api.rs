use std::{cmp::min, collections::HashSet};

use actix_cors::Cors;
use chrono::{NaiveDate, NaiveDateTime};
use dotenv::dotenv;

use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use log::warn;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use Company_Investigation::{
    jobs::{
        jobs::JobKind,
        relation_jobs::{RelationJob, RelationJobKind},
    },
    models::{Checkkind, Entity, Flagkind, Relationshipkind},
    postgres::Database,
    pulsar::PulsarClient,
    workers::entity_relation_worker::ENTITY_RELATION_TOPIC,
};

const MAX_DEPTH: usize = 3;

#[derive(Serialize, Deserialize)]
struct CheckInfo {
    check_id: Uuid,
    entity_number: String,
    name: Option<String>,
    instructed_on: NaiveDateTime,
    completed_on: Option<NaiveDateTime>,
    risk_level: String, // TODO: change when implemented
    distinct_flags: Vec<Flagkind>,
}

#[derive(Serialize, Deserialize)]
struct ChecksResponse {
    checks: Vec<CheckInfo>,
    has_error: bool,
}

#[derive(Deserialize)]
struct StartRelationsCheckParams {
    relations_depth: Option<usize>,
}

#[derive(Serialize, Deserialize)]
pub struct Relation {
    entity_id: Uuid,
    started_on: Option<NaiveDate>,
    ended_on: Option<NaiveDate>,
}

#[derive(Serialize, Deserialize)]
struct EntityWithRelations {
    entity: Entity,
    officers: Vec<Relation>,
    shareholders: Vec<Relation>,
    flags: Vec<Flagkind>,
    positions: Vec<String>,
    datasets: Vec<String>,
}

#[derive(Serialize, Deserialize)]
struct EntityCheckResponse {
    entities: Vec<EntityWithRelations>,
    started_at: NaiveDateTime,
    completed_at: Option<NaiveDateTime>,
}

fn get_entity_response(check_id: Uuid) -> Result<EntityCheckResponse, failure::Error> {
    let mut database = Database::connect()?;
    let mut entities: Vec<EntityWithRelations> = vec![];
    let check = database.get_check(check_id)?;
    let check_entities = database.get_entities(check_id)?;

    for entity in check_entities {
        let officers = database.get_relations(entity.id, Relationshipkind::Officer)?;
        let shareholders = database.get_relations(entity.id, Relationshipkind::Shareholder)?;

        let flags = database.get_flag_kinds_for_entity(&entity.id)?;
        let positions = database.get_positions(&entity.id)?;
        let datasets = database.get_positions(&entity.id)?;

        entities.push(EntityWithRelations {
            entity,
            officers: officers
                .into_iter()
                .map(|officer| Relation {
                    entity_id: officer.0,
                    started_on: officer.1,
                    ended_on: officer.2,
                })
                .collect(),
            shareholders: shareholders
                .into_iter()
                .map(|shareholder| Relation {
                    entity_id: shareholder.0,
                    started_on: shareholder.1,
                    ended_on: shareholder.2,
                })
                .collect(),
            flags,
            positions,
            datasets,
        })
    }

    Ok(EntityCheckResponse {
        entities,
        started_at: check.started_at,
        completed_at: database.check_completed_at(check_id)?,
    })
}

async fn start_check(company_house_number: String, depth: usize) -> Result<Uuid, failure::Error> {
    let mut database = Database::connect()?;
    let pulsar_client = PulsarClient::new().await;
    let mut producer = pulsar_client
        .create_producer(ENTITY_RELATION_TOPIC, None, None)
        .await;
    let company_house_number = format!("{:0>8}", company_house_number);

    let check_id = database.insert_check(Checkkind::EntityRelation)?;
    let entity_id =
        database.insert_entity(&Entity::create_root(company_house_number.clone()), check_id)?;

    let validated_depth = min(depth, MAX_DEPTH);

    if validated_depth > 0 {
        producer
            .enqueue_job(
                &mut database,
                Some(check_id),
                JobKind::RelationJob(RelationJob {
                    child_id: entity_id,
                    check_id,
                    company_house_number: company_house_number.clone(),
                    officer_id: None,
                    remaining_depth: validated_depth,
                    relation_job_kind: RelationJobKind::Officers,
                }),
            )
            .await?;

        producer
            .enqueue_job(
                &mut database,
                Some(check_id),
                JobKind::RelationJob(RelationJob {
                    child_id: entity_id,
                    check_id,
                    company_house_number,
                    officer_id: None,
                    remaining_depth: validated_depth,
                    relation_job_kind: RelationJobKind::Officers,
                }),
            )
            .await?;
    }

    Ok(check_id)
}

fn get_distinct_flags(
    database: &mut Database,
    check_id: &Uuid,
) -> Result<Vec<Flagkind>, failure::Error> {
    let mut flags = database.get_flag_kinds_for_check(check_id)?;
    let flag_set: HashSet<_> = flags.drain(..).collect();
    flags.extend(flag_set.into_iter());

    Ok(flags)
}

fn get_checks() -> Result<ChecksResponse, failure::Error> {
    let mut database = Database::connect().expect("Should be able to connect to db");
    let checks = database.get_checks()?;
    let mut check_response_vec: Vec<CheckInfo> = Vec::new();
    let mut has_error = false;

    for check in checks {
        let root_entity = database.get_root_entity(&check.id)?;
        let check_response = CheckInfo {
            check_id: check.id,
            entity_number: root_entity.company_house_number,
            name: root_entity.name,
            instructed_on: check.started_at,
            completed_on: database.check_completed_at(check.id)?,
            risk_level: "Low".to_string(), // TODO: update once risk implemented
            distinct_flags: get_distinct_flags(&mut database, &check.id)?,
        };
        check_response_vec.push(check_response);

        if database.does_check_have_errored_job(&check.id)? {
            has_error = true
        }
    }

    Ok(ChecksResponse {
        checks: check_response_vec,
        has_error,
    })
}

#[post("/start_check/{company_house_number}")]
async fn start_check_endpoint(
    path: web::Path<String>,
    info: Option<web::Query<StartRelationsCheckParams>>,
) -> impl Responder {
    let company_house_number = path.into_inner();

    let depth = match info {
        Some(info) => (info.relations_depth.unwrap_or_else(|| 0),),
        None => (0,),
    };

    match start_check(company_house_number.clone(), depth.0).await {
        Ok(check_id) => HttpResponse::Ok().json(check_id),
        Err(e) => {
            warn!("Failed to get relations: {}", e);
            HttpResponse::InternalServerError().json(format!(
                "Failed to start relation check for entity with number {}",
                company_house_number
            ))
        }
    }
}

#[get("/get_check/{check_id}")]
async fn get_check_endpoint(params: web::Path<Uuid>) -> impl Responder {
    let check_id = params.into_inner();
    match get_entity_response(check_id) {
        Ok(entity_response) => HttpResponse::Ok().json(entity_response),
        Err(e) => {
            warn!("Failed to get relations: {}", e);
            HttpResponse::InternalServerError()
                .json(format!("Failed to get relations for check {}", check_id))
        }
    }
}

#[get("/get_checks")]
async fn get_checks_endpoint() -> impl Responder {
    match get_checks() {
        Ok(checks_response) => HttpResponse::Ok().json(checks_response),
        Err(e) => {
            warn!("Failed to get checks: {}", e);
            HttpResponse::InternalServerError().json(format!("Failed to get checks"))
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init();

    HttpServer::new(|| {
        let cors = Cors::default().allow_any_origin().send_wildcard();

        App::new()
            .wrap(cors)
            // .app_data(web::Data::new(Database::connect()))
            // .app_data(web::Data::new(PulsarClient::new()))
            .service(start_check_endpoint)
            .service(get_check_endpoint)
            .service(get_checks_endpoint)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
