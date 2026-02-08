//! Web API module for FSID HTTP server

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tiny_http::{Header, Method, Request, Response, Server};

use crate::storage::FsidStorage;
use crate::{is_short_format, minimal, short, standard};

/// API request body
#[derive(Deserialize)]
struct ApiRequest {
    /// Action: "to", "from", "info", "list"
    action: String,
    /// Path for "to" action
    path: Option<String>,
    /// FSID for "from" and "info" actions
    fsid: Option<String>,
    /// Format: "standard", "short", "min"
    format: Option<String>,
}

/// API response
#[derive(Serialize)]
struct ApiResponse {
    success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    fsid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    info: Option<FsidInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    list: Option<Vec<ListEntry>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

/// FSID info response
#[derive(Serialize)]
struct FsidInfo {
    fsid: String,
    format: String,
    prefix: String,
    prefix_code: String,
    file_type: String,
    mode: String,
    valid: bool,
    path: Option<String>,
}

/// List entry
#[derive(Serialize)]
struct ListEntry {
    fsid: String,
    path: String,
}

/// Check if FSID is minimal format
fn is_minimal_format(fsid: &str) -> bool {
    fsid.len() == 13 && fsid.chars().all(|c| c.is_ascii_digit())
}

/// Handle API request
fn handle_request(req: &ApiRequest, storage_path: &PathBuf) -> ApiResponse {
    match req.action.as_str() {
        "to" => {
            let Some(ref path) = req.path else {
                return ApiResponse {
                    success: false,
                    fsid: None,
                    path: None,
                    info: None,
                    list: None,
                    error: Some("Missing 'path' field".to_string()),
                };
            };

            let path_obj = std::path::Path::new(path);
            if !path_obj.exists() && !path_obj.is_symlink() {
                return ApiResponse {
                    success: false,
                    fsid: None,
                    path: None,
                    info: None,
                    list: None,
                    error: Some(format!("Path does not exist: {}", path)),
                };
            }

            let format = req.format.as_deref().unwrap_or("standard");
            let fsid = match format {
                "short" => short::generate(path),
                "min" | "minimal" => {
                    let fsid = minimal::generate(path);
                    // Store minimal FSIDs
                    let mut store = FsidStorage::load(storage_path);
                    let canonical = std::fs::canonicalize(path_obj)
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_else(|_| path.clone());
                    store.insert(fsid.clone(), canonical, storage_path);
                    fsid
                }
                _ => standard::generate(path),
            };

            // Optionally store all FSIDs
            if format != "min" && format != "minimal" {
                let mut store = FsidStorage::load(storage_path);
                let canonical = std::fs::canonicalize(path_obj)
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|_| path.clone());
                store.insert(fsid.clone(), canonical, storage_path);
            }

            ApiResponse {
                success: true,
                fsid: Some(fsid),
                path: None,
                info: None,
                list: None,
                error: None,
            }
        }

        "from" => {
            let Some(ref fsid) = req.fsid else {
                return ApiResponse {
                    success: false,
                    fsid: None,
                    path: None,
                    info: None,
                    list: None,
                    error: Some("Missing 'fsid' field".to_string()),
                };
            };

            let fsid_lower = fsid.to_lowercase();

            // Try storage first
            let store = FsidStorage::load(storage_path);
            if let Some(path) = store.get(fsid).or_else(|| store.get(&fsid_lower)) {
                return ApiResponse {
                    success: true,
                    fsid: None,
                    path: Some(path.clone()),
                    info: None,
                    list: None,
                    error: None,
                };
            }

            // Decode based on format
            let decoded = if is_minimal_format(fsid) {
                None // Minimal requires storage
            } else if is_short_format(&fsid_lower) {
                short::decode(&fsid_lower)
            } else {
                standard::decode(fsid)
            };

            match decoded {
                Some(path) => ApiResponse {
                    success: true,
                    fsid: None,
                    path: Some(path),
                    info: None,
                    list: None,
                    error: None,
                },
                None => ApiResponse {
                    success: false,
                    fsid: None,
                    path: None,
                    info: None,
                    list: None,
                    error: Some("Failed to decode FSID".to_string()),
                },
            }
        }

        "info" => {
            let Some(ref fsid) = req.fsid else {
                return ApiResponse {
                    success: false,
                    fsid: None,
                    path: None,
                    info: None,
                    list: None,
                    error: Some("Missing 'fsid' field".to_string()),
                };
            };

            let fsid_lower = fsid.to_lowercase();
            let store = FsidStorage::load(storage_path);
            let stored_path = store.get(fsid).or_else(|| store.get(&fsid_lower)).cloned();

            let info = if is_minimal_format(fsid) {
                minimal::parse(fsid).map(|i| FsidInfo {
                    fsid: fsid.clone(),
                    format: "minimal".to_string(),
                    prefix: i.prefix_path,
                    prefix_code: i.prefix_code,
                    file_type: crate::constants::get_file_type_name(i.file_type_code).to_string(),
                    mode: format!("{} ({:o})", i.mode_symbolic, i.mode_octal),
                    valid: i.valid,
                    path: stored_path,
                })
            } else if is_short_format(&fsid_lower) {
                let prefix_code = &fsid_lower[0..2];
                let type_mode_code = fsid_lower.chars().nth(2).unwrap_or('z');
                let prefix_path = short::get_prefix_path(prefix_code).unwrap_or("/");
                let (file_type, mode, _) = short::get_type_mode_info(type_mode_code);
                let valid = short::validate(&fsid_lower);
                let decoded = short::decode(&fsid_lower);

                Some(FsidInfo {
                    fsid: fsid_lower.clone(),
                    format: "short".to_string(),
                    prefix: prefix_path.to_string(),
                    prefix_code: prefix_code.to_string(),
                    file_type: crate::constants::get_file_type_name(file_type).to_string(),
                    mode: format!("{:o}", mode),
                    valid,
                    path: stored_path.or(decoded),
                })
            } else {
                let prefix_code = &fsid[0..2];
                let file_type_code = fsid[2..3].parse::<u8>().unwrap_or(0);
                let mode_code = fsid[3..4].parse::<u8>().unwrap_or(9);
                let prefix_path = standard::get_prefix_path(prefix_code).unwrap_or("/");
                let (mode_symbolic, mode_octal) = standard::get_mode_description(mode_code);
                let valid = standard::validate(fsid);
                let decoded = standard::decode(fsid);

                Some(FsidInfo {
                    fsid: fsid.clone(),
                    format: "standard".to_string(),
                    prefix: prefix_path.to_string(),
                    prefix_code: prefix_code.to_string(),
                    file_type: crate::constants::get_file_type_name(file_type_code).to_string(),
                    mode: format!("{} ({:o})", mode_symbolic, mode_octal),
                    valid,
                    path: stored_path.or(decoded),
                })
            };

            match info {
                Some(i) => ApiResponse {
                    success: true,
                    fsid: None,
                    path: None,
                    info: Some(i),
                    list: None,
                    error: None,
                },
                None => ApiResponse {
                    success: false,
                    fsid: None,
                    path: None,
                    info: None,
                    list: None,
                    error: Some("Invalid FSID format".to_string()),
                },
            }
        }

        "list" => {
            let store = FsidStorage::load(storage_path);
            let list: Vec<ListEntry> = store
                .mappings
                .iter()
                .map(|(fsid, path)| ListEntry {
                    fsid: fsid.clone(),
                    path: path.clone(),
                })
                .collect();

            ApiResponse {
                success: true,
                fsid: None,
                path: None,
                info: None,
                list: Some(list),
                error: None,
            }
        }

        _ => ApiResponse {
            success: false,
            fsid: None,
            path: None,
            info: None,
            list: None,
            error: Some(format!("Unknown action: {}", req.action)),
        },
    }
}

/// Start the web API server
pub fn start_server(addr: &str, storage_path: PathBuf) {
    let server = match Server::http(addr) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to start server: {}", e);
            std::process::exit(1);
        }
    };

    println!("FSID Web API running on http://{}", addr);
    println!("Storage: {}", storage_path.display());
    println!();
    println!("POST / with JSON body:");
    println!("  {{\"action\": \"to\", \"path\": \"/etc/passwd\", \"format\": \"standard|short|min\"}}");
    println!("  {{\"action\": \"from\", \"fsid\": \"0100893316018\"}}");
    println!("  {{\"action\": \"info\", \"fsid\": \"0100893316018\"}}");
    println!("  {{\"action\": \"list\"}}");

    for mut request in server.incoming_requests() {
        let response = process_request(&mut request, &storage_path);
        let _ = request.respond(response);
    }
}

fn process_request(request: &mut Request, storage_path: &PathBuf) -> Response<std::io::Cursor<Vec<u8>>> {
    let cors = Header::from_bytes("Access-Control-Allow-Origin", "*").unwrap();

    // Handle CORS preflight
    if request.method() == &Method::Options {
        let allow_methods = Header::from_bytes("Access-Control-Allow-Methods", "GET, POST, OPTIONS").unwrap();
        let allow_headers = Header::from_bytes("Access-Control-Allow-Headers", "Content-Type").unwrap();
        return Response::from_string("")
            .with_header(cors)
            .with_header(allow_methods)
            .with_header(allow_headers);
    }

    // Serve static files on GET
    if request.method() == &Method::Get {
        let url = request.url();
        
        if url == "/style.css" {
            let css = include_str!("../web/style.css");
            let content_type = Header::from_bytes("Content-Type", "text/css; charset=utf-8").unwrap();
            return Response::from_string(css)
                .with_header(content_type)
                .with_header(cors);
        }
        
        if url == "/script.js" {
            let js = include_str!("../web/script.js");
            let content_type = Header::from_bytes("Content-Type", "application/javascript; charset=utf-8").unwrap();
            return Response::from_string(js)
                .with_header(content_type)
                .with_header(cors);
        }
        
        // Default: serve HTML
        let html = include_str!("../web/index.html");
        let content_type = Header::from_bytes("Content-Type", "text/html; charset=utf-8").unwrap();
        return Response::from_string(html)
            .with_header(content_type)
            .with_header(cors);
    }

    let content_type = Header::from_bytes("Content-Type", "application/json").unwrap();

    // Only accept POST for API
    if request.method() != &Method::Post {
        let error = serde_json::json!({"success": false, "error": "Method not allowed. Use GET for UI, POST for API."});
        return Response::from_string(error.to_string())
            .with_status_code(405)
            .with_header(content_type)
            .with_header(cors);
    }

    // Read body
    let mut body = String::new();
    if let Err(e) = request.as_reader().read_to_string(&mut body) {
        let error = serde_json::json!({"success": false, "error": format!("Failed to read body: {}", e)});
        return Response::from_string(error.to_string())
            .with_status_code(400)
            .with_header(content_type)
            .with_header(cors);
    }

    // Parse JSON
    let api_request: ApiRequest = match serde_json::from_str(&body) {
        Ok(r) => r,
        Err(e) => {
            let error = serde_json::json!({"success": false, "error": format!("Invalid JSON: {}", e)});
            return Response::from_string(error.to_string())
                .with_status_code(400)
                .with_header(content_type)
                .with_header(cors);
        }
    };

    // Handle request
    let response = handle_request(&api_request, storage_path);
    let json = serde_json::to_string(&response).unwrap_or_else(|_| r#"{"success":false,"error":"Serialization error"}"#.to_string());

    Response::from_string(json)
        .with_header(content_type)
        .with_header(cors)
}
