use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct FlagSearchResponse {
    pub limit: u32,
    pub offset: u32,
    pub total: Total,
    pub results: Vec<Entity>,
    pub facets: Facets,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Total {
    pub value: u32,
    pub relation: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Entity {
    pub id: String,
    pub caption: String,
    pub schema: String,
    pub properties: HashMap<String, Vec<String>>,
    pub datasets: Vec<String>,
    pub referents: Vec<String>,
    pub target: bool,
    pub first_seen: String,
    pub last_seen: String,
    pub last_change: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Facets {
    pub topics: FacetCategory,
    pub datasets: FacetCategory,
    pub countries: FacetCategory,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FacetCategory {
    pub label: String,
    pub values: Vec<FacetValue>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FacetValue {
    pub name: String,
    pub label: String,
    pub count: u32,
}
