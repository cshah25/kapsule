use serde::{Deserialize, Serialize};
use reqwest::Client;
use tauri::{State, ipc::Channel};
use bollard::image::CreateImageOptions;
use futures_util::StreamExt;
use std::sync::OnceLock;
use std::time::Duration;
use crate::AppState;

/// Shared HTTP client for Docker Hub requests (keeps the connection pool warm).
fn hub_client() -> &'static Client {
    static CLIENT: OnceLock<Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        Client::builder()
            .timeout(Duration::from_secs(15))
            .build()
            .expect("failed to build HTTP client")
    })
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SearchResult {
    pub repo_name: String,
    pub short_description: String,
    pub star_count: u32,
    pub is_official: bool,
}

#[derive(Deserialize, Debug)]
struct DockerHubSearchResponse {
    results: Vec<DockerHubSearchResult>,
}

#[derive(Deserialize, Debug)]
struct DockerHubSearchResult {
    repo_name: String,
    // Hub returns null for repositories without a description
    #[serde(default)]
    short_description: Option<String>,
    star_count: u32,
    is_official: bool,
}

#[tauri::command]
pub async fn search_images(query: String) -> Result<Vec<SearchResult>, String> {
    if query.is_empty() {
        return Ok(vec![]);
    }
    
    let res = hub_client()
        .get("https://hub.docker.com/v2/search/repositories/")
        .query(&[("query", query.as_str()), ("page_size", "24")])
        .send()
        .await
        .map_err(|e| e.to_string())?;
    
    if !res.status().is_success() {
        return Err(format!("Search failed: {}", res.status()));
    }
    
    let data: DockerHubSearchResponse = res.json().await.map_err(|e| e.to_string())?;
    
    let mut results = vec![];
    for r in data.results {
        results.push(SearchResult {
            repo_name: r.repo_name,
            short_description: r.short_description.unwrap_or_default(),
            star_count: r.star_count,
            is_official: r.is_official,
        });
    }
    
    Ok(results)
}

#[derive(Deserialize, Debug)]
struct DockerHubTagsResponse {
    results: Vec<DockerHubTag>,
}

#[derive(Deserialize, Debug)]
struct DockerHubTag {
    name: String,
}

#[tauri::command]
pub async fn get_image_tags(image: String) -> Result<Vec<String>, String> {
    let mut parts = image.split('/');
    let mut namespace = "library";
    let repo;
    
    let first = parts.next().unwrap_or_default();
    if let Some(second) = parts.next() {
        namespace = first;
        repo = second;
    } else {
        repo = first;
    }
    
    let url = format!("https://hub.docker.com/v2/repositories/{}/{}/tags?page_size=100", namespace, repo);
    let res = hub_client().get(&url).send().await.map_err(|e| e.to_string())?;
    
    if !res.status().is_success() {
        return Err(format!("Failed to fetch tags: {}", res.status()));
    }
    
    let data: DockerHubTagsResponse = res.json().await.map_err(|e| e.to_string())?;
    let tags = data.results.into_iter().map(|t| t.name).collect();
    
    Ok(tags)
}

#[derive(Serialize, Clone)]
pub struct PullProgress {
    pub id: Option<String>,
    pub status: String,
    pub progress: String,
    pub current: Option<i64>,
    pub total: Option<i64>,
}

/// Pulls an image, streaming progress over `on_message`. Resolves once the
/// pull has finished, or rejects with the engine's error so the UI can tell
/// a failed pull from a completed one.
#[tauri::command]
pub async fn pull_image(
    state: State<'_, AppState>,
    image: String,
    on_message: Channel<PullProgress>,
) -> Result<(), String> {
    let docker = state.get_client().await?;

    let mut stream = docker.create_image(Some(CreateImageOptions {
        from_image: image,
        ..Default::default()
    }), None, None);

    while let Some(info_result) = stream.next().await {
        // Registry errors (missing tag, auth, …) arrive here as DockerStreamError
        let info = info_result.map_err(|e| format!("Pull failed: {}", e))?;
        let (current, total) = info
            .progress_detail
            .map(|d| (d.current, d.total))
            .unwrap_or((None, None));
        let progress = PullProgress {
            id: info.id,
            status: info.status.unwrap_or_default(),
            progress: info.progress.unwrap_or_default(),
            current,
            total,
        };
        // The window went away — nobody is watching, but let the pull finish
        let _ = on_message.send(progress);
    }

    Ok(())
}
