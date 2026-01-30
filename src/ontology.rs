use serde::{Serialize, Deserialize};
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct OntologyEntry {
    pub canonical: &'static str,
    pub ontology: Option<&'static str>,
    pub css: &'static str,
    pub also: &'static [&'static str],
    pub slug: &'static str,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(crate = "serde")]
pub struct OntologyEntrySerializable {
    pub canonical: String,
    pub ontology: Option<String>,
    pub css: String,
    pub also: Vec<String>,
    pub slug: String,
}

impl From<&OntologyEntry> for OntologyEntrySerializable {
    fn from(entry: &OntologyEntry) -> Self {
        OntologyEntrySerializable {
            canonical: entry.canonical.to_string(),
            ontology: entry.ontology.map(|s| s.to_string()),
            css: entry.css.to_string(),
            also: entry.also.iter().map(|s| s.to_string()).collect(),
            slug: entry.slug.to_string(),
        }
    }
}

pub const ONTOLOGY: &[OntologyEntry] = &[
    OntologyEntry {
        canonical: "sequence_feature",
        ontology: None,
        css: "sequence-feature",
        also: &[],
        slug: "misc",
    },
    OntologyEntry {
        canonical: "coding_sequence",
        ontology: Some("SO:0000316"),
        css: "cds",
        also: &["coding sequence", "cds", "coding region"],
        slug: "cds",
    },
    OntologyEntry {
        canonical: "ribosome_entry_site",
        ontology: Some("SO:0000139"),
        css: "ribosome-entry-site",
        also: &["rbs", "ribosome binding", "ribosome entry"],
        slug: "rbs",
    },
    OntologyEntry {
        canonical: "scar",
        ontology: None,
        css: "scar",
        also: &[],
        slug: "scar",
    },
    OntologyEntry {
        canonical: "promoter",
        ontology: Some("SO:0000167"),
        css: "promoter",
        also: &[],
        slug: "promoter",
    },
    OntologyEntry {
        canonical: "stop_codon",
        ontology: Some("SO:0000319"),
        css: "stop-codon",
        also: &[],
        slug: "stop-codon",
    },
    OntologyEntry {
        canonical: "operator",
        ontology: Some("SO:0000057"),
        css: "operator",
        also: &[],
        slug: "operator",
    },
    OntologyEntry {
        canonical: "primer_binding_site",
        ontology: Some("SO:0005850"),
        css: "primer-binding-site",
        also: &[
            "primer entry site",
            "pbs",
            "pes",
            "primer binding",
            "primer entry",
        ],
        slug: "primer-binding-site",
    },
    OntologyEntry {
        canonical: "terminator",
        ontology: Some("SO:0000141"),
        css: "terminator",
        also: &[],
        slug: "terminator",
    },
    OntologyEntry {
        canonical: "origin_of_replication",
        ontology: Some("SO:0000296"),
        css: "origin-of-replication",
        also: &["origin"],
        slug: "origin-of-replication",
    },
    OntologyEntry {
        canonical: "operator",
        ontology: Some("SO:0000057"),
        css: "operator",
        also: &[],
        slug: "operator",
    },
    OntologyEntry {
        canonical: "deletion",
        ontology: Some("SO:0000159"),
        css: "protein-stability-element",
        also: &[],
        slug: "protein-stability-element",
    },
    OntologyEntry {
        canonical: "polya_site",
        ontology: Some("SO:0000553"),
        css: "poly-a-site",
        also: &["polya"],
        slug: "poly-a-site",
    },
    OntologyEntry {
        canonical: "composite",
        ontology: None,
        css: "composite",
        also: &["biobrick", "composite part"],
        slug: "composite",
    },
    OntologyEntry {
        canonical: "plasmid_backbone",
        ontology: None,
        css: "plasmid-backbone",
        also: &["plasmid backbone", "backbone"],
        slug: "plasmid-backbone",
    },
    OntologyEntry {
        canonical: "plasmid",
        ontology: Some("SO:0000155"),
        css: "plasmid",
        also: &["vector", "complete plasmid", "plasmid sequence"],
        slug: "plasmid",
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