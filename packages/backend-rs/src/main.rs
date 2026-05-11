use axum::{
    extract::Path,
    http::{header, StatusCode, Uri},
    response::{Html, IntoResponse, Json, Response},
    routing::get,
    Router,
};
use include_dir::{include_dir, Dir};
use serde::Serialize;
use std::path::PathBuf;
use tokio::fs;
use tower_http::cors::CorsLayer;

const SCANS_ROOT: &str = r"C:\dev\projects\bascan\berserk";

/// Embedded frontend build
static FRONTEND: Dir = include_dir!("$CARGO_MANIFEST_DIR/../frontend/build");

#[derive(Serialize)]
struct Volume {
    id: String,
    title: String,
    #[serde(rename = "coverUrl")]
    cover_url: String,
    #[serde(rename = "pageCount")]
    page_count: usize,
}

#[derive(Serialize)]
struct Page {
    filename: String,
    url: String,
    #[serde(rename = "isSpread")]
    is_spread: bool,
}

fn is_image(name: &str) -> bool {
    let lower = name.to_lowercase();
    lower.ends_with(".jpg") || lower.ends_with(".jpeg") || lower.ends_with(".png") || lower.ends_with(".webp")
}

fn is_spread(name: &str) -> bool {
    // Match patterns like 006-007.jpg
    let stem = name.rsplit_once('.').map(|(s, _)| s).unwrap_or(name);
    stem.contains('-') && stem.split('-').all(|p| p.chars().all(|c| c.is_ascii_digit()))
}

fn natural_sort_key(s: &str) -> Vec<u64> {
    let mut result = Vec::new();
    let mut num_buf = String::new();
    for c in s.chars() {
        if c.is_ascii_digit() {
            num_buf.push(c);
        } else {
            if !num_buf.is_empty() {
                result.push(num_buf.parse().unwrap_or(0));
                num_buf.clear();
            }
            result.push(c as u64 + 1_000_000);
        }
    }
    if !num_buf.is_empty() {
        result.push(num_buf.parse().unwrap_or(0));
    }
    result
}

async fn list_volumes() -> Json<Vec<Volume>> {
    let mut volumes = Vec::new();
    let mut entries = fs::read_dir(SCANS_ROOT).await.unwrap();

    while let Ok(Some(entry)) = entries.next_entry().await {
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.starts_with("Berserk T") { continue; }
        if !entry.file_type().await.map(|ft| ft.is_dir()).unwrap_or(false) { continue; }

        let folder = PathBuf::from(SCANS_ROOT).join(&name);
        let mut files: Vec<String> = Vec::new();
        if let Ok(mut dir) = fs::read_dir(&folder).await {
            while let Ok(Some(f)) = dir.next_entry().await {
                let fname = f.file_name().to_string_lossy().to_string();
                if is_image(&fname) { files.push(fname); }
            }
        }
        files.sort_by(|a, b| natural_sort_key(a).cmp(&natural_sort_key(b)));

        let cover = files.first().cloned().unwrap_or_default();
        let encoded_name = urlencoding::encode(&name);
        let encoded_cover = urlencoding::encode(&cover);

        volumes.push(Volume {
            cover_url: format!("/images/{}/{}", encoded_name, encoded_cover),
            page_count: files.len(),
            title: name.clone(),
            id: name,
        });
    }

    volumes.sort_by(|a, b| natural_sort_key(&a.id).cmp(&natural_sort_key(&b.id)));
    Json(volumes)
}

async fn list_pages(Path(id): Path<String>) -> Result<Json<Vec<Page>>, StatusCode> {
    let folder = PathBuf::from(SCANS_ROOT).join(&id);
    if !folder.is_dir() { return Err(StatusCode::NOT_FOUND); }

    let mut files = Vec::new();
    let mut entries = fs::read_dir(&folder).await.map_err(|_| StatusCode::NOT_FOUND)?;
    while let Ok(Some(f)) = entries.next_entry().await {
        let fname = f.file_name().to_string_lossy().to_string();
        if is_image(&fname) { files.push(fname); }
    }
    files.sort_by(|a, b| natural_sort_key(a).cmp(&natural_sort_key(b)));

    let encoded_id = urlencoding::encode(&id);
    let pages: Vec<Page> = files.iter().map(|f| {
        let encoded_file = urlencoding::encode(f);
        Page {
            filename: f.clone(),
            url: format!("/images/{}/{}", encoded_id, encoded_file),
            is_spread: is_spread(f),
        }
    }).collect();

    Ok(Json(pages))
}

async fn serve_image(Path((volume, file)): Path<(String, String)>) -> impl IntoResponse {
    let path = PathBuf::from(SCANS_ROOT).join(&volume).join(&file);

    match fs::read(&path).await {
        Ok(bytes) => (
            StatusCode::OK,
            [
                (header::CONTENT_TYPE, "image/jpeg"),
                (header::CACHE_CONTROL, "public, max-age=31536000"),
            ],
            bytes,
        ).into_response(),
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

/// Serve embedded frontend files, with SPA fallback to index.html
async fn serve_frontend(uri: Uri) -> Response {
    let path = uri.path().trim_start_matches('/');
    
    // Try to serve the exact file
    if let Some(file) = FRONTEND.get_file(path) {
        let mime = match path.rsplit_once('.').map(|(_, ext)| ext) {
            Some("html") => "text/html",
            Some("js") => "application/javascript",
            Some("css") => "text/css",
            Some("svg") => "image/svg+xml",
            Some("json") => "application/json",
            Some("woff2") => "font/woff2",
            Some("woff") => "font/woff",
            _ => "application/octet-stream",
        };
        return (
            StatusCode::OK,
            [(header::CONTENT_TYPE, mime), (header::CACHE_CONTROL, "public, max-age=3600")],
            file.contents(),
        ).into_response();
    }

    // SPA fallback — serve index.html for any unmatched route
    if let Some(index) = FRONTEND.get_file("index.html") {
        return Html(std::str::from_utf8(index.contents()).unwrap_or("")).into_response();
    }

    StatusCode::NOT_FOUND.into_response()
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/api/volumes", get(list_volumes))
        .route("/api/volumes/{id}/pages", get(list_pages))
        .route("/images/{volume}/{file}", get(serve_image))
        .fallback(get(serve_frontend))
        .layer(CorsLayer::permissive());

    let addr = "0.0.0.0:3001";
    println!("Bascan running on http://localhost:3001");
    println!("Serving scans from: {}", SCANS_ROOT);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
