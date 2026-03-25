use std::sync::{Mutex, OnceLock};

use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};

use crate::cache::SqliteCache;
use crate::types::Biobrick;

pub fn cache_search(cache: &SqliteCache, query: &str, n: usize) -> Result<Vec<(Biobrick, f64)>, rusqlite::Error> {
    let n = n.min(50);
    if n == 0 {
        return Ok(vec![]);
    }

    let query = query.trim();
    if query.is_empty() {
        return Ok(vec![]);
    }

    let parts = cache.list_parts()?;
    if parts.is_empty() {
        return Ok(vec![]);
    }

    let docs = parts.iter().map(biobrick_to_text).collect::<Vec<_>>();
    let model_mutex = embedding_model().map_err(rusqlite::Error::ToSqlConversionFailure)?;
    let model = model_mutex
        .lock()
        .map_err(|_| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(std::io::Error::other(
                "Embedding model lock poisoned",
            )))
        })?;

    let mut inputs = Vec::with_capacity(docs.len() + 1);
    inputs.push(query.to_string());
    inputs.extend(docs);

    let embeddings = model
        .embed(inputs, None)
        .map_err(|e| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(std::io::Error::other(
                e.to_string(),
            )))
        })?;

    if embeddings.len() < 2 {
        return Ok(vec![]);
    }

    let query_embedding = &embeddings[0];
    let doc_embeddings = &embeddings[1..];

    let candidate_indices = ann_candidates(query_embedding, doc_embeddings, n);

    let mut scored: Vec<(Biobrick, f64)> = candidate_indices
        .into_iter()
        .filter_map(|i| {
            let score = cosine_similarity(query_embedding, &doc_embeddings[i]);
            parts.get(i).cloned().map(|p| (p, score.max(0.0)))
        })
        .filter(|(_, score)| *score > 0.05)
        .collect();

    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    scored.truncate(n);
    Ok(scored)
}

fn embedding_model() -> Result<&'static Mutex<TextEmbedding>, Box<dyn std::error::Error + Send + Sync>> {
    static MODEL: OnceLock<Mutex<TextEmbedding>> = OnceLock::new();
    if let Some(model) = MODEL.get() {
        return Ok(model);
    }

    let options = InitOptions::new(EmbeddingModel::AllMiniLML6V2);
    let embedding = TextEmbedding::try_new(options)?;
    Ok(MODEL.get_or_init(|| Mutex::new(embedding)))
}

fn biobrick_to_text(biobrick: &Biobrick) -> String {
    let mut text = String::new();
    text.push_str("id: ");
    text.push_str(&biobrick.metadata.id);
    text.push_str("\nname: ");
    text.push_str(&biobrick.metadata.name);
    text.push_str("\ndescription: ");
    text.push_str(&biobrick.metadata.description);
    text.push_str("\ntype: ");
    text.push_str(&biobrick.metadata.r#type.canonical);

    if !biobrick.metadata.authors.is_empty() {
        text.push_str("\nauthors: ");
        text.push_str(
            &biobrick
                .metadata
                .authors
                .iter()
                .map(|a| a.name.as_str())
                .collect::<Vec<_>>()
                .join(", "),
        );
    }

    if !biobrick.features.is_empty() {
        text.push_str("\nfeatures: ");
        text.push_str(
            &biobrick
                .features
                .iter()
                .map(|f| format!("{} {} {}", f.id, f.name, f.r#type.canonical))
                .collect::<Vec<_>>()
                .join(" | "),
        );
    }

    text
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f64 {
    if a.is_empty() || b.is_empty() || a.len() != b.len() {
        return 0.0;
    }

    let mut dot = 0.0_f64;
    let mut norm_a = 0.0_f64;
    let mut norm_b = 0.0_f64;

    for i in 0..a.len() {
        let av = a[i] as f64;
        let bv = b[i] as f64;
        dot += av * bv;
        norm_a += av * av;
        norm_b += bv * bv;
    }

    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }

    (dot / (norm_a.sqrt() * norm_b.sqrt())).clamp(-1.0, 1.0)
}

fn ann_candidates(query_embedding: &[f32], doc_embeddings: &[Vec<f32>], n: usize) -> Vec<usize> {
    const ANN_THRESHOLD: usize = 200;
    if doc_embeddings.len() <= ANN_THRESHOLD {
        return (0..doc_embeddings.len()).collect();
    }

    let dim = query_embedding.len();
    let planes = random_planes(dim, 64);
    let q_sig = signature(query_embedding, &planes);

    let mut hamming_ranked = doc_embeddings
        .iter()
        .enumerate()
        .map(|(i, emb)| (i, (q_sig ^ signature(emb, &planes)).count_ones()))
        .collect::<Vec<_>>();

    hamming_ranked.sort_by_key(|(_, dist)| *dist);
    let k = (n * 12).max(64).min(doc_embeddings.len());
    hamming_ranked.into_iter().take(k).map(|(i, _)| i).collect()
}

fn random_planes(dim: usize, count: usize) -> Vec<Vec<f32>> {
    let mut seed: u64 = 0x9E3779B97F4A7C15;
    (0..count)
        .map(|_| {
            (0..dim)
                .map(|_| {
                    seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
                    let bits = ((seed >> 33) as u32) | 1;
                    let v = (bits as f32) / (u32::MAX as f32);
                    (v * 2.0) - 1.0
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

fn signature(embedding: &[f32], planes: &[Vec<f32>]) -> u64 {
    let mut sig = 0_u64;
    for (bit, plane) in planes.iter().take(64).enumerate() {
        let mut dot = 0.0_f32;
        for i in 0..embedding.len().min(plane.len()) {
            dot += embedding[i] * plane[i];
        }
        if dot >= 0.0 {
            sig |= 1_u64 << bit;
        }
    }
    sig
}