use std::fs;
use std::path::Path;
use regex::Regex;

#[derive(Debug, Clone)]
pub struct DocBlock {
    pub title: String,
    pub url: String,
    pub content: String,
}

pub fn parse_local_markdown_docs(folder_path: &str) -> Vec<DocBlock> {
    let mut chunks = Vec::new();
    let header_re = Regex::new(r"(?m)^(#+)\s+(.*?)$").unwrap();
    
    fn walk_dir(dir: &Path, chunks: &mut Vec<DocBlock>, header_re: &Regex) {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    walk_dir(&path, chunks, header_re);
                } else if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                    if ["md", "markdown", "txt"].contains(&ext.to_lowercase().as_str()) {
                        if let Ok(content) = fs::read_to_string(&path) {
                            let file_name = path.file_name().and_then(|s| s.to_str()).unwrap_or("file");
                            let file_url = format!("file://{}", path.to_string_lossy());
                            
                            let mut matches: Vec<(usize, usize, &str, &str)> = Vec::new();
                            for cap in header_re.captures_iter(&content) {
                                let mat = cap.get(0).unwrap();
                                let level = cap.get(1).unwrap().as_str();
                                let title = cap.get(2).unwrap().as_str().trim();
                                matches.push((mat.start(), mat.end(), level, title));
                            }
                            
                            if matches.is_empty() {
                                if !content.trim().is_empty() {
                                    chunks.push(DocBlock {
                                        title: file_name.to_string(),
                                        url: file_url,
                                        content: content.trim().to_string(),
                                    });
                                }
                                continue;
                            }
                            
                            for idx in 0..matches.len() {
                                let (_, start_idx, _, title) = matches[idx];
                                let end_idx = if idx + 1 < matches.len() {
                                    matches[idx + 1].0
                                } else {
                                    content.len()
                                };
                                
                                let section_content = content[start_idx..end_idx].trim().to_string();
                                if section_content.len() > 30 {
                                    chunks.push(DocBlock {
                                        title: format!("{} - {}", file_name, title),
                                        url: file_url.clone(),
                                        content: section_content,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    walk_dir(Path::new(folder_path), &mut chunks, &header_re);
    chunks
}

pub async fn scrape_web_docs(url: &str) -> Vec<DocBlock> {
    let client = reqwest::Client::builder()
        .user_agent("mcp-coder-memory-rust/0.1.0")
        .build()
        .unwrap_or_default();
        
    let response = match client.get(url).send().await {
        Ok(res) => res,
        Err(e) => {
            println!("[Scraper Error] Failed to fetch url {}: {}", url, e);
            return Vec::new();
        }
    };
    
    let html = match response.text().await {
        Ok(txt) => txt,
        Err(e) => {
            println!("[Scraper Error] Failed to get response text: {}", e);
            return Vec::new();
        }
    };
    
    let script_style_re = Regex::new(r"(?s)<(script|style|nav|footer|header|head|aside).*?>.*?<\/\1>").unwrap();
    let clean_html = script_style_re.replace_all(&html, "").to_string();
    
    let element_re = Regex::new(r"(?s)<(h1|h2|h3|h4|h5|h6|p|li|pre|code|div).*?>(.*?)<\/\1>").unwrap();
    let strip_tags_re = Regex::new(r"<[^>]*>").unwrap();
    
    let mut chunks = Vec::new();
    let mut current_header = "Documentation Page".to_string();
    
    for cap in element_re.captures_iter(&clean_html) {
        let tag = cap.get(1).unwrap().as_str();
        let inner_html = cap.get(2).unwrap().as_str();
        
        let cleaned_text = strip_tags_re.replace_all(inner_html, "").to_string();
        let txt = cleaned_text.trim();
        if txt.is_empty() || txt.len() <= 20 {
            continue;
        }
        
        let whitespace_re = Regex::new(r"\s+").unwrap();
        let clean_txt = whitespace_re.replace_all(txt, " ").to_string();
        
        if tag.starts_with('h') {
            current_header = clean_txt.clone();
        } else {
            chunks.push(DocBlock {
                title: current_header.clone(),
                url: url.to_string(),
                content: clean_txt,
            });
        }
    }
    
    chunks
}
