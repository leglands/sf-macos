use serde_json::Value;
use tokio::process::Command;
use crate::SfResult;

async fn git_cmd(args: &[&str], cwd: &str) -> SfResult<String> {
    let output = Command::new("git").args(args).current_dir(cwd).output().await
        .map_err(|e| crate::SfError::Tool { name: "git".into(), msg: e.to_string() })?;
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

pub async fn status(args: &Value) -> SfResult<String> {
    let cwd = args["cwd"].as_str().unwrap_or(".");
    git_cmd(&["status", "--porcelain"], cwd).await
}

pub async fn diff(args: &Value) -> SfResult<String> {
    let cwd = args["cwd"].as_str().unwrap_or(".");
    let ref_ = args["ref"].as_str().unwrap_or("HEAD");
    git_cmd(&["diff", "--no-pager", ref_], cwd).await
}

pub async fn log(args: &Value) -> SfResult<String> {
    let cwd = args["cwd"].as_str().unwrap_or(".");
    let n = args["n"].as_u64().unwrap_or(10);
    let n_str = n.to_string();
    git_cmd(&["log", "--oneline", &format!("-{n_str}")], cwd).await
}
