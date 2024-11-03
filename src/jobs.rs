use sqlx::PgConnection;
use uuid::Uuid;

use crate::postgres::insert_company;

pub async fn process_search_request(conn: &mut PgConnection, company_house_id: &String){

    // insert company into company table with uuid
    let company_id = insert_company(conn, company_house_id).await.unwrap();

    // produce message containing company_id and company_house_id
    
}

pub fn get_shareholders(conn: &mut PgConnection, company_id: Uuid, company_house_id: &String) {

    // get shareholers using company_house_id and company house api

    // for each shareholder

        // if shareholder is company, store in company db

        // if max depth not reached, produce message containing both company ids
}
