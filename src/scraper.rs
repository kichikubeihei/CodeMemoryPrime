use std::fs;
use std::path::Path;
use regex::Regex;
use pulldown_cmark::{Parser, Event, Tag, TagEnd};
use scraper::{Html, Selector};

#[derive(Debug, Clone)]
pub struct DocBlock {
    pub title: String,
    pub url: String,
    pub content: String,
    pub links: Vec<(String, String)>,
}

pub fn parse_local_markdown_docs(folder_path: &str) -> Vec<DocBlock> {
    let mut chunks = Vec::new();
    
    fn walk_dir(dir: &Path, chunks: &mut Vec<DocBlock>) {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    walk_dir(&path, chunks);
                } else if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                    if ["md", "markdown", "txt"].contains(&ext.to_lowercase().as_str()) {
                        if let Ok(content) = fs::read_to_string(&path) {
                            let file_name = path.file_name().and_then(|s| s.to_str()).unwrap_or("file");
                            let file_url = format!("file://{}", path.to_string_lossy());
                            
                            let mut current_content = String::new();
                            let mut current_links = Vec::new();
                            let mut breadcrumbs = vec![file_name.to_string()];
                            let mut in_heading = false;
                            let mut heading_text = String::new();
                            let mut heading_level = 1;
                            
                            for event in Parser::new(&content) {
                                match event {
                                    Event::Start(Tag::Heading { level, .. }) => {
                                        let trimmed = current_content.trim();
                                        if trimmed.len() > 30 {
                                            chunks.push(DocBlock {
                                                title: breadcrumbs.join(" > "),
                                                url: file_url.clone(),
                                                content: trimmed.to_string(),
                                                links: std::mem::take(&mut current_links),
                                            });
                                        }
                                        current_content.clear();
                                        in_heading = true;
                                        heading_text.clear();
                                        heading_level = level as usize;
                                    }
                                    Event::End(TagEnd::Heading(..)) => {
                                        in_heading = false;
                                        breadcrumbs.truncate(heading_level);
                                        breadcrumbs.push(heading_text.trim().to_string());
                                    }
                                    Event::Start(Tag::Link { dest_url, title, .. }) => {
                                        current_links.push((title.to_string(), dest_url.to_string()));
                                    }
                                    Event::Text(t) | Event::Code(t) => {
                                        if in_heading {
                                            heading_text.push_str(&t);
                                        } else {
                                            current_content.push_str(&t);
                                            current_content.push('\n');
                                        }
                                    }
                                    _ => {}
                                }
                            }
                            
                            let trimmed = current_content.trim();
                            if trimmed.len() > 30 {
                                chunks.push(DocBlock {
                                    title: breadcrumbs.join(" > "),
                                    url: file_url.clone(),
                                    content: trimmed.to_string(),
                                    links: current_links,
                                });
                            }
                        }
                    }
                }
            }
        }
    }
    
    walk_dir(Path::new(folder_path), &mut chunks);
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
    
    let tags = ["script", "style", "nav", "footer", "header", "head", "aside"];
    let mut clean_html = html.clone();
    for tag in tags {
        let pattern = format!(r"(?is)<{}\b[^>]*>.*?</{}>", tag, tag);
        let re = Regex::new(&pattern).unwrap();
        clean_html = re.replace_all(&clean_html, "").to_string();
    }
    
    let document = Html::parse_document(&clean_html);
    let mut chunks = Vec::new();
    
    let mut current_title = "Documentation Page".to_string();
    let mut current_content = String::new();
    let mut current_links = Vec::new();
    
    let selector = Selector::parse("h1, h2, h3, h4, h5, h6, p, li, pre").unwrap();
    let a_selector = Selector::parse("a").unwrap();
    
    for element in document.select(&selector) {
        let tag = element.value().name();
        let text = element.text().collect::<Vec<_>>().join(" ").trim().to_string();
        
        if tag.starts_with('h') && tag.len() == 2 {
            if current_content.trim().len() > 30 {
                chunks.push(DocBlock {
                    title: current_title.clone(),
                    url: url.to_string(),
                    content: current_content.trim().to_string(),
                    links: std::mem::take(&mut current_links),
                });
            }
            current_content.clear();
            current_title = text;
        } else {
            if !text.is_empty() {
                current_content.push_str(&text);
                current_content.push_str("\n\n");
            }
            for a in element.select(&a_selector) {
                if let Some(href) = a.value().attr("href") {
                    let a_text = a.text().collect::<Vec<_>>().join(" ").trim().to_string();
                    current_links.push((a_text, href.to_string()));
                }
            }
        }
    }
    
    if current_content.trim().len() > 30 {
        chunks.push(DocBlock {
            title: current_title,
            url: url.to_string(),
            content: current_content.trim().to_string(),
            links: current_links,
        });
    }
    
    chunks
}
