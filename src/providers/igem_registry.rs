use async_trait::async_trait;
use crate::types::{Biobrick, MetaBiobrick, MetaProvider, Author};
use crate::ontology::multiple_type_inference;
use serde::Deserialize;

pub struct IgemApiProvider;

fn slugify(id: &str) -> String {
    id.trim().to_lowercase().replace('_', "-")
}

#[async_trait]
impl super::ProviderEnumTrait for IgemApiProvider {
    fn name(&self) -> &'static str {
        "iGEM Registry"
    }

    fn link(&self, id: &str) -> String {
        format!("https://registry.igem.org/parts/{}", slugify(id))
    }

    fn url(&self, id: &str) -> String {
        format!("https://api.registry.igem.org/v1/parts/slugs/{}", slugify(id))
    }

    async fn parse(&self, id: &str, json_text: &str) -> Option<Biobrick> {
        let api_part: ApiPart = serde_json::from_str(json_text).ok()?;
        let sequence = api_part.sequence.clone().unwrap_or_default().to_lowercase();
        if sequence.is_empty() { return None; }
        let features = vec![];
        let authors = api_part.authors.unwrap_or_default().into_iter().map(|a| Author {
            name: a,
            role: None,
        }).collect();

        let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
        let creation = api_part.audit.as_ref()
            .and_then(|audit| audit.created.clone())
            .unwrap_or_else(|| chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true));
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
                    date: now.clone(),
                }],
                description: api_part.short_description.unwrap_or_default(),
                authors,
                creation,
            },
            sequence,
            features,
        })
    }
}

#[derive(Deserialize)]
struct ApiAudit {
    created: Option<String>,
}

#[derive(Deserialize)]
struct ApiPart {
    title: Option<String>,
    sequence: Option<String>,
    short_description: Option<String>,
    part_type: Option<String>,
    authors: Option<Vec<String>>,
    audit: Option<ApiAudit>,
}