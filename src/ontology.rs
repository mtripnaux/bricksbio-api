use serde::{Serialize, Deserialize};
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct OntologyEntry {
    pub canonical: &'static str,
    pub so: Option<&'static str>,
    pub css: &'static str,
    pub also: &'static [&'static str],
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(crate = "serde")]
pub struct OntologyEntrySerializable {
    pub canonical: String,
    pub so: Option<String>,
    pub css: String,
    pub also: Vec<String>,
}

impl From<&OntologyEntry> for OntologyEntrySerializable {
    fn from(entry: &OntologyEntry) -> Self {
        OntologyEntrySerializable {
            canonical: entry.canonical.to_string(),
            so: entry.so.map(|s| s.to_string()),
            css: entry.css.to_string(),
            also: entry.also.iter().map(|s| s.to_string()).collect(),
        }
    }
}

pub const ONTOLOGY: &[OntologyEntry] = &[
    OntologyEntry {
        canonical: "sequence_feature",
        so: None,
        css: "sequence-feature",
        also: &[],
    },
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
        canonical: "plasmid_backbone",
        so: None,
        css: "plasmid-backbone",
        also: &["plasmid backbone", "backbone"],
    },
    OntologyEntry {
        canonical: "plasmid",
        so: Some("SO:0000155"),
        css: "plasmid",
        also: &["vector", "complete plasmid", "plasmid sequence"],
    },
];


pub fn type_inference(note: &str) -> &'static OntologyEntry {
    if note.is_empty() {
        return ONTOLOGY.iter().find(|e| e.canonical == "sequence_feature").unwrap();
    }

    let note_lower = note
        .to_lowercase()
        .replace('_', " ")
        .replace('-', " ");

    for entry in ONTOLOGY {
        if note_lower.contains(&entry.canonical.replace('_', " ")) {
            return entry;
        }
        for synonym in entry.also {
            if note_lower.contains(&synonym.to_lowercase()) {
                return entry;
            }
        }
    }

    ONTOLOGY.iter().find(|e| e.canonical == "sequence_feature").unwrap()
}

pub fn multiple_type_inference(notes: &[String]) -> &'static OntologyEntry {
    let mut results = HashSet::new();

    for note in notes {
        let entry = type_inference(note);
        if entry.canonical != "sequence_feature" {
            results.insert(entry.canonical);
        }
    }

    if results.len() == 1 {
        let canonical = results.into_iter().next().unwrap();
        return ONTOLOGY.iter().find(|e| e.canonical == canonical).unwrap();
    }

    if results.len() > 1 {
        let canonical = results.into_iter().next().unwrap();
        return ONTOLOGY.iter().find(|e| e.canonical == canonical).unwrap();
    }

    ONTOLOGY.iter().find(|e| e.canonical == "sequence_feature").unwrap()
}