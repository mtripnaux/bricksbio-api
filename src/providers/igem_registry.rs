use crate::providers::Provider;
use crate::types::{Biobrick, MetaBiobrick, MetaProvider, Author};
use crate::ontology::multiple_type_inference;
use serde::Deserialize;

pub struct IgemApiProvider;

fn slugify(id: &str) -> String {
    id.trim().to_lowercase().replace('_', "-")
}

impl Provider for IgemApiProvider {
    fn name(&self) -> &'static str {
        "iGEM Registry"
    }

    fn link(&self, id: &str) -> String {
        format!("https://registry.igem.org/parts/{}", slugify(id))
    }

    fn url(&self, id: &str) -> String {
        format!("https://api.registry.igem.org/v1/parts/slugs/{}", slugify(id))
    }

    fn parse(&self, id: &str, json_text: &str) -> Option<Biobrick> {
        let api_part: ApiPart = serde_json::from_str(json_text).ok()?;
        let sequence = api_part.sequence.clone().unwrap_or_default().to_lowercase();
        if sequence.is_empty() { return None; }
        let features = vec![]; // API v1 ne fournit pas les features détaillées
        let authors = api_part.authors.unwrap_or_default().into_iter().map(|a| Author {
            name: a,
            orcid: None,
            email: None,
            affiliation: None,
        }).collect();
        Some(Biobrick {
            metadata: MetaBiobrick {
                id: id.to_string(),
                name: api_part.title.clone().unwrap_or_else(|| id.to_string()),
                r#type: multiple_type_inference(&[api_part.part_type.clone().unwrap_or_default()]).into(),
                circular: false,
                size: sequence.len() as i32,
                providers: vec![MetaProvider {
                    name: self.name().to_string(),
                    link: self.link(id),
                }],
                description: api_part.short_description.unwrap_or_default(),
                features,
            },
            sequence,
            authors,
        })
    }
}

#[derive(Deserialize)]
struct ApiPart {
    title: Option<String>,
    sequence: Option<String>,
    short_description: Option<String>,
    part_type: Option<String>,
    authors: Option<Vec<String>>,
}