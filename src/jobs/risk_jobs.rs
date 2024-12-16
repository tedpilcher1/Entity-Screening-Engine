use chrono::{DateTime, Duration, NaiveDate, Offset, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    models::{Entity, Entitykind, FlagStringList},
    workers::risk_worker::RiskWorker,
};

const DORMANCY_YEARS: i64 = 5;

#[derive(Serialize, Deserialize, Debug)]
pub struct RiskJob {
    pub scope: RiskJobScope,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum RiskJobScope {
    Global(GlobalRiskJob),
    Local(LocalRiskJob),
}

// Jobs that should be run after a check has completed, and all (intended)
// relations are identified
//
// Considers all entities and relations
#[derive(Serialize, Deserialize, Debug)]
pub enum GlobalRiskJob {
    // Determines if there is a circular relationship between entities
    CircularRelations,
    // When companies are registered/created in bulk within the registration date window
    MassRegistration,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LocalRiskJob {
    pub entity_id: Uuid,
    pub kind: LocalRiskJobKind,
}

// Jobs that can be run immedialty after an entity is recorded
//
// Only consider a single entity without any relations
#[derive(Serialize, Deserialize, Debug)]
pub enum LocalRiskJobKind {
    // Finds flags for entities (only individuals currently), i.e. sanctions
    // Also find datasets and previous positions of entity
    Flags,
    // Determines if individuals are implausibly young or old
    OutlierAge,
    // Determines if a company has been dormant for more than 5 years
    Dormancy,
}

impl RiskJob {
    pub async fn do_job(&self, worker: &mut RiskWorker) -> Result<(), failure::Error> {
        match &self.scope {
            RiskJobScope::Global(global_risk_job) => self.do_global_job(global_risk_job, worker),
            RiskJobScope::Local(local_risk_job) => self.do_local_job(local_risk_job, worker).await,
        }
    }

    fn do_global_job(
        &self,
        job: &GlobalRiskJob,
        worker: &mut RiskWorker,
    ) -> Result<(), failure::Error> {
        match job {
            GlobalRiskJob::CircularRelations => unimplemented!(),
            GlobalRiskJob::MassRegistration => unimplemented!(),
        }

        Ok(())
    }

    async fn do_local_job(
        &self,
        job: &LocalRiskJob,
        worker: &mut RiskWorker,
    ) -> Result<(), failure::Error> {
        let entity = worker.database.get_entity(job.entity_id)?;
        match job.kind {
            LocalRiskJobKind::Flags => self.do_flags_job(entity, worker).await,
            LocalRiskJobKind::OutlierAge => self.do_outlier_age_job(entity, worker),
            LocalRiskJobKind::Dormancy => self.do_dormancy_job(entity, worker).await,
        }
    }

    async fn do_flags_job(
        &self,
        entity: Entity,
        worker: &mut RiskWorker,
    ) -> Result<(), failure::Error> {
        if entity.kind != Entitykind::Individual {
            return Ok(());
        }

        if let Some(name) = entity.name {
            let search_result = worker.open_sanctions_client.get_flags(name).await?;

            if let Some(os_entity) = search_result.results.first() {
                for (key, value) in os_entity.properties.to_owned().into_iter() {
                    if key == "topics" {
                        worker
                            .database
                            .insert_flags(entity.id, FlagStringList(value).into())?
                    } else if key == "position" {
                        worker.database.insert_positions(entity.id, value)?
                    }
                }
                worker
                    .database
                    .insert_datasets(entity.id, os_entity.datasets.to_owned())?;
            }
        }
        Ok(())
    }

    fn do_outlier_age_job(
        &self,
        entity: Entity,
        worker: &mut RiskWorker,
    ) -> Result<(), failure::Error> {
        if entity.kind != Entitykind::Individual {
            return Ok(());
        }

        let mut outlier_age = false;

        if let Some(dob) = entity.date_of_origin {
            let parsed_dob: DateTime<Utc> = dob.parse()?;

            match chrono::Utc::now().years_since(parsed_dob) {
                Some(age) => {
                    if age < 15 || age > 85 {
                        outlier_age = true;
                    }
                }
                None => {}
            }
        }

        worker
            .database
            .insert_outlier_age(&entity.id, outlier_age)?;

        Ok(())
    }

    async fn do_dormancy_job(
        &self,
        entity: Entity,
        worker: &mut RiskWorker,
    ) -> Result<(), failure::Error> {
        // TODO: this should be rate limited as it's using the company house api
        // but as companies are processed less frequently, it can go without for now

        let mut is_dormant = false;
        let filing_history = worker
            .company_house_client
            .get_filing_history(&entity.company_house_number)
            .await?;

        // if last filing over 5 years ago (relative to now), mark company as dormant
        let last_filing = filing_history.items.first();
        if let Some(last_filing) = last_filing {
            if let Some(last_filing_date) = last_filing.date {
                let five_years_ago = Utc::now().date_naive() - Duration::days(DORMANCY_YEARS * 365);
                if last_filing_date < five_years_ago {
                    is_dormant = true;
                }
            }
        }

        worker
            .database
            .insert_dormant_company(&entity.id, is_dormant)?;

        Ok(())
    }
}
