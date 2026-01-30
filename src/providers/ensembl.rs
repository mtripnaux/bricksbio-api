use crate::providers::ProviderEnumTrait;
use crate::types::Biobrick;
use crate::parsers::genbank::{parse_genbank_raw, genbank_to_biobrick};
use async_trait::async_trait;

pub struct EnsemblProvider;

#[async_trait]
impl ProviderEnumTrait for EnsemblProvider {
    fn name(&self) -> &'static str {
        "Ensembl"
    }

    fn link(&self, id: &str) -> String {
        format!("https://www.ensembl.org/Homo_sapiens/Gene/Summary?g={}", id)
    }

    fn url(&self, id: &str) -> String {
        format!("https://www.ensembl.org/Homo_sapiens/Export/Output/Gene?db=core;flank3_display=0;flank5_display=0;g={id};output=genbank;_format=Text", id = id)
    }

    async fn parse(&self, id: &str, _text: &str) -> Option<Biobrick> {
        let url = self.url(id);
        let client = reqwest::Client::new();
        let resp = client.get(&url)
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .send().await.ok()?;
        let status = resp.status();
        if !status.is_success() {
            println!("    Ensembl HTTP error: {}", status);
            return None;
        }
        let text = resp.text().await.ok()?;
        if text.contains("Error") || text.contains("not found") {
            println!("    Ensembl error: {}", text.chars().take(100).collect::<String>());
            return None;
        }
        parse_genbank_raw(&text)
            .map(|gb_data| {
                let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
                genbank_to_biobrick(id, self.name(), &self.link(id), gb_data, now)
            })
    }
}
