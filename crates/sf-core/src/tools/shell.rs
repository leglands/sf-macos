use serde_json::Value;
use tokio::process::Command;
use crate::SfResult;

pub async fn run(args: &Value) -> SfResult<String> {
    let cmd = args["command"].as_str().ok_or_else(|| crate::SfError::Tool { name: "shell_run".into(), msg: "missing command".into() })?;
    let cwd = args["cwd"].as_str().unwrap_or(".");
    let timeout_secs = args["timeout"].as_u64().unwrap_or(30);

    let output = tokio::time::timeout(
        std::time::Duration::from_secs(timeout_secs),
        Command::new("sh").arg("-c").arg(cmd).current_dir(cwd).output()
    ).await
        .map_err(|_| crate::SfError::Tool { name: "shell_run".into(), msg: format!("Timeout after {timeout_secs}s") })?
        .map_err(|e| crate::SfError::Tool { name: "shell_run".into(), msg: e.to_string() })?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let status = output.status.code().unwrap_or(-1);

    if stderr.is_empty() {
        Ok(format!("[exit {status}]\n{stdout}"))
    } else {
        Ok(format!("[exit {status}]\nSTDOUT: {stdout}\nSTDERR: {stderr}"))
    }
}
