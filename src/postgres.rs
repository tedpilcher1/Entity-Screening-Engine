use sqlx::postgres::PgConnection;
use uuid::Uuid;
use sqlx::{query, Acquire};


pub async fn init_db(conn: &mut PgConnection) -> Result<(), failure::Error> {
    
    // TODO: Need to link id as foreign key to both
    // parent_id and child_id in shareholer
    let mut transaction = conn.begin().await?;

    // Execute the first CREATE TABLE statement
    query(
        r#"
        CREATE TABLE IF NOT EXISTS company (
            id UUID PRIMARY KEY UNIQUE,
            company_house_id TEXT NOT NULL
        )
        "#
    )
    .execute(&mut *transaction)
    .await?;

    // Execute the second CREATE TABLE statement
    query(
        r#"
        CREATE TABLE IF NOT EXISTS shareholder (
            parent_id UUID NOT NULL,
            child_id UUID NOT NULL,
            PRIMARY KEY(parent_id, child_id)
        )
        "#
    )
    .execute(&mut *transaction)
    .await?;

    // Commit the transaction if both statements succeed
    transaction.commit().await?;

    Ok(())
}


pub async fn insert_company(conn: &mut PgConnection, company_house_id: &String) -> Result<Uuid, failure::Error> {

    let id: Uuid = Uuid::new_v4();

    query(
        "INSERT INTO company (id, company_house_id) VALUES ($1, $2)"
    )
    .bind(id) // Bind the UUID to the first placeholder ($1)
    .bind(company_house_id)    // Bind the name to the second placeholder ($2)
    .execute(conn)
    .await?;

    Ok(id)
}

pub async fn insert_shareholder(conn: &mut PgConnection, parent_id: Uuid, child_id: Uuid) -> Result<(), failure::Error> {

    query(
        "INSERT INTO shareholder (parent_id, child_id) VALUES ($1, $2)"
    )
    .bind(parent_id)
    .bind(child_id)
    .execute(conn)
    .await?;

    Ok(())
}