use serde::{Deserialize, Serialize};
use sqlx::postgres::PgConnection;
use sqlx::{query, Connection, FromRow};
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

        query(
            r#"
            CREATE TABLE IF NOT EXISTS entity (
                id UUID PRIMARY KEY UNIQUE NOT NULL,
                company_house_id TEXT NOT NULL,
                name TEXT,
                kind TEXT,
                country TEXT,
                postal_code TEXT,
                date_of_origin TEXT,
                is_root BOOLEAN NOT NULL
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

        query(
            r#"
            CREATE TABLE IF NOT EXISTS officer (
                id UUID PRIMARY KEY UNIQUE NOT NULL, 
                company_id UUID NOT NULL,
                entity_id UUID NOT NULL,
                officer_role TEXT
            )
            "#,
        )
        .execute(&mut *transaction)
        .await?;

        // Commit the transaction if both statements succeed
        transaction.commit().await?;

        Ok(())
    }

    pub async fn insert_root_entity(
        &mut self,
        company_house_id: &String,
    ) -> Result<Uuid, failure::Error> {
        self.insert_entity_internal(company_house_id, None, None, None, None, None, true)
            .await
    }

    pub async fn insert_entity(
        &mut self,
        company_house_id: &String,
        name: Option<String>,
        kind: Option<String>,
        country: Option<String>,
        postal_code: Option<String>,
        date_of_origin: Option<String>,
    ) -> Result<Uuid, failure::Error> {
        self.insert_entity_internal(
            company_house_id,
            name,
            kind,
            country,
            postal_code,
            date_of_origin,
            false,
        )
        .await
    }

    async fn insert_entity_internal(
        &mut self,
        company_house_id: &String,
        name: Option<String>,
        kind: Option<String>,
        country: Option<String>,
        postal_code: Option<String>,
        date_of_origin: Option<String>,
        is_root: bool,
    ) -> Result<Uuid, failure::Error> {
        let id: Uuid = Uuid::new_v4();

        query("INSERT INTO entity (id, company_house_id, name, kind, country, postal_code, date_of_origin, is_root) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)")
            .bind(id)
            .bind(company_house_id)
            .bind(name)
            .bind(kind)
            .bind(country)
            .bind(postal_code)
            .bind(date_of_origin)
            .bind(is_root)
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

    pub async fn get_shareholders(
        &mut self,
        root_entity_id: &Uuid,
    ) -> Result<Vec<EntityDetails>, failure::Error> {
        let query = r#"
            SELECT 
                c.id AS entity_id,
                c.company_house_id AS company_house_id,
                c.name AS entity_name,
                c.kind as entity_kind,
                c.country as entity_country,
                c.postal_code as entity_postal_code,
                c.date_of_origin as entity_date_of_origin
            FROM 
                shareholder sh
            INNER JOIN 
                entity c ON sh.child_id = c.id
            WHERE 
                sh.parent_id = $1;
        "#;

        let shareholders: Vec<EntityDetails> = sqlx::query_as::<_, EntityDetails>(query)
            .bind(root_entity_id)
            .fetch_all(&mut self.conn)
            .await?;

        Ok(shareholders)
    }

    pub async fn insert_officer(
        &mut self,
        company_id: Uuid,
        entity_id: Uuid,
        officer_role: Option<String>,
    ) -> Result<(), failure::Error> {
        query(
            "INSERT INTO officer (id, company_id, entity_id, officer_role) VALUES ($1, $2, $3, $4)",
        )
        .bind(Uuid::new_v4())
        .bind(company_id)
        .bind(entity_id)
        .bind(officer_role)
        .execute(&mut self.conn)
        .await?;

        Ok(())
    }

    pub async fn get_officers(
        &mut self,
        company_id: &Uuid,
    ) -> Result<Vec<EntityDetails>, failure::Error> {
        let query = r#"
            SELECT 
                c.id AS entity_id,
                c.company_house_id AS company_house_id,
                c.name AS entity_name,
                c.kind as entity_kind,
                c.country as entity_country,
                c.postal_code as entity_postal_code,
                c.date_of_origin as entity_date_of_origin
            FROM 
                officer of
            INNER JOIN 
                entity c ON of.entity_id = c.id
            WHERE 
                of.company_id = $1;
        "#;

        let officers: Vec<EntityDetails> = sqlx::query_as::<_, EntityDetails>(query)
            .bind(company_id)
            .fetch_all(&mut self.conn)
            .await?;

        Ok(officers)
    }
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct EntityDetails {
    pub entity_id: Uuid,
    pub company_house_id: String,
    pub entity_name: Option<String>,
    pub entity_kind: Option<String>,
    pub entity_country: Option<String>,
    pub entity_postal_code: Option<String>,
}
