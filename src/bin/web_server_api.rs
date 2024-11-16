use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use uuid::Uuid;


async fn get_shareholders(root_profile_id: web::Path<Uuid>)  -> impl Responder{
    HttpResponse::NoContent().finish()
}

#[derive(Serialize, Deserialize)]
struct CompanyShareholdersResponse {
    id: Uuid,
    company_house_id: String,
    name: Option<String>,
    kind: Option<String>,
    country: Option<String>,
    postal_code: Option<String>,
    shareholders: Vec<CompanyShareholdersResponse>,
}


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/get_shareholders/{root_profile_id}", web::get().to(get_shareholders))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}