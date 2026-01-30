use futures::future::join_all;
use crate::types::Biobrick;
use crate::providers::get_all_providers;

pub async fn meta_search(id: &str) -> Option<Biobrick> {
    println!("Searching for part: {}", id);
    
    let providers = get_all_providers();
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .unwrap();
    
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
                        let parsed = provider.parse(id, &text);
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