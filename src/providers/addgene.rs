use async_trait::async_trait;
use crate::types::{Biobrick, Author, MetaBiobrick, MetaProvider};
use crate::parsers::genbank::{parse_genbank_raw, genbank_to_biobrick};
use crate::ontology::multiple_type_inference;
use scraper::{Html, Selector};

pub struct AddGeneProvider;

#[async_trait]
impl super::ProviderEnumTrait for AddGeneProvider {
    fn name(&self) -> &'static str {
        "AddGene"
    }

    fn link(&self, id: &str) -> String {
        format!("https://www.addgene.org/{}/", id)
    }

    fn url(&self, id: &str) -> String {
        format!("https://www.addgene.org/{}/sequences/", id)
    }

    async fn parse(&self, id: &str, html_text: &str) -> Option<Biobrick> {
        let client = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .build()
            .ok()?;

        // 1. Extract sequence data from the sequence page (passed as html_text)
        let (genbank_link, fallback_seq) = {
            let document = Html::parse_document(html_text);

            let mut g_link = None;
            let snapgene_container_selector = Selector::parse(".item-sequence-container-snapgene").unwrap();
            
            for container in document.select(&snapgene_container_selector) {
                let container_html = container.html();
                let is_ngs = container_html.contains("Addgene NGS Result");
                
                let gbk_selector = Selector::parse("a.genbank-file-download").unwrap();
                if let Some(gbk_el) = container.select(&gbk_selector).next() {
                    let href = gbk_el.value().attr("href").unwrap_or("");
                    if !href.is_empty() {
                        let mut url = href.to_string();
                        if url.starts_with('/') {
                            url = format!("https://www.addgene.org{}", url);
                        }
                        g_link = Some(url);
                        if is_ngs { break; } // NGS is priority
                    }
                }
            }

            if g_link.is_none() {
                let anchor_selector = Selector::parse("a").unwrap();
                for anchor in document.select(&anchor_selector) {
                    let text = anchor.text().collect::<String>().to_lowercase();
                    let href = anchor.value().attr("href").unwrap_or("").to_lowercase();
                    if (text.contains("genbank") || href.ends_with(".gbk")) && !href.is_empty() {
                        let mut url = anchor.value().attr("href").unwrap().to_string();
                        if url.starts_with('/') {
                            url = format!("https://www.addgene.org{}", url);
                        }
                        g_link = Some(url);
                        if href.contains("addgene-plasmid") && href.contains("sequence") {
                            break; 
                        }
                    }
                }
            }

            let mut f_seq = None;
            let textarea_selector = Selector::parse("textarea.copy-from").unwrap();
            for textarea in document.select(&textarea_selector) {
                let text = textarea.text().collect::<String>();
                if text.is_empty() { continue; }
                
                let is_ngs = text.contains("NGS Result");
                let seq = text.lines()
                    .skip_while(|line| line.starts_with('>'))
                    .collect::<Vec<_>>()
                    .join("")
                    .replace(['\n', '\r', ' '], "");
                
                if !seq.is_empty() {
                    f_seq = Some(seq);
                    if is_ngs { break; }
                }
            }

            (g_link, f_seq)
        };

        // 2. Fetch Home page for "Purpose" & metadata
        let mut material_name = None;
        let mut authors = Vec::new();
        let mut purpose = None;

        if let Ok(resp) = client.get(&self.link(id)).send().await {
            if let Ok(text) = resp.text().await {
                let (mn, auts, purp) = {
                    let document = Html::parse_document(&text);
                    let m_name = document.select(&Selector::parse(".material-name").unwrap())
                        .next()
                        .map(|el| el.text().collect::<String>().trim().to_string());

                    let mut a_list = Vec::new();
                    if let Some(pi_el) = document.select(&Selector::parse(".breadcrumb-pi a").unwrap()).next() {
                        a_list.push(Author {
                            name: pi_el.text().collect::<String>().trim().to_string(),
                            role: None,
                        });
                    }

                    let p_text = document.select(&Selector::parse(".field").unwrap())
                        .find(|el| {
                            el.select(&Selector::parse(".field-label").unwrap())
                                .next()
                                .map(|l| l.text().collect::<String>().trim() == "Purpose")
                                .unwrap_or(false)
                        })
                        .and_then(|el| el.select(&Selector::parse(".field-content").unwrap()).next())
                        .map(|el| {
                            el.text()
                                .collect::<String>()
                                .replace(['\n', '\r'], " ")
                                .split_whitespace()
                                .collect::<Vec<_>>()
                                .join(" ")
                        });
                    
                    (m_name, a_list, p_text)
                };
                material_name = mn;
                authors = auts;
                purpose = purp;
            }
        }

        // 3. Final construction
        let mut biobrick = None;
        let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);

        if let Some(url) = genbank_link {
            if let Ok(resp) = client.get(&url).send().await {
                if let Ok(gb_text) = resp.text().await {
                    if let Some(gb_data) = parse_genbank_raw(&gb_text) {
                        let mut brick = genbank_to_biobrick(id, self.name(), &self.link(id), gb_data, now.clone());
                        
                        // Override/Augment metadata
                        if let Some(ref name) = material_name {
                            brick.metadata.name = name.clone();
                        }
                        if !authors.is_empty() {
                            brick.metadata.authors = authors.clone();
                        }
                        if let Some(ref purp) = purpose {
                            brick.metadata.description = purp.clone();
                        }

                        // Remove "creation" as requested
                        brick.metadata.creation = String::new();
                        
                        biobrick = Some(brick);
                    }
                }
            }
        }

        if biobrick.is_none() {
            if let Some(seq) = fallback_seq {
                let name = material_name.unwrap_or_else(|| format!("AddGene Plasmid {}", id));
                biobrick = Some(Biobrick {
                    metadata: MetaBiobrick {
                        id: id.to_string(),
                        name: name.clone(),
                        description: purpose.unwrap_or_default(),
                        creation: String::new(),
                        size: seq.len() as i32,
                        circular: true,
                        r#type: multiple_type_inference(&[name]).into(),
                        authors,
                        providers: vec![MetaProvider {
                            name: self.name().to_string(),
                            link: self.link(id),
                            date: now,
                        }],
                    },
                    sequence: seq,
                    features: vec![],
                });
            }
        }

        biobrick
    }
}
