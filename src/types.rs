use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Author {
    pub name: String,
    pub orcid: Option<String>,
    pub email: Option<String>,
    pub affiliation: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Location {
    pub start: i32,
    pub end: i32,
    pub strand: i32, // 1 | 2
    pub forward: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MetaFeature {
    pub id: String,
    pub name: String,
    pub r#type: String, // SBOLType
    pub location: Location,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MetaProvider {
    pub name: String,
    pub link: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MetaBiobrick {
    pub id: String,
    pub name: String,
    pub r#type: String, // SBOLType
    pub circular: bool,
    pub size: i32,
    pub providers: Vec<MetaProvider>,
    pub description: String,
    pub features: Vec<MetaFeature>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Biobrick {
    pub metadata: MetaBiobrick,
    pub sequence: String,
    pub authors: Vec<Author>,
}
