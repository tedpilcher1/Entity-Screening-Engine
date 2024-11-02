use sqlx::postgres::PgConnection;
use uuid::Uuid;
use sqlx::query;


pub async fn init_db(conn: &mut PgConnection) -> Result<(), failure::Error> {

    // TODO: refactor this into single transaction
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