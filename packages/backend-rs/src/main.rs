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

/// Library root: contains series folders, each containing volume folders
const LIBRARY_ROOT: &str = r"C:\dev\projects\bascan\library";

/// Embedded frontend build
static FRONTEND: Dir = include_dir!("$CARGO_MANIFEST_DIR/../frontend/build");

#[derive(Serialize)]
struct Series {
    id: String,
    title: String,
    #[serde(rename = "coverUrl")]
    cover_url: String,
    #[serde(rename = "volumeCount")]
    volume_count: usize,
}

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

/// Count volume directories in a series folder
async fn count_volumes_in(path: &std::path::Path) -> usize {
    let mut count = 0;
    if let Ok(mut entries) = fs::read_dir(path).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            if entry.file_type().await.map(|ft| ft.is_dir()).unwrap_or(false) {
                count += 1;
            }
        }
    }
    count
}

/// Get first image file in first volume of a series (for series cover)
async fn series_cover(series_path: &std::path::Path, series_id: &str) -> String {
    if let Ok(mut entries) = fs::read_dir(series_path).await {
        let mut vol_dirs: Vec<String> = Vec::new();
        while let Ok(Some(entry)) = entries.next_entry().await {
            if entry.file_type().await.map(|ft| ft.is_dir()).unwrap_or(false) {
                vol_dirs.push(entry.file_name().to_string_lossy().to_string());
            }
        }
        vol_dirs.sort_by(|a, b| natural_sort_key(a).cmp(&natural_sort_key(b)));
        if let Some(first_vol) = vol_dirs.first() {
            let vol_path = series_path.join(first_vol);
            if let Ok(mut files_entries) = fs::read_dir(&vol_path).await {
                let mut files = Vec::new();
                while let Ok(Some(f)) = files_entries.next_entry().await {
                    let fname = f.file_name().to_string_lossy().to_string();
                    if is_image(&fname) { files.push(fname); }
                }
                files.sort_by(|a, b| natural_sort_key(a).cmp(&natural_sort_key(b)));
                if let Some(cover) = files.first() {
                    return format!("/images/{}/{}/{}", 
                        urlencoding::encode(series_id),
                        urlencoding::encode(first_vol),
                        urlencoding::encode(cover));
                }
            }
        }
    }
    String::new()
}

// GET /api/series — list all series
async fn list_series() -> Json<Vec<Series>> {
    let mut series = Vec::new();
    let mut entries = fs::read_dir(LIBRARY_ROOT).await.unwrap();

    while let Ok(Some(entry)) = entries.next_entry().await {
        let name = entry.file_name().to_string_lossy().to_string();
        if !entry.file_type().await.map(|ft| ft.is_dir()).unwrap_or(false) { continue; }

        let path = PathBuf::from(LIBRARY_ROOT).join(&name);
        let vol_count = count_volumes_in(&path).await;
        let cover = series_cover(&path, &name).await;

        series.push(Series {
            title: name.clone(),
            cover_url: cover,
            volume_count: vol_count,
            id: name,
        });
    }

    series.sort_by(|a, b| natural_sort_key(&a.id).cmp(&natural_sort_key(&b.id)));
    Json(series)
}

// GET /api/series/:series_id/volumes — list volumes in a series
async fn list_volumes(Path(series_id): Path<String>) -> Result<Json<Vec<Volume>>, StatusCode> {
    let series_path = PathBuf::from(LIBRARY_ROOT).join(&series_id);
    if !series_path.is_dir() { return Err(StatusCode::NOT_FOUND); }

    let mut volumes = Vec::new();
    let mut entries = fs::read_dir(&series_path).await.map_err(|_| StatusCode::NOT_FOUND)?;

    while let Ok(Some(entry)) = entries.next_entry().await {
        let name = entry.file_name().to_string_lossy().to_string();
        if !entry.file_type().await.map(|ft| ft.is_dir()).unwrap_or(false) { continue; }

        let folder = series_path.join(&name);
        let mut files: Vec<String> = Vec::new();
        if let Ok(mut dir) = fs::read_dir(&folder).await {
            while let Ok(Some(f)) = dir.next_entry().await {
                let fname = f.file_name().to_string_lossy().to_string();
                if is_image(&fname) { files.push(fname); }
            }
        }
        files.sort_by(|a, b| natural_sort_key(a).cmp(&natural_sort_key(b)));

        let cover = files.first().cloned().unwrap_or_default();
        let encoded_series = urlencoding::encode(&series_id);
        let encoded_name = urlencoding::encode(&name);
        let encoded_cover = urlencoding::encode(&cover);

        volumes.push(Volume {
            cover_url: format!("/images/{}/{}/{}", encoded_series, encoded_name, encoded_cover),
            page_count: files.len(),
            title: name.clone(),
            id: name,
        });
    }

    volumes.sort_by(|a, b| natural_sort_key(&a.id).cmp(&natural_sort_key(&b.id)));
    Ok(Json(volumes))
}

// GET /api/series/:series_id/volumes/:volume_id/pages
async fn list_pages(Path((series_id, volume_id)): Path<(String, String)>) -> Result<Json<Vec<Page>>, StatusCode> {
    let folder = PathBuf::from(LIBRARY_ROOT).join(&series_id).join(&volume_id);
    if !folder.is_dir() { return Err(StatusCode::NOT_FOUND); }

    let mut files = Vec::new();
    let mut entries = fs::read_dir(&folder).await.map_err(|_| StatusCode::NOT_FOUND)?;
    while let Ok(Some(f)) = entries.next_entry().await {
        let fname = f.file_name().to_string_lossy().to_string();
        if is_image(&fname) { files.push(fname); }
    }
    files.sort_by(|a, b| natural_sort_key(a).cmp(&natural_sort_key(b)));

    let encoded_series = urlencoding::encode(&series_id);
    let encoded_vol = urlencoding::encode(&volume_id);
    let pages: Vec<Page> = files.iter().map(|f| {
        let encoded_file = urlencoding::encode(f);
        Page {
            filename: f.clone(),
            url: format!("/images/{}/{}/{}", encoded_series, encoded_vol, encoded_file),
            is_spread: is_spread(f),
        }
    }).collect();

    Ok(Json(pages))
}

// Serve images: /images/:series/:volume/:file
async fn serve_image(Path((series, volume, file)): Path<(String, String, String)>) -> impl IntoResponse {
    let path = PathBuf::from(LIBRARY_ROOT).join(&series).join(&volume).join(&file);

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

    if let Some(index) = FRONTEND.get_file("index.html") {
        return Html(std::str::from_utf8(index.contents()).unwrap_or("")).into_response();
    }

    StatusCode::NOT_FOUND.into_response()
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/api/series", get(list_series))
        .route("/api/series/{series_id}/volumes", get(list_volumes))
        .route("/api/series/{series_id}/volumes/{volume_id}/pages", get(list_pages))
        .route("/images/{series}/{volume}/{file}", get(serve_image))
        .fallback(get(serve_frontend))
        .layer(CorsLayer::permissive());

    let addr = "0.0.0.0:3001";
    println!("Bascan running on http://localhost:3001");
    println!("Library: {}", LIBRARY_ROOT);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
