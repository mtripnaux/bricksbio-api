use std::collections::HashSet;

#[allow(dead_code)]
pub struct OntologyEntry {
    pub canonical: &'static str,
    pub so: Option<&'static str>,
    pub css: &'static str,
    pub also: &'static [&'static str],
}

pub const ONTOLOGY: &[OntologyEntry] = &[
    OntologyEntry {
        canonical: "coding_sequence",
        so: Some("SO:0000316"),
        css: "cds",
        also: &["coding sequence", "cds", "coding region"],
    },
    OntologyEntry {
        canonical: "ribosome_entry_site",
        so: Some("SO:0000139"),
        css: "ribosome-entry-site",
        also: &["rbs", "ribosome binding", "ribosome entry"],
    },
    OntologyEntry {
        canonical: "scar",
        so: None,
        css: "scar",
        also: &[],
    },
    OntologyEntry {
        canonical: "promoter",
        so: Some("SO:0000167"),
        css: "promoter",
        also: &[],
    },
    OntologyEntry {
        canonical: "stop_codon",
        so: Some("SO:0000319"),
        css: "stop-codon",
        also: &[],
    },
    OntologyEntry {
        canonical: "operator",
        so: Some("SO:0000057"),
        css: "operator",
        also: &[],
    },
    OntologyEntry {
        canonical: "primer_binding_site",
        so: Some("SO:0005850"),
        css: "primer-binding-site",
        also: &[
            "primer entry site",
            "pbs",
            "pes",
            "primer binding",
            "primer entry",
        ],
    },
    OntologyEntry {
        canonical: "terminator",
        so: Some("SO:0000141"),
        css: "terminator",
        also: &[],
    },
    OntologyEntry {
        canonical: "origin_of_replication",
        so: Some("SO:0000296"),
        css: "origin-of-replication",
        also: &["origin"],
    },
    OntologyEntry {
        canonical: "operator",
        so: Some("SO:0000057"),
        css: "operator",
        also: &[],
    },
    OntologyEntry {
        canonical: "deletion",
        so: Some("SO:0000159"),
        css: "protein-stability-element",
        also: &[],
    },
    OntologyEntry {
        canonical: "polya_site",
        so: Some("SO:0000553"),
        css: "poly-a-site",
        also: &["polya"],
    },
    OntologyEntry {
        canonical: "composite",
        so: None,
        css: "composite",
        also: &["biobrick", "composite part"],
    },
    OntologyEntry {
        canonical: "plasmid",
        so: Some("SO:0000155"),
        css: "plasmid",
        also: &["vector", "complete plasmid", "plasmid sequence"],
    },
];

pub fn type_inference(note: &str) -> String {
    if note.is_empty() {
        return "sequence_feature".to_string();
    }

    let note_lower = note
        .to_lowercase()
        .replace('_', " ")
        .replace('-', " ");

    for entry in ONTOLOGY {
        if note_lower.contains(&entry.canonical.replace('_', " ")) {
            return entry.canonical.to_string();
        }
        for synonym in entry.also {
            if note_lower.contains(&synonym.to_lowercase()) {
                return entry.canonical.to_string();
            }
        }
    }

    "sequence_feature".to_string()
}

pub fn multiple_type_inference(notes: &[String]) -> String {
    let mut results = HashSet::new();

    for note in notes {
        let r#type = type_inference(note);
        if r#type != "sequence_feature" {
            results.insert(r#type);
        }
    }

    if results.len() == 1 {
        return results.into_iter().next().unwrap();
    }

    if results.len() > 1 {
        return results.into_iter().next().unwrap();
    }

    "sequence_feature".to_string()
}