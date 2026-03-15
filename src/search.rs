use futures::future::join_all;

use crate::AppState;
use crate::types::Biobrick;
use crate::providers::get_all_providers;

pub async fn meta_search(state: &AppState, id: &str) -> Option<Biobrick> {
    let id_normalized = normalize_id(id);

    if let Some(cached) = state.cache.get_part(&id_normalized) {
        spawn_refresh(state.clone(), id.to_string(), id_normalized).await;
        return Some(cached);
    }

    let fetched = fetch_and_merge(&state.client, id).await;

    if let Some(ref biobrick) = fetched {
        if let Err(error) = state.cache.put_part(&normalize_id(id), biobrick) {
            eprintln!("Failed to persist cache entry for {}: {}", id, error);
        }
    }

    fetched
}

async fn spawn_refresh(state: AppState, id: String, id_normalized: String) {
    let mut refresh_in_flight = state.refresh_in_flight.lock().await;
    if !refresh_in_flight.insert(id_normalized.clone()) {
        return;
    }
    drop(refresh_in_flight);

    tokio::spawn(async move {
        let refresh_result = fetch_and_merge(&state.client, &id).await;

        if let Some(ref biobrick) = refresh_result {
            if let Err(error) = state.cache.put_part(&id_normalized, biobrick) {
                eprintln!("Failed to refresh cache entry for {}: {}", id, error);
            }
        }

        let mut refresh_in_flight = state.refresh_in_flight.lock().await;
        refresh_in_flight.remove(&id_normalized);
    });
}

async fn fetch_and_merge(client: &reqwest::Client, id: &str) -> Option<Biobrick> {
    println!("Searching for part: {}", id);
    
    let providers = get_all_providers();
    
    let mut futures = Vec::new();

    for provider in providers {
        let url = provider.url(id);
        let name = provider.name();
        let client = client.clone();
        
        futures.push(async move {
            println!("  Trying provider: {} - {}", name, url);
            let res = client.get(&url).send().await;
            match res {
                Ok(response) => {
                    if response.status().is_success() {
                        let text = response.text().await.unwrap_or_default();
                        let parsed = provider.parse(id, &text).await;
                        return parsed;
                    } else {
                        println!("    Failed with status: {}", response.status());
                    }
                }
                Err(e) => {
                    println!("    Request error: {}", e);
                }
            }
            None
        });
    }

    let results: Vec<Biobrick> = join_all(futures)
        .await
        .into_iter()
        .filter_map(|r| r)
        .collect();

    println!("Found results from {} providers", results.len());

    if results.is_empty() {
        return None;
    }

    let mut final_biobrick = results[0].clone();

    for next_result in results.into_iter().skip(1) {
        final_biobrick = crate::merge::enrich(final_biobrick, next_result);
    }

    Some(final_biobrick)
}

fn normalize_id(id: &str) -> String {
    id.trim().to_lowercase()
}