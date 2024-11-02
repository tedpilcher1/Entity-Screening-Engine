use sqlx::{postgres::{PgConnectOptions, PgSslMode}, Connection, PgConnection, Pool};
use uuid::Uuid;

// pub async fn init_db(pool: &Pool<Postgres>) -> Result<()> {
pub async fn init_db(conn: &mut PgConnection) -> Result<(), failure::Error> {

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS company (
            id UUID PRIMARY KEY UNIQUE,
            company_house_id TEXT NOT NULL
        )
        "#
    )
    .execute(&mut *conn)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS shareholder (
            parent_id UUID  NOT NULL,
            child_id UUID  NOT NULL,
            PRIMARY KEY(parent_id, child_id)
        )
        "#
    )
    .execute(conn)
    .await?;

    Ok(())
}


pub fn store_company(company_house_number: &String) -> Uuid {

    // generate UUID

    // store in db
    
    todo!()
}