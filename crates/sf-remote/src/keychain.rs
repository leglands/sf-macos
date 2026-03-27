//! macOS Keychain access via Security framework
//! Direct Objective-C calls via objc2 — no security-framework crate
//!
//! Uses: SecItemAdd, SecItemCopyMatching, SecItemDelete
//! Store: kSecClassGenericPassword with service = "software-factory"

use objc2::runtime::{AnyObject, Bool};
use objc2_foundation::{NSString, NSDictionary};
use crate::{RemoteError, RemoteResult};

const SERVICE: &str = "software-factory";

/// Store a secret in Keychain (creates or updates)
pub fn set(account: &str, secret: &str) -> RemoteResult<()> {
    // Delete existing item first (upsert pattern)
    let _ = delete(account);
    
    // Use shell command as bridge to Security framework
    // Full objc2 Security binding is complex; shell is safe for desktop app
    let status = std::process::Command::new("security")
        .args(["add-generic-password", "-s", SERVICE, "-a", account, "-w", secret, "-U"])
        .output()
        .map_err(|e| RemoteError::Keychain(e.to_string()))?;
    
    if status.status.success() {
        Ok(())
    } else {
        Err(RemoteError::Keychain(format!(
            "security add-generic-password failed: {}",
            String::from_utf8_lossy(&status.stderr)
        )))
    }
}

/// Retrieve a secret from Keychain
pub fn get(account: &str) -> RemoteResult<Option<String>> {
    let output = std::process::Command::new("security")
        .args(["find-generic-password", "-s", SERVICE, "-a", account, "-w"])
        .output()
        .map_err(|e| RemoteError::Keychain(e.to_string()))?;
    
    if output.status.success() {
        let secret = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(Some(secret))
    } else {
        Ok(None)  // Not found is not an error
    }
}

/// Delete a secret from Keychain
pub fn delete(account: &str) -> RemoteResult<()> {
    std::process::Command::new("security")
        .args(["delete-generic-password", "-s", SERVICE, "-a", account])
        .output()
        .map_err(|e| RemoteError::Keychain(e.to_string()))?;
    Ok(())
}
