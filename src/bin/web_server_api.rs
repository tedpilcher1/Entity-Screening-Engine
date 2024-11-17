use std::cmp::min;

use dotenv::dotenv;

use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use Company_Investigation::{
    jobs::{Job, RecursiveShareholders},
    postgres::{CompanyDetails, Database},
    pulsar::PulsarClient,
};

const MAX_DEPTH: i32 = 5;

async fn iteratively_get_shareholders(
    database: &mut Database,
    root_company_id: &Uuid,
) -> Result<Vec<Company>, failure::Error> {
    let mut result: Vec<Company> = vec![];
    // Store indices instead of trying to clone vectors
    let mut stack: Vec<(Uuid, usize)> = vec![(root_company_id.clone(), 0)];

    while let Some((current_id, parent_index)) = stack.pop() {
        let company_details = database.get_shareholders(&current_id).await?;

        for company in company_details {
            let company_id = company.company_id;

            let new_company = Company {
                company_details: company,
                shareholders: vec![], // Start with empty shareholders
            };

            // Push the company's ID and its index in the result vector
            let current_index = result.len();
            stack.push((company_id, current_index));

            if parent_index < result.len() {
                // Add this company as a shareholder to its parent
                result[parent_index].shareholders.push(new_company);
            } else {
                // This is a root level company
                result.push(new_company);
            }
        }
    }

    Ok(result)
}

async fn get_shareholders(root_company_id: web::Path<Uuid>) -> impl Responder {
    let mut database = Database::connect()
        .await
        .expect("Should be able to connect to db");
    let shareholders = iteratively_get_shareholders(&mut database, &root_company_id)
        .await
        .unwrap_or_default();

    HttpResponse::Ok().json(CompanyShareholdersResponse {
        root_company_id: *root_company_id,
        shareholders,
    })
}

async fn start_get_shareholders_task(
    database: &mut Database,
    company_house_number: String,
    depth: i32,
) -> Result<Uuid, failure::Error> {
    let root_profile_id = database.insert_root_company(&company_house_number).await?;
    let pulsar_client = PulsarClient::new().await;
    let mut producer = pulsar_client.create_producer().await;

    let job = Job::RecursiveShareholders(RecursiveShareholders {
        parent_id: root_profile_id,
        parent_company_id: company_house_number,
        remaining_depth: min(depth, MAX_DEPTH),
    });

    producer.produce_message(job).await?;

    Ok(root_profile_id)
}

async fn shareholders(params: web::Path<(String, i32)>) -> impl Responder {
    let (company_house_number, depth) = (params.0.clone(), params.1);
    let padded_company_house_number = format!("{:0>8}", company_house_number);

    let mut database = Database::connect()
        .await
        .expect("Should be able to connect to db");

    match start_get_shareholders_task(&mut database, padded_company_house_number, depth).await {
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
    shareholders: Vec<Company>,
}

#[derive(Serialize, Deserialize)]
struct Company {
    company_details: CompanyDetails,
    shareholders: Vec<Company>,
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
                web::resource("/shareholders/{company_house_number}/{depth}")
                    .route(web::post().to(shareholders)),
            )
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
