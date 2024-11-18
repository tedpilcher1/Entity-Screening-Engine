use std::cmp::min;

use dotenv::dotenv;

use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use Company_Investigation::{
    jobs::{Job, Officers, RecursiveShareholders},
    postgres::{Database, EntityDetails},
    pulsar::PulsarClient,
};

const MAX_DEPTH: i32 = 5;

async fn iteratively_get_shareholders(
    database: &mut Database,
    root_company_id: &Uuid,
) -> Result<Vec<Entity>, failure::Error> {
    let mut result: Vec<Entity> = vec![];
    // Store indices instead of trying to clone vectors
    let mut stack: Vec<(Uuid, usize)> = vec![(root_company_id.clone(), 0)];

    while let Some((current_id, parent_index)) = stack.pop() {
        let entity_details = database.get_shareholders(&current_id).await?;

        for entity in entity_details {
            let entity_id = entity.entity_id;

            let officers_details = database.get_officers(&entity_id).await?;
            let mut officers = vec![];

            for officer in officers_details {
                let new_officer = Entity {
                    entity_details: officer,
                    shareholders: vec![],
                    officers: vec![],
                };
                officers.push(new_officer);
            }

            let new_entity = Entity {
                entity_details: entity,
                shareholders: vec![], // Start with empty shareholders
                officers,
            };

            // Push the company's ID and its index in the result vector
            let current_index = result.len();
            stack.push((entity_id, current_index));

            if parent_index < result.len() {
                // Add this company as a shareholder to its parent
                result[parent_index].shareholders.push(new_entity);
            } else {
                // This is a root level company
                result.push(new_entity);
            }
        }
    }

    Ok(result)
}

async fn get_shareholders(root_company_id: web::Path<Uuid>) -> impl Responder {
    let mut database = Database::connect()
        .await
        .expect("Should be able to connect to db");
    let shareholders = match iteratively_get_shareholders(&mut database, &root_company_id).await {
        Ok(shareholders) => shareholders,
        Err(e) => {
            println!("{:?}", e);
            vec![]
        }
    };

    HttpResponse::Ok().json(CompanyShareholdersResponse {
        root_company_id: *root_company_id,
        shareholders,
    })
}

async fn start_get_shareholders_task(
    database: &mut Database,
    company_house_number: String,
    depth: i32,
    get_officers: bool,
) -> Result<Uuid, failure::Error> {
    let root_profile_id = database.insert_root_entity(&company_house_number).await?;
    let pulsar_client = PulsarClient::new().await;
    let mut producer = pulsar_client.create_producer().await;

    let job = Job::RecursiveShareholders(RecursiveShareholders {
        parent_id: root_profile_id,
        parent_company_id: company_house_number,
        remaining_depth: min(depth, MAX_DEPTH),
        get_officers,
    });

    producer.produce_message(job).await?;

    Ok(root_profile_id)
}

async fn shareholders(params: web::Path<(String, i32, bool)>) -> impl Responder {
    let (company_house_number, depth, get_officers) = (params.0.clone(), params.1, params.2);
    let padded_company_house_number = format!("{:0>8}", company_house_number);

    let mut database: Database = Database::connect()
        .await
        .expect("Should be able to connect to db");
    
    let _check_id = database.insert_check().await.expect("should be able to create check");

    match start_get_shareholders_task(
        &mut database,
        padded_company_house_number,
        depth,
        get_officers,
    )
    .await
    {
        Ok(root_profile_id) => HttpResponse::Ok().json(root_profile_id),
        Err(e) => {
            println!("{:?}", e); // TODO, replace with proper logging
            HttpResponse::InternalServerError().into()
        }
    }
}

#[derive(Serialize, Deserialize)]
struct CompanyShareholdersResponse {
    root_company_id: Uuid,
    shareholders: Vec<Entity>,
}

#[derive(Serialize, Deserialize)]
struct Entity {
    entity_details: EntityDetails,
    shareholders: Vec<Entity>,
    officers: Vec<Entity>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    HttpServer::new(|| {
        App::new()
            .service(
                web::resource("/get_shareholders/{root_profile_id}")
                    .route(web::get().to(get_shareholders)),
            )
            .service(
                web::resource("/shareholders/{company_house_number}/{depth}/{get_officers}")
                    .route(web::post().to(shareholders)),
            )
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
