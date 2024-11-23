use chrono::NaiveDateTime;
use dotenv::dotenv;

use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use log::warn;
use pulsar::producer;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use Company_Investigation::{
    jobs::{Job, Officers, RecursiveShareholders}, model::RelationshipKind, postgres::Database, postgres_types::Entity, pulsar::PulsarClient, schema::entity
};

type OfficerDepth = i32;
type ShareholderDepth = i32;
const MAX_DEPTH: i32 = 5;

fn get_entity_response(check_id: Uuid) -> Result<EntityResponse, failure::Error> {

    let mut database = Database::connect()?;
    let mut entities: Vec<EntityWithRelations> = vec![];
    let check = database.get_check(check_id)?;
    let check_entities = database.get_entities(check_id)?;

    for entity in check_entities {
        let officers = database.get_relations(entity.id, RelationshipKind::Officer)?;
        let shareholders = database.get_relations(entity.id, RelationshipKind::Shareholder)?;
        entities.push(EntityWithRelations {
            entity,
            officers,
            shareholders,
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
    officers: Vec<Entity>,
    shareholders: Vec<Entity>,
}

#[derive(Serialize, Deserialize)]
struct EntityResponse {
    entities: Vec<EntityWithRelations>,
    started_at: NaiveDateTime,
    completed_at: Option<NaiveDateTime>, 
}

#[get("/get_relations")]
async fn get_relations_endpoint(params: web::Query<Uuid>) -> impl Responder {
    let check_id = params.into_inner();
    match get_entity_response(check_id) {
        Ok(entity_response) => HttpResponse::Ok().json(entity_response),
        Err(e) => {
            warn!("Failed to get relations: {}", e);
            HttpResponse::InternalServerError().json(format!("Failed to get relations for check {}", check_id))
        }
    }
}

async fn start_relations_check(company_house_number: String, officer_depth: OfficerDepth, shareholder_depth: ShareholderDepth) -> Result<Uuid, failure::Error> {

    let mut database = Database::connect()?;
    let pulsar_client = PulsarClient::new().await;
    let mut producer = pulsar_client.create_producer().await;
    
    let check_id = database.insert_check()?;    
    let entity_id = database.insert_entity(&Entity::create_root(), check_id)?;

    if officer_depth > 0 {
        producer.produce_message(Job::Officers(Officers {
            entity_id,
            check_id,
            company_house_number: company_house_number.clone(),
        })).await?;
    }

    if shareholder_depth > 0 {
        producer.produce_message(Job::RecursiveShareholders(RecursiveShareholders {
            parent_id: entity_id,
            check_id,
            parent_company_number: company_house_number,
            remaining_depth: shareholder_depth,
            get_officers: officer_depth > 0,
        })).await?;
    }


    Ok(check_id)
}

#[post("/start_relations_check")]
async fn start_relations_check_endpoint(params: web::Query<(String, OfficerDepth, ShareholderDepth)>) -> impl Responder {
    let (company_house_number, officer_depth, shareholder_depth) = params.into_inner();

   match start_relations_check(company_house_number.clone(), officer_depth, shareholder_depth).await {
        Ok(check_id) => HttpResponse::Ok().json(check_id),
        Err(e) => {
            warn!("Failed to get relations: {}", e);
            HttpResponse::InternalServerError().json(format!("Failed to start relation check for entity with number {}", company_house_number))
        }
   }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    HttpServer::new(|| {
        App::new()
            .service(get_relations_endpoint)
            .service(start_relations_check_endpoint)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
