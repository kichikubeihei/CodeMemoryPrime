use std::path::Path;
use regex::Regex;

#[derive(Debug, Clone)]
pub struct Chunk {
    pub chunk_type: String,
    pub name: String,
    pub code_content: String,
    pub summary: String,
    pub parent_context: String,
}

pub fn extract_imports(file_path: &str, content: &str) -> Vec<String> {
    let ext = Path::new(file_path)
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();
        
    let mut imports = Vec::new();
    
    // JS/TS import regexes
    let js_re = Regex::new(r#"^\s*(?:import\s+.*?from\s+['"]([^'"]+)['"]|import\s+['"]([^'"]+)['"]|const\s+.*?=\s*require\s*\(\s*['"]([^'"]+)['"]\s*\))"#).unwrap();
    // Python import regexes
    let py_re = Regex::new(r#"^\s*(?:import\s+([\w\.,\s]+)|from\s+(\S+)\s+import)"#).unwrap();
    // Rust import regexes
    let rs_re = Regex::new(r#"^\s*use\s+([^;]+);"#).unwrap();
    
    for line in content.lines() {
        let line_strip = line.trim();
        if line_strip.is_empty() {
            continue;
        }
        
        if ext == "py" {
            if let Some(caps) = py_re.captures(line) {
                if let Some(m) = caps.get(1) {
                    for name in m.as_str().split(',') {
                        imports.push(name.trim().to_string());
                    }
                } else if let Some(m) = caps.get(2) {
                    imports.push(m.as_str().to_string());
                }
            }
        } else if ["js", "jsx", "ts", "tsx", "svelte"].contains(&ext.as_str()) {
            if let Some(caps) = js_re.captures(line) {
                let val = caps.get(1).or_else(|| caps.get(2)).or_else(|| caps.get(3));
                if let Some(m) = val {
                    imports.push(m.as_str().to_string());
                }
            }
        } else if ext == "rs" {
            if let Some(caps) = rs_re.captures(line) {
                let val = caps.get(1).map(|m| m.as_str()).unwrap_or("").split('{').next().unwrap_or("").trim();
                imports.push(val.to_string());
            }
        }
    }
    
    imports.sort();
    imports.dedup();
    imports
}

fn count_braces(line: &str) -> (i32, i32) {
    let mut open_braces = 0;
    let mut close_braces = 0;
    let mut in_string = false;
    let mut string_char = None;
    let mut chars = line.chars().peekable();
    
    while let Some(c) = chars.next() {
        if in_string {
            if c == '\\' {
                chars.next();
                continue;
            }
            if Some(c) == string_char {
                in_string = false;
            }
            continue;
        }
        
        // JS/TS/Rust line comment
        if c == '/' && chars.peek() == Some(&'/') {
            break;
        }
        // Python line comment
        if c == '#' {
            break;
        }
        
        if c == '"' || c == '\'' || c == '`' {
            in_string = true;
            string_char = Some(c);
            continue;
        }
        
        if c == '{' {
            open_braces += 1;
        } else if c == '}' {
            close_braces += 1;
        }
    }
    
    (open_braces, close_braces)
}

fn extract_logical_braced_chunks(file_name: &str, content: &str, signature_re: &Regex) -> Vec<Chunk> {
    let lines: Vec<&str> = content.lines().collect();
    let mut chunks = Vec::new();
    
    let mut current_chunk = Vec::new();
    let mut chunk_name = String::new();
    let mut chunk_type = String::new();
    let mut brace_depth = 0;
    let mut in_block = false;
    
    for line in lines {
        let (open_b, close_b) = count_braces(line);
        
        if !in_block {
            if let Some(caps) = signature_re.captures(line) {
                let name = caps.iter().skip(1).flatten().next().map(|m| m.as_str().to_string());
                if let Some(n) = name {
                    in_block = true;
                    chunk_name = n;
                    
                    let line_lower = line.to_lowercase();
                    if line_lower.contains("class") {
                        chunk_type = "class".to_string();
                    } else if line_lower.contains("struct") {
                        chunk_type = "struct".to_string();
                    } else if line_lower.contains("impl") {
                        chunk_type = "implementation".to_string();
                    } else if line_lower.contains("trait") {
                        chunk_type = "trait".to_string();
                    } else if line_lower.contains("enum") {
                        chunk_type = "enum".to_string();
                    } else {
                        chunk_type = "function".to_string();
                    }
                    
                    current_chunk = vec![line.to_string()];
                    brace_depth = open_b - close_b;
                    
                    if brace_depth == 0 && line.contains('{') && line.contains('}') {
                        chunks.push(Chunk {
                            chunk_type: chunk_type.clone(),
                            name: chunk_name.clone(),
                            code_content: current_chunk.join("\n"),
                            summary: format!("{} {} defined in {}", chunk_type, chunk_name, file_name),
                            parent_context: format!("File: {}", file_name),
                        });
                        in_block = false;
                    }
                }
            }
        } else {
            current_chunk.push(line.to_string());
            brace_depth += open_b - close_b;
            
            if brace_depth <= 0 {
                chunks.push(Chunk {
                    chunk_type: chunk_type.clone(),
                    name: chunk_name.clone(),
                    code_content: current_chunk.join("\n"),
                    summary: format!("{} {} defined in {}", chunk_type, chunk_name, file_name),
                    parent_context: format!("File: {}", file_name),
                });
                in_block = false;
                current_chunk.clear();
                brace_depth = 0;
            }
        }
    }
    
    chunks
}

fn extract_svelte_html_chunks(file_name: &str, content: &str) -> Vec<Chunk> {
    let mut chunks = Vec::new();
    
    let script_re = Regex::new(r"(?s)<script.*?>([\s\S]*?)<\/script>").unwrap();
    let mut i = 1;
    for cap in script_re.captures_iter(content) {
        chunks.push(Chunk {
            chunk_type: "script".to_string(),
            name: format!("script_block_{}", i),
            code_content: cap.get(0).unwrap().as_str().to_string(),
            summary: format!("Script block {} in Svelte component {}", i, file_name),
            parent_context: format!("File: {}", file_name),
        });
        i += 1;
    }
    
    let style_re = Regex::new(r"(?s)<style.*?>([\s\S]*?)<\/style>").unwrap();
    let mut j = 1;
    for cap in style_re.captures_iter(content) {
        chunks.push(Chunk {
            chunk_type: "style".to_string(),
            name: format!("style_block_{}", j),
            code_content: cap.get(0).unwrap().as_str().to_string(),
            summary: format!("Style block {} in Svelte component {}", j, file_name),
            parent_context: format!("File: {}", file_name),
        });
        j += 1;
    }
    
    let mut markup = script_re.replace_all(content, "").to_string();
    markup = style_re.replace_all(&markup, "").to_string();
    let markup_trim = markup.trim().to_string();
    if !markup_trim.is_empty() {
        chunks.push(Chunk {
            chunk_type: "markup".to_string(),
            name: "template_markup".to_string(),
            code_content: markup_trim,
            summary: format!("HTML template markup in Svelte component {}", file_name),
            parent_context: format!("File: {}", file_name),
        });
    }
    
    chunks
}

fn extract_python_chunks_light(file_name: &str, content: &str) -> Vec<Chunk> {
    let py_sig_re = Regex::new(r"^\s*(?:def|class)\s+(\w+)").unwrap();
    let lines: Vec<&str> = content.lines().collect();
    let mut chunks = Vec::new();
    
    let mut current_chunk = Vec::new();
    let mut chunk_name = String::new();
    let mut chunk_type = String::new();
    let mut base_indent = 0;
    let mut in_block = false;
    
    for line in lines {
        let line_strip = line.trim();
        if line_strip.is_empty() {
            if in_block {
                current_chunk.push(line.to_string());
            }
            continue;
        }
        
        let indent = line.len() - line.trim_start().len();
        
        if !in_block {
            if let Some(caps) = py_sig_re.captures(line) {
                if let Some(m) = caps.get(1) {
                    in_block = true;
                    chunk_name = m.as_str().to_string();
                    chunk_type = if line.trim_start().starts_with("class") { "class".to_string() } else { "function".to_string() };
                    base_indent = indent;
                    current_chunk = vec![line.to_string()];
                }
            }
        } else {
            if indent <= base_indent && !line_strip.is_empty() {
                chunks.push(Chunk {
                    chunk_type: chunk_type.clone(),
                    name: chunk_name.clone(),
                    code_content: current_chunk.join("\n"),
                    summary: format!("{} {} defined in {}", chunk_type, chunk_name, file_name),
                    parent_context: format!("File: {}", file_name),
                });
                
                in_block = false;
                
                if let Some(caps) = py_sig_re.captures(line) {
                    if let Some(m) = caps.get(1) {
                        in_block = true;
                        chunk_name = m.as_str().to_string();
                        chunk_type = if line.trim_start().starts_with("class") { "class".to_string() } else { "function".to_string() };
                        base_indent = indent;
                        current_chunk = vec![line.to_string()];
                    }
                }
            } else {
                current_chunk.push(line.to_string());
            }
        }
    }
    
    if in_block && !current_chunk.is_empty() {
        chunks.push(Chunk {
            chunk_type: chunk_type.clone(),
            name: chunk_name.clone(),
            code_content: current_chunk.join("\n"),
            summary: format!("Type: {} Name: {} defined in {}", chunk_type, chunk_name, file_name),
            parent_context: format!("File: {}", file_name),
        });
    }
    
    chunks
}

fn extract_text_fallback_chunks(file_name: &str, content: &str, size_lines: usize) -> Vec<Chunk> {
    let lines: Vec<&str> = content.lines().collect();
    let total = lines.len();
    if total == 0 {
        return Vec::new();
    }
    
    let mut chunks = Vec::new();
    for i in (0..total).step_by(size_lines) {
        let end = std::cmp::min(i + size_lines, total);
        let segment = lines[i..end].join("\n");
        chunks.push(Chunk {
            chunk_type: "text_block".to_string(),
            name: format!("{}_lines_{}_{}", file_name, i + 1, end),
            code_content: segment,
            summary: format!("Lines {} to {} of {}", i + 1, end, file_name),
            parent_context: format!("File: {}", file_name),
        });
    }
    
    chunks
}

pub fn detect_llm_chunk_type(name: &str, content: &str, file_path: &str) -> Option<String> {
    let content_lower = content.to_lowercase();
    let name_lower = name.to_lowercase();
    let path_lower = file_path.to_lowercase();

    // Check for LLM API calls
    if content_lower.contains("chat.completions.create")
        || content_lower.contains("messages.create")
        || content_lower.contains("generativeai")
        || content_lower.contains("query_llm")
        || content_lower.contains("query_ollama")
        || content_lower.contains("createchatcompletion")
        || content_lower.contains("generatecontent")
        || content_lower.contains("langchain")
    {
        return Some("llm_call".to_string());
    }

    // Check for LLM Prompts & System Instructions
    if path_lower.ends_with(".prompt")
        || path_lower.contains("prompts/")
        || name_lower.contains("prompt")
        || content_lower.contains("system_prompt")
        || content_lower.contains("system_message")
        || content_lower.contains("you are an ai")
        || content_lower.contains("you are an expert")
        || content_lower.contains("<system_prompt>")
        || (content_lower.contains("role") && content_lower.contains("system") && content_lower.contains("content"))
    {
        return Some("llm_prompt".to_string());
    }

    None
}

pub fn parse_file_chunks(file_path: &str, content: &str) -> Vec<Chunk> {
    let file_name = Path::new(file_path)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("file");
        
    let ext = Path::new(file_path)
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();
        
    let imports = extract_imports(file_path, content);
    let imports_str = if imports.is_empty() {
        format!("File: {}", file_name)
    } else {
        format!("File: {} | Imports: {}", file_name, imports.join(", "))
    };

    let js_sig = Regex::new(r#"(?:export\s+)?(?:async\s+)?(?:function\s+(\w+)|class\s+(\w+)|const\s+(\w+)\s*=\s*(?:\([^)]*\)|[^=]+?)\s*=>|let\s+(\w+)\s*=\s*(?:\([^)]*\)|[^=]+?)\s*=>)"#).unwrap();
    let rs_sig = Regex::new(r#"^\s*(?:pub\s+)?(?:async\s+)?(?:fn\s+(\w+)|struct\s+(\w+)|enum\s+(\w+)|impl\s*(?:<\w+>)?\s+(\w+)|trait\s+(\w+))"#).unwrap();
    let css_sig = Regex::new(r#"^([.#\w\s,:>-]+)\s*\{"#).unwrap();
    
    let mut chunks = Vec::new();
    
    if ext == "py" {
        chunks = extract_python_chunks_light(file_name, content);
    } else if ["js", "jsx", "ts", "tsx"].contains(&ext.as_str()) {
        chunks = extract_logical_braced_chunks(file_name, content, &js_sig);
    } else if ext == "rs" {
        chunks = extract_logical_braced_chunks(file_name, content, &rs_sig);
    } else if ext == "css" {
        chunks = extract_logical_braced_chunks(file_name, content, &css_sig);
    } else if ["html", "svelte"].contains(&ext.as_str()) {
        chunks = extract_svelte_html_chunks(file_name, content);
    }
    
    if chunks.is_empty() {
        chunks = extract_text_fallback_chunks(file_name, content, 40);
    }
    
    for chunk in &mut chunks {
        chunk.parent_context = imports_str.clone();
        if let Some(llm_type) = detect_llm_chunk_type(&chunk.name, &chunk.code_content, file_path) {
            chunk.chunk_type = llm_type;
        }
    }
    
    chunks
}
