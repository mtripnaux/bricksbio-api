use crate::types::{Biobrick, MetaBiobrick, MetaFeature, MetaProvider, Location};
use crate::ontology::multiple_type_inference;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct GenBankData {
    pub name: String,
    pub definition: String,
    pub creation: Option<String>,
    pub sequence: String,
    pub circular: bool,
    pub features: Vec<GenBankFeature>,
}

#[derive(Debug, Clone)]
pub struct GenBankFeature {
    pub kind: String,
    pub start: i32,
    pub end: i32,
    pub strand: i32,
    pub qualifiers: Vec<(String, String)>,
}

pub fn parse_genbank_raw(text: &str) -> Option<GenBankData> {
    let lines: Vec<&str> = text.lines().collect();
    if lines.is_empty() {
        return None;
    }
    let mut name = String::new();
    let mut definition = String::new();
    let mut sequence = String::new();
    let mut circular = false;
    let mut features = Vec::new();
    let mut in_features = false;
    let mut in_origin = false;
    let mut creation = None;
    for line in lines {
        let trimmed = line.trim();
        if line.starts_with("LOCUS") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                name = parts[1].to_string();
            }
            if line.contains("circular") {
                circular = true;
            }
            if parts.len() >= 6 {
                let date_str = parts[parts.len()-1];
                let normalized_date = if date_str.contains('-') {
                    let d_parts: Vec<&str> = date_str.split('-').collect();
                    if d_parts.len() == 3 {
                        let mut month = d_parts[1].to_lowercase();
                        if let Some(first) = month.get_mut(0..1) {
                            first.make_ascii_uppercase();
                        }
                        format!("{}-{}-{}", d_parts[0], month, d_parts[2])
                    } else {
                        date_str.to_string()
                    }
                } else {
                    date_str.to_string()
                };

                if let Ok(parsed) = chrono::NaiveDate::parse_from_str(&normalized_date, "%d-%b-%Y") {
                    creation = Some(parsed.format("%Y-%m-%dT00:00:00.000Z").to_string());
                }
            }
        } else if line.starts_with("DEFINITION") {
            definition = line["DEFINITION".len()..].trim().to_string();
        } else if line.starts_with("FEATURES") {
            in_features = true;
            in_origin = false;
        } else if line.starts_with("ORIGIN") {
            in_features = false;
            in_origin = true;
        } else if line.starts_with("//") {
            break;
        } else if in_features && line.starts_with("     ") && !line.starts_with("                     ") {
            let feature_line = line.trim();
            if let Some(space_idx) = feature_line.find(char::is_whitespace) {
                let kind = feature_line[..space_idx].to_string();
                let location = feature_line[space_idx..].trim();
                if let Some((start, end, strand)) = parse_location(location) {
                    features.push(GenBankFeature {
                        kind,
                        start,
                        end,
                        strand,
                        qualifiers: Vec::new(),
                    });
                }
            }
        } else if in_features && line.starts_with("                     /") {
            if let Some(last_feature) = features.last_mut() {
                let qualifier = line.trim();
                if qualifier.starts_with('/') {
                    if let Some(eq_idx) = qualifier.find('=') {
                        let key = qualifier[1..eq_idx].to_string();
                        let value = qualifier[eq_idx + 1..].trim_matches('"').to_string();
                        last_feature.qualifiers.push((key, value));
                    }
                }
            }
        } else if in_origin && !trimmed.is_empty() {
            let seq_part: String = trimmed.chars()
                .filter(|c| c.is_alphabetic())
                .collect();
            sequence.push_str(&seq_part);
        }
    }
    if name.is_empty() && sequence.is_empty() {
        return None;
    }
    Some(GenBankData {
        name,
        definition,
        sequence,
        circular,
        features,
        creation,
    })
}

fn parse_location(loc: &str) -> Option<(i32, i32, i32)> {
    let loc = loc.trim();
    if loc.starts_with("complement(") && loc.ends_with(')') {
        let inner = &loc[11..loc.len() - 1];
        return parse_location(inner).map(|(s, e, _)| (s, e, 2));
    }
    if let Some(dot_idx) = loc.find("..") {
        let start = loc[..dot_idx].parse::<i32>().ok()?;
        let end = loc[dot_idx + 2..].parse::<i32>().ok()?;
        return Some((start, end, 1));
    }
    if let Ok(pos) = loc.parse::<i32>() {
        return Some((pos, pos, 1));
    }
    None
}

pub fn genbank_to_biobrick(id: &str, provider: &str, provider_link: &str, gb_data: GenBankData, date: String) -> Biobrick {
    let size = gb_data.sequence.len() as i32;
    let features: Vec<MetaFeature> = gb_data.features.iter().map(|f| {
        let name = f.qualifiers.iter()
            .find(|(k, _)| k == "label" || k == "gene" || k == "note" || k == "locus_tag")
            .map(|(_, v)| v.clone())
            .unwrap_or_else(|| f.kind.clone());
        MetaFeature {
            id: format!("{}_{}", name, f.start),
            name: name.clone(),
            r#type: multiple_type_inference(&[name.clone(), f.kind.clone()]).into(),
            location: Location {
                start: f.start,
                end: f.end,
                forward: f.strand == 1,
            },
        }
    }).collect();
    let name = if gb_data.definition.is_empty() {
        String::from("")
    } else {
        gb_data.definition.clone()
    };
    Biobrick {
        metadata: MetaBiobrick {
            id: id.to_string(),
            name,
            r#type: multiple_type_inference(&[gb_data.definition.clone()]).into(),
            circular: gb_data.circular,
            size,
            providers: vec![MetaProvider {
                name: provider.to_string(),
                link: provider_link.to_string(),
                date,
            }],
            description: String::from(""),
            authors: vec![],
            creation: gb_data.creation.unwrap_or_default(),
        },
        sequence: gb_data.sequence,
        features,
    }
}