use serde_json::Value;
use regex::Regex;
use std::fs;
use crate::SfResult;

pub async fn grep(args: &Value) -> SfResult<String> {
    let pattern = args["pattern"].as_str().ok_or_else(|| crate::SfError::Tool { name: "grep".into(), msg: "missing pattern".into() })?;
    let path = args["path"].as_str().unwrap_or(".");
    let max_results = args["max_results"].as_u64().unwrap_or(50) as usize;

    let re = Regex::new(pattern).map_err(|e| crate::SfError::Tool { name: "grep".into(), msg: e.to_string() })?;
    let mut results = Vec::new();

    walk_and_grep(&re, path, &mut results, max_results)?;

    if results.is_empty() {
        Ok("No matches found.".into())
    } else {
        Ok(results.join("\n"))
    }
}

fn walk_and_grep(re: &Regex, path: &str, results: &mut Vec<String>, max: usize) -> SfResult<()> {
    if results.len() >= max { return Ok(()); }
    let meta = fs::metadata(path)?;
    if meta.is_file() {
        if let Ok(content) = fs::read_to_string(path) {
            for (i, line) in content.lines().enumerate() {
                if re.is_match(line) {
                    results.push(format!("{path}:{}: {line}", i + 1));
                    if results.len() >= max { return Ok(()); }
                }
            }
        }
    } else if meta.is_dir() {
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                let child = entry.path();
                let name = child.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
                // Skip hidden dirs and common noise
                if name.starts_with('.') || name == "target" || name == "node_modules" { continue; }
                walk_and_grep(re, &child.to_string_lossy(), results, max)?;
                if results.len() >= max { break; }
            }
        }
    }
    Ok(())
}
