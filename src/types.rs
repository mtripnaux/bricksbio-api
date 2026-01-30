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
    pub strand: i32, // 1 | 2
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
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(crate = "serde")]
pub struct MetaBiobrick {
    pub id: String,
    pub name: String,
    pub description: String,
    pub authors: Vec<Author>,
    pub size: i32,
    pub circular: bool,
    pub r#type: crate::ontology::OntologyEntrySerializable,
    pub providers: Vec<MetaProvider>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Biobrick {
    pub metadata: MetaBiobrick,
    pub sequence: String,
    pub features: Vec<MetaFeature>,
}