use std::process::{Command, Stdio};
use std::env;
use std::path::PathBuf;
use tauri::AppHandle;
use tauri::path::BaseDirectory;
use tauri::Manager;

pub fn call_bash_function(
    function_name: &str,
    args: &[&str],
) -> Result<(i32, String, String), String> {
    // Call a bash function from functions.sh and supply it arguements if any
    let exe_path = env::current_exe().map_err(|e| format!("Failed to get current exe path: {e}"))?;
    let exe_dir = exe_path.parent().ok_or("Failed to get exe parent directory")?;
    let functions_path: PathBuf = exe_dir.join("functions.sh");

    if !functions_path.exists() {
        return Err(format!("functions.sh not found at {:?}", functions_path));
    }

    let bash_command = format!(
        "set -euo pipefail; source '{}' && {} \"$@\"",
        functions_path.display(),
        function_name
    );

    let mut command = Command::new("bash");
    command
        .arg("-c")
        .arg(bash_command)
        .arg("--");

    for arg in args {
        command.arg(arg);
    }

    let output = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| format!("Failed to spawn bash: {e}"))?;

    let return_code = output.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    Ok((return_code, stdout, stderr))
}

#[tauri::command]
pub fn configure_environment(app: AppHandle) -> String {
    // Write the necessary environment configuration to a file
    let (_return_code, stdout, _stderr) = match call_bash_function("reconstruct_user_environment", &[]) {
        Ok(t) => t,
        Err(e) => {
            return format!("Failed to reconstruct user environment: {}", e);
        }
    };

    let environment_path = match app
        .path()
        .resolve("environment.json", BaseDirectory::AppData) {
            Ok(p) => p,
            Err(e) => return format!("Failed to resolve environment path: {}", e),
        };

    if let Err(e) = std::fs::write(&environment_path, stdout) {
        return format!("Failed to write environment.json: {}", e);
    }

    return format!("Wrote environment.json to {:?}", environment_path)
}

#[tauri::command]
pub async fn ping(ip: &str) -> Result<String, String> {
    // Check the latency of the server
    let (_return_code, stdout, _stderr) = match call_bash_function("ping_test", &[ip]) {
        Ok(t) => t,
        Err(e) => {
            return Err(format!("Failed to ping server: {}", e));
        }
    };

    let avg_ping = stdout
        .lines()
        .find(|line| line.starts_with("rtt "))
        .and_then(|line| {
            line.split('=')
                .nth(1)?
                .trim()
                .split('/')
                .nth(1)?
                .parse::<f64>()
                .ok()
        })
        .unwrap_or(0.0);

    return Ok(format!("{} ms", avg_ping))
}
