use crate::providers::Provider;
use crate::types::Biobrick;
use crate::parsers::genbank::{parse_genbank_raw, genbank_to_biobrick};

pub struct NcbiProvider;

impl Provider for NcbiProvider {
    fn name(&self) -> &'static str {
        "NCBI"
    }

    fn link(&self, id: &str) -> String {
        format!("https://www.ncbi.nlm.nih.gov/nuccore/{}", id)
    }
    
    fn url(&self, id: &str) -> String {
        format!("https://eutils.ncbi.nlm.nih.gov/entrez/eutils/efetch.fcgi?db=nuccore&id={}&rettype=gb&retmode=text", id)
    }
    
    fn parse(&self, id: &str, text: &str) -> Option<Biobrick> {
        if text.contains("Error:") || text.contains("Failed") {
            println!("    NCBI error: {}", text.chars().take(100).collect::<String>());
            return None;
        }
        
        parse_genbank_raw(text)
            .map(|gb_data| {
                let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
                genbank_to_biobrick(id, self.name(), &self.link(id), gb_data, now)
            })
    }
}
