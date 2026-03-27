use serde_json::Value;
use crate::SfResult;
use std::fs;

pub async fn read(args: &Value) -> SfResult<String> {
    let path = args["path"].as_str().ok_or_else(|| crate::SfError::Tool { name: "file_read".into(), msg: "missing path".into() })?;
    Ok(fs::read_to_string(path)?)
}

pub async fn write(args: &Value) -> SfResult<String> {
    let path = args["path"].as_str().ok_or_else(|| crate::SfError::Tool { name: "file_write".into(), msg: "missing path".into() })?;
    let content = args["content"].as_str().unwrap_or("");
    fs::write(path, content)?;
    Ok(format!("Written {} bytes to {path}", content.len()))
}

pub async fn list(args: &Value) -> SfResult<String> {
    let path = args["path"].as_str().unwrap_or(".");
    let entries = fs::read_dir(path)?;
    let mut lines = Vec::new();
    for entry in entries.flatten() {
        let meta = entry.metadata()?;
        let kind = if meta.is_dir() { "d" } else { "f" };
        lines.push(format!("{kind} {}", entry.file_name().to_string_lossy()));
    }
    lines.sort();
    Ok(lines.join("\n"))
}
