use crate::providers::Provider;
use crate::types::Biobrick;
use crate::parsers::genbank::{parse_genbank_raw, genbank_to_biobrick};

pub struct SynBioHubProvider;

impl Provider for SynBioHubProvider {
    fn name(&self) -> &'static str {
        "iGEM via SynBioHub"
    }
    
    fn link(&self, id: &str) -> String {
        format!("https://synbiohub.org/public/igem/{}/1", id)
    }
    
    fn url(&self, id: &str) -> String {
        format!("https://synbiohub.org/public/igem/{}/1/gb", id)
    }
    
    fn parse(&self, id: &str, text: &str) -> Option<Biobrick> {
        parse_genbank_raw(text)
            .map(|gb_data| {
                let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
                genbank_to_biobrick(id, self.name(), &self.link(id), gb_data, now)
            })
    }
}
