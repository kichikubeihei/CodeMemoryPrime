use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs;
use std::path::Path;
use std::sync::LazyLock;

static PARAM_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\{[^}]+\}|:[a-zA-Z0-9_]+|\$\{[^}]+\}").unwrap()
});

static FETCH_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?:fetch|\$fetch|axios\.(?:get|post|put|delete|patch))\s*\(\s*[`'"]([^\s`'"\?\#]+)"#).unwrap()
});

static METHOD_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"method\s*:\s*['"]([A-Z]+)['"]"#).unwrap()
});

static LARAVEL_ROUTE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"Route::(get|post|put|delete|patch|options)\s*\(\s*['"]([^'"]+)['"]"#).unwrap()
});

static EXPRESS_ROUTE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?:app|router)\.(get|post|put|delete|patch)\s*\(\s*['"]([^'"]+)['"]"#).unwrap()
});

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RouteEndpoint {
    pub path: String,
    pub normalized_path: String,
    pub method: String,
    pub file_path: String,
    pub line_number: usize,
    pub framework: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FrontendCall {
    pub raw_url: String,
    pub normalized_path: String,
    pub method: String,
    pub file_path: String,
    pub line_number: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RouteBoundaryReport {
    pub status: String,
    pub workspace_root: String,
    pub total_frontend_calls: usize,
    pub total_backend_endpoints: usize,
    pub connected_routes_count: usize,
    pub orphaned_calls_count: usize,
    pub uncalled_endpoints_count: usize,
    pub connected_routes: Vec<Value>,
    pub orphaned_frontend_calls: Vec<FrontendCall>,
    pub uncalled_backend_endpoints: Vec<RouteEndpoint>,
}

pub fn normalize_route_pattern(route: &str) -> String {
    let stripped = route.trim().trim_start_matches('/');
    // Normalize path parameter syntax: {id}, :id, ${id} -> {param}
    let normalized = PARAM_RE.replace_all(stripped, "{param}").to_string();
    normalized.trim_end_matches('/').to_lowercase()
}

fn extract_cloudflare_endpoint(rel_path: &str, ext: &str) -> Option<RouteEndpoint> {
    if !rel_path.starts_with("functions/") || (ext != "ts" && ext != "js") {
        return None;
    }
    let without_prefix = rel_path.trim_start_matches("functions");
    let route_path = without_prefix.trim_end_matches(".ts").trim_end_matches(".js");
    let route_path = if route_path.ends_with("/index") {
        route_path.trim_end_matches("/index").to_string()
    } else {
        route_path.to_string()
    };

    Some(RouteEndpoint {
        path: route_path.clone(),
        normalized_path: normalize_route_pattern(&route_path),
        method: "ANY".to_string(),
        file_path: rel_path.to_string(),
        line_number: 1,
        framework: "Cloudflare Pages Functions".to_string(),
    })
}

fn extract_frontend_calls(content: &str, rel_path: &str, calls: &mut Vec<FrontendCall>) {
    for (line_idx, line) in content.lines().enumerate() {
        for cap in FETCH_RE.captures_iter(line) {
            let Some(m) = cap.get(1) else { continue; };
            let url = m.as_str();
            if !url.starts_with('/') && !url.starts_with("http://") && !url.starts_with("https://") && !url.starts_with("api/") {
                continue;
            }

            let clean_path = if let Some(idx) = url.find("://") {
                if let Some(path_start) = url[idx + 3..].find('/') {
                    &url[idx + 3 + path_start..]
                } else {
                    url
                }
            } else {
                url
            };

            let method = if let Some(m_cap) = METHOD_RE.captures(line) {
                m_cap.get(1).map(|s| s.as_str()).unwrap_or("GET").to_uppercase()
            } else if line.contains("axios.post") {
                "POST".to_string()
            } else if line.contains("axios.put") {
                "PUT".to_string()
            } else if line.contains("axios.delete") {
                "DELETE".to_string()
            } else {
                "GET".to_string()
            };

            calls.push(FrontendCall {
                raw_url: url.to_string(),
                normalized_path: normalize_route_pattern(clean_path),
                method,
                file_path: rel_path.to_string(),
                line_number: line_idx + 1,
            });
        }
    }
}

fn extract_laravel_routes(content: &str, rel_path: &str, endpoints: &mut Vec<RouteEndpoint>) {
    let is_api_file = rel_path.contains("api.php");
    for (line_idx, line) in content.lines().enumerate() {
        for cap in LARAVEL_ROUTE_RE.captures_iter(line) {
            let method = cap.get(1).map(|s| s.as_str()).unwrap_or("GET").to_uppercase();
            if let Some(r_cap) = cap.get(2) {
                let mut r_path = r_cap.as_str().to_string();
                if is_api_file && !r_path.starts_with("api/") && !r_path.starts_with("/api/") {
                    r_path = format!("api/{}", r_path.trim_start_matches('/'));
                }
                endpoints.push(RouteEndpoint {
                    path: r_path.clone(),
                    normalized_path: normalize_route_pattern(&r_path),
                    method,
                    file_path: rel_path.to_string(),
                    line_number: line_idx + 1,
                    framework: "Laravel Route".to_string(),
                });
            }
        }
    }
}

fn extract_express_routes(content: &str, rel_path: &str, endpoints: &mut Vec<RouteEndpoint>) {
    for (line_idx, line) in content.lines().enumerate() {
        for cap in EXPRESS_ROUTE_RE.captures_iter(line) {
            let method = cap.get(1).map(|s| s.as_str()).unwrap_or("GET").to_uppercase();
            if let Some(r_cap) = cap.get(2) {
                let r_path = r_cap.as_str();
                endpoints.push(RouteEndpoint {
                    path: r_path.to_string(),
                    normalized_path: normalize_route_pattern(r_path),
                    method,
                    file_path: rel_path.to_string(),
                    line_number: line_idx + 1,
                    framework: "Express Route".to_string(),
                });
            }
        }
    }
}

fn correlate_route_boundaries(
    frontend_calls: Vec<FrontendCall>,
    backend_endpoints: Vec<RouteEndpoint>,
    root_path: &str,
) -> RouteBoundaryReport {
    let mut connected_routes = Vec::new();
    let mut orphaned_calls = Vec::new();
    let mut matched_endpoint_indices = std::collections::HashSet::new();

    for call in &frontend_calls {
        let mut matched = false;
        for (idx, ep) in backend_endpoints.iter().enumerate() {
            if call.normalized_path == ep.normalized_path
                || call.normalized_path.ends_with(&ep.normalized_path)
                || ep.normalized_path.ends_with(&call.normalized_path)
            {
                matched = true;
                matched_endpoint_indices.insert(idx);
                connected_routes.push(json!({
                    "url": call.raw_url,
                    "frontend": format!("{}:{}", call.file_path, call.line_number),
                    "backend": format!("{}:{} ({})", ep.file_path, ep.line_number, ep.framework),
                    "method": call.method,
                    "status": "CONNECTED"
                }));
                break;
            }
        }
        if !matched {
            orphaned_calls.push(call.clone());
        }
    }

    let uncalled_endpoints: Vec<RouteEndpoint> = backend_endpoints
        .into_iter()
        .enumerate()
        .filter(|(idx, _)| !matched_endpoint_indices.contains(idx))
        .map(|(_, ep)| ep)
        .collect();

    RouteBoundaryReport {
        status: "success".to_string(),
        workspace_root: root_path.to_string(),
        total_frontend_calls: frontend_calls.len(),
        total_backend_endpoints: matched_endpoint_indices.len() + uncalled_endpoints.len(),
        connected_routes_count: connected_routes.len(),
        orphaned_calls_count: orphaned_calls.len(),
        uncalled_endpoints_count: uncalled_endpoints.len(),
        connected_routes,
        orphaned_frontend_calls: orphaned_calls,
        uncalled_backend_endpoints: uncalled_endpoints,
    }
}

pub fn scan_workspace_routes(root_path: &str) -> RouteBoundaryReport {
    let root = Path::new(root_path);
    let mut frontend_calls = Vec::new();
    let mut backend_endpoints = Vec::new();

    let mut dirs_to_visit = vec![root.to_path_buf()];
    while let Some(dir) = dirs_to_visit.pop() {
        let Ok(entries) = fs::read_dir(&dir) else { continue; };
        for entry in entries.flatten() {
            let path = entry.path();
            let file_name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");

            if file_name.starts_with('.')
                || file_name == "node_modules"
                || file_name == "target"
                || file_name == "vendor"
                || file_name == "dist"
                || file_name == "build"
                || file_name == "out"
            {
                continue;
            }

            if path.is_dir() {
                dirs_to_visit.push(path);
                continue;
            }

            let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
            let rel_path = path.strip_prefix(root).unwrap_or(&path).to_string_lossy().to_string();

            if let Some(cf_ep) = extract_cloudflare_endpoint(&rel_path, &ext) {
                backend_endpoints.push(cf_ep);
            }

            let Ok(content) = fs::read_to_string(&path) else { continue; };

            if ["svelte", "vue", "ts", "tsx", "js", "jsx", "html"].contains(&ext.as_str()) && !rel_path.starts_with("functions/") {
                extract_frontend_calls(&content, &rel_path, &mut frontend_calls);
            }

            if ext == "php" && rel_path.contains("routes/") {
                extract_laravel_routes(&content, &rel_path, &mut backend_endpoints);
            }

            if (ext == "js" || ext == "ts") && (rel_path.contains("server/") || rel_path.contains("routes/")) {
                extract_express_routes(&content, &rel_path, &mut backend_endpoints);
            }
        }
    }

    correlate_route_boundaries(frontend_calls, backend_endpoints, root_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_route_pattern() {
        assert_eq!(normalize_route_pattern("/api/raids"), "api/raids");
        assert_eq!(normalize_route_pattern("/api/raids/{id}"), "api/raids/{param}");
        assert_eq!(normalize_route_pattern("api/raids/:id"), "api/raids/{param}");
        assert_eq!(normalize_route_pattern("/api/users/${userId}/profile"), "api/users/{param}/profile");
    }
}
