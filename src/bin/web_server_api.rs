use std::cmp::min;

use chrono::NaiveDateTime;
use dotenv::dotenv;

use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use log::warn;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use Company_Investigation::{
    jobs::{Job, JobKind, Officers, Shareholders},
    models::{Entity, Relationshipkind},
    postgres::Database,
    pulsar::PulsarClient,
};

type OfficerDepth = usize;
type ShareholderDepth = usize;
const MAX_DEPTH: usize = 5;

fn get_entity_response(check_id: Uuid) -> Result<EntityResponse, failure::Error> {
    let mut database = Database::connect()?;
    let mut entities: Vec<EntityWithRelations> = vec![];
    let check = database.get_check(check_id)?;
    let check_entities = database.get_entities(check_id)?;

    for entity in check_entities {
        let officers = database.get_relations(entity.id, Relationshipkind::Officer)?;
        let shareholders = database.get_relations(entity.id, Relationshipkind::Shareholder)?;
        entities.push(EntityWithRelations {
            entity,
            officers: officers.into_iter().map(|officer| officer.id).collect(),
            shareholders: shareholders.into_iter().map(|shareholder| shareholder.id).collect(),
        });
    }

    Ok(EntityResponse {
        entities,
        started_at: check.started_at,
        completed_at: None, // TODO once check handler/tracker service implemented
    })
}

#[derive(Serialize, Deserialize)]
struct EntityWithRelations {
    entity: Entity,
    officers: Vec<Uuid>,
    shareholders: Vec<Uuid>,
}

#[derive(Serialize, Deserialize)]
struct EntityResponse {
    entities: Vec<EntityWithRelations>,
    started_at: NaiveDateTime,
    completed_at: Option<NaiveDateTime>,
}

#[get("/get_relations/{check_id}")]
async fn get_relations_endpoint(params: web::Path<Uuid>) -> impl Responder {
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

async fn start_relations_check(
    company_house_number: String,
    officer_depth: OfficerDepth,
    shareholder_depth: ShareholderDepth,
) -> Result<Uuid, failure::Error> {
    let mut database = Database::connect()?;
    let pulsar_client = PulsarClient::new().await;
    let mut producer = pulsar_client.create_producer().await;
    let company_house_number = format!("{:0>8}", company_house_number);

    let check_id = database.insert_check()?;
    let entity_id = database.insert_entity(&Entity::create_root(company_house_number.clone()), check_id)?;

    let validated_officer_depth = min(officer_depth, MAX_DEPTH);
    let validated_shareholder_depth = min(shareholder_depth, MAX_DEPTH);

    if validated_officer_depth > 0 {
        producer
            .enqueue_job(&mut database, check_id, JobKind::Officers(Officers {
                entity_id,
                check_id,
                company_house_number: company_house_number.clone(),
                remaining_officers_depth: validated_officer_depth,
                remaining_shareholder_depth: validated_shareholder_depth,
            }))
            .await?;
    }

    if validated_shareholder_depth > 0 {
        producer
            .enqueue_job(&mut database, check_id, JobKind::Shareholders(Shareholders {
                parent_id: entity_id,
                check_id,
                parent_company_number: company_house_number,
                remaining_shareholder_depth: validated_shareholder_depth,
                remaining_officers_depth: validated_officer_depth,
            }))
            .await?;
    }

    Ok(check_id)
}

#[derive(Deserialize)]
struct StartRelationsCheckParams {
    officer_depth: Option<usize>,
    shareholder_depth: Option<usize>,
}

#[post("/start_relations_check/{company_house_number}")]
async fn start_relations_check_endpoint(
    path: web::Path<String>,
    info: Option<web::Query<StartRelationsCheckParams>>,
) -> impl Responder {
    let company_house_number = path.into_inner();

    let (officer_depth, shareholder_depth) = match info {
        Some(info) => {
            (info.officer_depth.unwrap_or_else(|| 1), info.shareholder_depth.unwrap_or_else(|| 1))
        }
        None => (1, 1),
    };

    match start_relations_check(
        company_house_number.clone(),
        officer_depth,
        shareholder_depth,
    )
    .await
    {
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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init();

    HttpServer::new(|| {
        App::new()
            // .app_data(web::Data::new(Database::connect()))
            // .app_data(web::Data::new(PulsarClient::new()))
            .service(start_relations_check_endpoint)
            .service(get_relations_endpoint)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
