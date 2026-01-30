use crate::types::{Biobrick, MetaBiobrick, MetaFeature, MetaProvider, Author};
use crate::ontology::multiple_type_inference;
use std::collections::HashSet;

pub fn enrich(biobrick1: Biobrick, biobrick2: Biobrick) -> Biobrick {
    if biobrick1.sequence != biobrick2.sequence {
        eprintln!("Mismatched sequence");
    }

    if biobrick1.metadata.id != biobrick2.metadata.id {
        panic!("Mismatched ids");
    }

    let mut merged_features = biobrick1.metadata.features.clone();
    merged_features.extend(biobrick2.metadata.features);

    let merged_authors = concat_unique_authors(&biobrick1.authors, &biobrick2.authors);

    Biobrick {
        metadata: MetaBiobrick {
            id: biobrick1.metadata.id.clone(),
            name: merge_strings(&biobrick1.metadata.name, &biobrick2.metadata.name),
            r#type: multiple_type_inference(&[
                biobrick1.metadata.r#type.canonical.clone(),
                biobrick2.metadata.r#type.canonical.clone(),
            ]).into(),
            size: biobrick1.metadata.size,
            circular: biobrick1.metadata.circular || biobrick2.metadata.circular,
            providers: concat_unique_providers(
                &biobrick1.metadata.providers,
                &biobrick2.metadata.providers,
            ),
            description: merge_strings(&biobrick1.metadata.description, &biobrick2.metadata.description),
            features: clean_features_list(merged_features, biobrick1.metadata.size),
        },
        sequence: biobrick1.sequence,
        authors: merged_authors,
    }
}

fn concat_unique_providers(list1: &[MetaProvider], list2: &[MetaProvider]) -> Vec<MetaProvider> {
    let mut seen_names = HashSet::new();
    let mut unique = Vec::new();
    
    for provider in list1 {
        if seen_names.insert(provider.name.clone()) {
            unique.push(provider.clone());
        }
    }
    
    for provider in list2 {
        if seen_names.insert(provider.name.clone()) {
            unique.push(provider.clone());
        }
    }
    
    unique
}

fn concat_unique_authors(list1: &[Author], list2: &[Author]) -> Vec<Author> {
    let mut seen_names = HashSet::new();
    let mut unique = Vec::new();
    
    for author in list1 {
        if seen_names.insert(author.name.clone()) {
            unique.push(author.clone());
        }
    }
    
    for author in list2 {
        if seen_names.insert(author.name.clone()) {
            unique.push(author.clone());
        }
    }
    
    unique
}

#[allow(dead_code)]
pub fn merge_features(f1: MetaFeature, f2: MetaFeature) -> MetaFeature {
    MetaFeature {
        name: merge_strings(&f1.name, &f2.name),
        id: f1.id,
        r#type: multiple_type_inference(&[f1.r#type.canonical.clone(), f2.r#type.canonical.clone()]).into(),
        location: f1.location,
    }
}

fn compare_strings(a: &str, b: &str) -> bool {
    let allowed_difference = 3;
    let mut difference_count = 0;

    let chars_a: Vec<char> = a.chars().collect();
    let chars_b: Vec<char> = b.chars().collect();

    for i in 0..std::cmp::min(chars_a.len(), chars_b.len()) {
        if chars_a[i].to_lowercase().next() != chars_b[i].to_lowercase().next() {
            difference_count += 1;
            if difference_count > allowed_difference {
                return false;
            }
        }
    }

    difference_count += (chars_a.len() as i32 - chars_b.len() as i32).abs();
    if difference_count > allowed_difference {
        return false;
    }

    if a.is_empty() || b.is_empty() {
        return false;
    }

    a.to_lowercase() == b.to_lowercase()
}

pub fn merge_strings(a: &str, b: &str) -> String {
    if a.is_empty() {
        return b.to_string();
    }
    if b.is_empty() {
        return a.to_string();
    }

    if compare_strings(a, b) {
        return a.to_string();
    }

    if a.contains(b) {
        return a.to_string();
    }
    if b.contains(a) {
        return b.to_string();
    }

    format!("{} / {}", a, b)
}

fn clean_features_list(features: Vec<MetaFeature>, _size: i32) -> Vec<MetaFeature> {
    let mut seen = HashSet::new();
    let mut unique = Vec::new();
    for f in features {
        if seen.insert(f.id.clone()) {
            unique.push(f);
        }
    }
    unique
}