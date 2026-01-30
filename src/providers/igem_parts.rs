use crate::providers::Provider;
use crate::types::{Biobrick, MetaBiobrick, MetaFeature, MetaProvider, Location, Author};
use crate::ontology::multiple_type_inference;
use scraper::{Html, Selector};

pub struct IgemPartsProvider;

impl Provider for IgemPartsProvider {
    fn name(&self) -> &'static str {
        "iGEM Parts Registry (Legacy)"
    }
    
    fn link(&self, id: &str) -> String {
        format!("https://parts.igem.org/Part:{}", id)
    }
    
    fn url(&self, id: &str) -> String {
        self.link(id)
    }
    
    fn parse(&self, id: &str, html_text: &str) -> Option<Biobrick> {
        println!("    Parsing iGEM Parts HTML, length: {}", html_text.len());
        
        let document = Html::parse_document(html_text);

        let mut name = String::new();
        let mut description = String::new();
        let p_selector = Selector::parse("p").unwrap();
        let span_selector = Selector::parse("span").unwrap();
        if let Ok(content_sel) = Selector::parse("#mw-content-text") {
            if let Some(content) = document.select(&content_sel).next() {
                let mut ps = content.select(&p_selector);
                if let Some(first_p) = ps.next() {
                    if let Some(span) = first_p.select(&span_selector).next() {
                        name = span.text().collect::<String>().trim().to_string();
                    }
                    if let Some(second_p) = ps.next() {
                        description = second_p.text().collect::<String>().trim().to_string();
                    }
                }
            }
        }

        if name.is_empty() {
            if let Ok(selector) = Selector::parse("span#part_name") {
                name = document.select(&selector)
                    .next()
                    .map(|el| el.text().collect::<String>().trim().to_string())
                    .filter(|n| !n.is_empty())
                    .unwrap_or_else(|| id.to_string());
            } else {
                name = id.to_string();
            }
        }

        if description.is_empty() {
            if let Ok(p_sel) = Selector::parse("p") {
                description = document.select(&p_sel)
                    .next()
                    .map(|el| el.text().collect::<String>().trim().to_string())
                    .unwrap_or_default();
            }
        }

        let edit_html = match fetch_edit_page(id) {
            Ok(html) => html,
            Err(e) => {
                println!("    Error fetching edit page: {}", e);
                return None;
            }
        };
        
        let edit_doc = Html::parse_document(&edit_html);
        
        let sequence = extract_sequence_from_edit(&edit_doc);
        if sequence.is_empty() {
            println!("    No sequence found in edit page, aborting");
            return None;
        }
        println!("    Got sequence: {} bp", sequence.len());
        
        let features = extract_features_from_edit(&edit_doc);
        println!("    Found {} features", features.len());
        
        let mut authors = extract_authors_from_edit(&edit_doc);
        if authors.is_empty() {
            authors = extract_authors(&document);
        }
        
        let part_type = extract_part_type(&document).unwrap_or_else(|| "unknown".to_string());
        
            Some(Biobrick {
                metadata: MetaBiobrick {
                    id: id.to_string(),
                    name,
                    r#type: multiple_type_inference(&[part_type.clone(), description.clone()]).into(),
                    circular: false,
                    size: sequence.len() as i32,
                    providers: vec![MetaProvider {
                        name: self.name().to_string(),
                        link: self.link(id),
                    }],
                    description,
                    authors,
                },
                sequence,
                features,
            })
    }
}

fn fetch_edit_page(id: &str) -> Result<String, Box<dyn std::error::Error>> {
    let url = format!("https://parts.igem.org/partsdb/edit_seq.cgi?part={}", id);
    println!("    Fetching edit page: {}", url);
    
    let text = ureq::get(&url)
        .set("User-Agent", "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .timeout(std::time::Duration::from_secs(10))
        .call()?
        .into_string()?;
    
    Ok(text)
}

fn extract_sequence_from_edit(document: &Html) -> String {
    if let Ok(selector) = Selector::parse("textarea[name='user_input']") {
        if let Some(element) = document.select(&selector).next() {
            let raw = element.text().collect::<String>();
            return raw.chars()
                .filter(|c| c.is_ascii_alphabetic())
                .collect::<String>()
                .to_lowercase();
        }
    }
    String::new()
}

fn extract_features_from_edit(document: &Html) -> Vec<MetaFeature> {
    let mut features = Vec::new();
    
    if let Ok(selector) = Selector::parse("div[id^='regular_features_'] table tr") {
        for row in document.select(&selector) {
            let td_selector = Selector::parse("td").unwrap();
            let cells: Vec<String> = row.select(&td_selector)
                .map(|el| el.text().collect::<String>().trim().to_string())
                .collect();
            
            if cells.len() >= 6 {
                let id = cells[0].clone();
                let kind = cells[1].to_lowercase();
                let label = cells[2].clone();
                let start = cells[3].parse::<i32>().unwrap_or(0);
                let end = cells[4].parse::<i32>().unwrap_or(0);
                let direction = cells[5].to_lowercase();
                
                let strand = if direction.contains("rev") { 2 } else { 1 };
                
                features.push(MetaFeature {
                    id: format!("igem_{}", id),
                    name: if label.is_empty() { kind.clone() } else { label },
                    r#type: multiple_type_inference(&[kind.clone()]).into(),
                    location: Location {
                        start,
                        end,
                        strand,
                        forward: strand == 1,
                    },
                });
            }
        }
    }
    
    features
}

fn extract_authors_from_edit(document: &Html) -> Vec<Author> {
    let mut authors = Vec::new();
    if let Ok(selector) = Selector::parse("div") {
        for div in document.select(&selector) {
            let text = div.text().collect::<String>();
            if text.contains("Designed by:") && text.len() < 500 {
                if let Some(idx) = text.find("Designed by:") {
                    let after = &text[idx + 12..];
                    let author_part = if let Some(g_idx) = after.find("Group:") {
                        &after[..g_idx]
                    } else if let Some(p_idx) = after.find('(') {
                        &after[..p_idx]
                    } else {
                        after
                    };
                    
                    let names = split_authors(author_part);
                    for name in names {
                        authors.push(Author {
                            name,
                            role: None,
                        });
                    }
                    if !authors.is_empty() {
                        return authors;
                    }
                }
            }
        }
    }
    authors
}

fn split_authors(text: &str) -> Vec<String> {
    let mut result = Vec::new();
    let parts = text.split(|c| c == ',' || c == ';');
    for part in parts {
        let sub_parts = if part.contains(" and ") {
            part.split(" and ").collect::<Vec<_>>()
        } else {
            vec![part]
        };
        
        for name in sub_parts {
            let mut cleaned = name.trim().to_string();
            
            cleaned = cleaned.split_whitespace().collect::<Vec<_>>().join(" ");
            
            if cleaned.ends_with('.') {
                cleaned.pop();
                cleaned = cleaned.trim().to_string();
            }
            
            if cleaned.is_empty() || cleaned.len() < 3 {
                continue;
            }
            
            if cleaned.chars().all(|c| c.is_numeric()) {
                continue;
            }

            if !cleaned.to_lowercase().contains("designed by") {
                result.push(cleaned);
            }
        }
    }
    result
}

fn extract_part_type(document: &Html) -> Option<String> {
    if let Ok(selector) = Selector::parse("div[title='Part Type']") {
        if let Some(element) = document.select(&selector).next() {
            let part_type = element.text().collect::<String>().trim().to_string();
            if !part_type.is_empty() {
                println!("      Found part type: {}", part_type);
                return Some(part_type);
            }
        }
    }
    
    let text = document.root_element().text().collect::<String>();
    if let Some(idx) = text.find("Type:") {
        let remaining = &text[idx + 5..];
        if let Some(end_idx) = remaining.find(|c: char| c == '\n' || c == '<') {
            let found = remaining[..end_idx].trim().to_string();
            println!("      Found part type from text: {}", found);
            return Some(found);
        }
    }
    None
}

fn extract_authors(document: &Html) -> Vec<Author> {
    let mut authors = Vec::new();
    let mut seen_names = std::collections::HashSet::new();
    
    if let Ok(div_selector) = Selector::parse("div") {
        for div in document.select(&div_selector) {
            let text = div.text().collect::<String>();
            if text.contains("Designed by:") && text.len() < 300 {
                if let Some(idx) = text.find("Designed by:") {
                    let after_designed = &text[idx + 12..];
                    
                    let author_chunk = if let Some(group_idx) = after_designed.find("Group:") {
                        &after_designed[..group_idx]
                    } else if let Some(term_idx) = after_designed.find('(') {
                        &after_designed[..term_idx]
                    } else {
                        after_designed
                    };

                    let names = split_authors(author_chunk);
                    for name in names {
                        if seen_names.insert(name.clone()) {
                            authors.push(Author {
                                name,
                                role: None,
                            });
                        }
                    }
                    
                    if !authors.is_empty() {
                        return authors;
                    }
                }
            }
        }
    }
    authors
}
