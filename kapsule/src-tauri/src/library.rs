use serde::{Deserialize, Serialize};
use reqwest::Client;
use tauri::{State, ipc::Channel};
use bollard::image::CreateImageOptions;
use futures_util::StreamExt;
use crate::AppState;

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
    short_description: String,
    star_count: u32,
    is_official: bool,
}

#[tauri::command]
pub async fn search_images(query: String) -> Result<Vec<SearchResult>, String> {
    if query.is_empty() {
        return Ok(vec![]);
    }
    
    let url = format!("https://hub.docker.com/v2/search/repositories/?query={}&page_size=24", query);
    let client = Client::new();
    let res = client.get(&url).send().await.map_err(|e| e.to_string())?;
    
    if !res.status().is_success() {
        return Err(format!("Search failed: {}", res.status()));
    }
    
    let data: DockerHubSearchResponse = res.json().await.map_err(|e| e.to_string())?;
    
    let mut results = vec![];
    for r in data.results {
        results.push(SearchResult {
            repo_name: r.repo_name,
            short_description: r.short_description,
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
    let client = Client::new();
    let res = client.get(&url).send().await.map_err(|e| e.to_string())?;
    
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

#[tauri::command]
pub async fn pull_image(
    state: State<'_, AppState>,
    image: String,
    on_message: Channel<PullProgress>,
) -> Result<(), String> {
    let engine = state.active_engine.lock().await.clone().ok_or("No active engine")?;
    let docker = crate::engine::connect(engine).await.ok_or("Failed to connect")?;

    tauri::async_runtime::spawn(async move {
        let mut stream = docker.create_image(Some(CreateImageOptions {
            from_image: image.clone(),
            ..Default::default()
        }), None, None);

        while let Some(info_result) = stream.next().await {
            match info_result {
                Ok(info) => {
                    let id = info.id.clone();
                    let status = info.status.clone().unwrap_or_default();
                    let progress = info.progress.clone().unwrap_or_default();
                    let (mut current, mut total) = (None, None);
                    if let Some(detail) = info.progress_detail {
                        current = detail.current;
                        total = detail.total;
                    }
                    if on_message.send(PullProgress { id, status, progress, current, total }).is_err() {
                        break;
                    }
                }
                Err(e) => {
                    let _ = on_message.send(PullProgress { id: None, status: format!("Error: {}", e), progress: "".to_string(), current: None, total: None });
                    break;
                }
            }
        }
        
        let _ = on_message.send(PullProgress { id: None, status: "Done".to_string(), progress: "".to_string(), current: None, total: None });
    });

    Ok(())
}
