use sqlx::postgres::PgConnection;
use sqlx::{query, Connection};
use uuid::Uuid;

const DB_URL: &str = "postgres://localhost/postgres";

pub struct Database {
    conn: PgConnection,
}

impl Database {
    pub async fn connect() -> Result<Self, failure::Error> {
        let mut database = Database {
            conn: PgConnection::connect(DB_URL).await.unwrap(),
        };
        database.init_db().await?;
        Ok(database)
    }

    async fn init_db(&mut self) -> Result<(), failure::Error> {
        // TODO: Need to link id as foreign key to both
        // parent_id and child_id in shareholer
        let mut transaction = self.conn.begin().await?;

        // Execute the first CREATE TABLE statement
        query(
            r#"
            CREATE TABLE IF NOT EXISTS company (
                id UUID PRIMARY KEY UNIQUE,
                company_house_id TEXT NOT NULL
            )
            "#,
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
            "#,
        )
        .execute(&mut *transaction)
        .await?;

        // Commit the transaction if both statements succeed
        transaction.commit().await?;

        Ok(())
    }

    pub async fn insert_company(
        &mut self,
        company_house_id: &String,
    ) -> Result<Uuid, failure::Error> {
        let id: Uuid = Uuid::new_v4();

        query("INSERT INTO company (id, company_house_id) VALUES ($1, $2)")
            .bind(id) // Bind the UUID to the first placeholder ($1)
            .bind(company_house_id) // Bind the name to the second placeholder ($2)
            .execute(&mut self.conn)
            .await?;

        Ok(id)
    }

    pub async fn insert_shareholder(
        &mut self,
        parent_id: Uuid,
        child_id: Uuid,
    ) -> Result<(), failure::Error> {
        query("INSERT INTO shareholder (parent_id, child_id) VALUES ($1, $2)")
            .bind(parent_id)
            .bind(child_id)
            .execute(&mut self.conn)
            .await?;

        Ok(())
    }

    pub async fn get_recursive_shareholders(
        &mut self,
        parent_id: Uuid,
        depth: i32,
    ) -> Result<(), failure::Error> {
        todo!()
    }
}
