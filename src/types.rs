use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Author {
    pub name: String,
    pub role: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Location {
    pub start: i32,
    pub end: i32,
    pub forward: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MetaFeature {
    pub id: String,
    pub name: String,
    pub r#type: crate::ontology::OntologyEntrySerializable,
    pub location: Location,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MetaProvider {
    pub name: String,
    pub link: String,
    pub date: String, // ISO 8601 format
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(crate = "serde")]
pub struct MetaBiobrick {
    pub id: String,
    pub name: String,
    pub description: String,
    pub creation: String, // ISO 8601 format
    pub size: i32,
    pub circular: bool,
    pub r#type: crate::ontology::OntologyEntrySerializable,
    pub authors: Vec<Author>,
    pub providers: Vec<MetaProvider>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Biobrick {
    pub metadata: MetaBiobrick,
    pub sequence: String,
    pub features: Vec<MetaFeature>,
}

#[derive(Debug, Serialize)]
pub struct CacheStats {
    pub entries: i64,
}

#[derive(Debug, Deserialize)]
pub struct CacheSearchParams {
    pub q: String,
    pub n: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct SearchHit {
    pub r#match: f64,
    pub biobrick: Biobrick,
}

#[derive(Debug, Serialize)]
pub struct SearchResponse {
    pub query: String,
    pub requested: usize,
    pub count: usize,
    pub results: Vec<SearchHit>,
}

#[derive(Debug, Serialize)]
pub struct ApiStats {
    pub unique_ips_today: i64,
    pub unique_ips_total: i64,
    pub requests_today: i64,
    pub requests_total: i64,
    pub unique_ips_last_7_days: i64,
}