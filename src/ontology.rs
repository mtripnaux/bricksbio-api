use serde::{Serialize, Deserialize};
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct OntologyEntry {
    pub canonical: &'static str,
    pub ontology: Option<&'static str>,
    pub visual: Option<&'static str>,
    pub also: &'static [&'static str],
    pub slug: Option<&'static str>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(crate = "serde")]
pub struct OntologyEntrySerializable {
    pub canonical: String,
    pub ontology: Option<String>,
    pub visual: Option<String>,
    pub also: Vec<String>,
    pub slug: Option<String>,
}

impl From<&OntologyEntry> for OntologyEntrySerializable {
    fn from(entry: &OntologyEntry) -> Self {
        OntologyEntrySerializable {
            canonical: entry.canonical.to_string(),
            ontology: entry.ontology.map(|s| s.to_string()),
            visual: entry.visual.map(|s| s.to_string()),
            also: entry.also.iter().map(|s| s.to_string()).collect(),
            slug: entry.slug.map(|s| s.to_string()),
        }
    }
}

pub const ONTOLOGY: &[OntologyEntry] = &[
    OntologyEntry {
        canonical: "unknown-feature",
        ontology: None,
        visual: Some("user-defined"),
        also: &["unknown", "sequence feature", "misc"],
        slug: Some("misc"),
    },
    OntologyEntry {
        canonical: "engineered-region",
        ontology: Some("SO:0000804"),
        visual: Some("engineered-region"),
        also: &["engineered region"],
        slug: None,
    },
    OntologyEntry {
        canonical: "coding-sequence",
        ontology: Some("SO:0000316"),
        visual: Some("cds"),
        also: &["coding sequence", "cds", "coding region"],
        slug: Some("cds"),
    },
    OntologyEntry {
        canonical: "ribosome-entry-site",
        ontology: Some("SO:0000139"),
        visual: Some("ribosome-entry-site"),
        also: &["rbs", "ribosome binding", "ribosome entry"],
        slug: Some("rbs"),
    },
    OntologyEntry {
        canonical: "assembly-scar",
        ontology: Some("SO:0001953"),
        visual: Some("assembly-scar"),
        also: &[],
        slug: Some("scar"),
    },
    OntologyEntry {
        canonical: "promoter",
        ontology: Some("SO:0000167"),
        visual: Some("promoter"),
        also: &[],
        slug: Some("promoter"),
    },
    OntologyEntry {
        canonical: "stop-codon",
        ontology: Some("SO:0000319"),
        visual: Some("stop-codon"),
        also: &[],
        slug: Some("stop-codon"),
    },
    OntologyEntry {
        canonical: "operator",
        ontology: Some("SO:0000057"),
        visual: Some("operator"),
        also: &[],
        slug: Some("operator"),
    },
    OntologyEntry {
        canonical: "primer-binding-site",
        ontology: Some("SO:0005850"),
        visual: Some("primer-binding-site"),
        also: &[
            "primer entry site",
            "pbs",
            "pes",
            "primer binding",
            "primer entry",
        ],
        slug: Some("primer-binding-site"),
    },
    OntologyEntry {
        canonical: "terminator",
        ontology: Some("SO:0000141"),
        visual: Some("terminator"),
        also: &[],
        slug: Some("terminator"),
    },
    OntologyEntry {
        canonical: "origin-of-replication",
        ontology: Some("SO:0000296"),
        visual: Some("origin-of-replication"),
        also: &["origin"],
        slug: Some("origin-of-replication"),
    },
    OntologyEntry {
        canonical: "deletion",
        ontology: Some("SO:0000159"),
        visual: Some("protein-stability-element"),
        also: &[],
        slug: Some("protein-stability-element"),
    },
    OntologyEntry {
        canonical: "poly-a-site",
        ontology: Some("SO:0000553"),
        visual: Some("poly-a-site"),
        also: &["polya"],
        slug: Some("poly-a-site"),
    },
    OntologyEntry {
        canonical: "composite",
        ontology: None,
        visual: Some("composite"),
        also: &["biobrick", "composite part"],
        slug: Some("composite"),
    },
    OntologyEntry {
        canonical: "plasmid-backbone",
        ontology: None,
        visual: Some("plasmid-backbone"),
        also: &["plasmid backbone", "backbone"],
        slug: Some("plasmid-backbone"),
    },
    OntologyEntry {
        canonical: "plasmid",
        ontology: Some("SO:0000155"),
        visual: None,
        also: &["vector", "complete plasmid", "plasmid sequence"],
        slug: Some("plasmid"),
    },
    OntologyEntry {
        canonical: "inert-dna-spacer",
        ontology: Some("SO:0002223"),
        visual: None,
        also: &["spacer", "dna spacer"],
        slug: Some("spacer"),
    },
    OntologyEntry {
        canonical: "aptamer",
        ontology: Some("SO:0000031"),
        visual: Some("aptamer"),
        also: &["aptamer"],
        slug: Some("aptamer"),
    }
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