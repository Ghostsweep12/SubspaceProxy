use std::process::{Command, Stdio};
use std::env;
use std::path::PathBuf;
use tauri::AppHandle;
use tauri::path::BaseDirectory;
use tauri::Manager;
use serde::{Deserialize, Serialize};

pub fn call_bash_function(function_name: &str,args: &[&str],) -> Result<(i32, String, String), String> {
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

#[derive(Serialize, Deserialize, Debug)]
pub struct Environment {
    REAL_USER: String,
    REAL_UID: u32,
    REAL_HOME: String,
    REAL_XDG_RUNTIME: String,
    PULSE_SOCK: String,
    DBUS_SOCK: String,
    TARGET_DISPLAY: String,
    TARGET_XAUTH: String,
    TARGET_WAYLAND: String,
}

#[tauri::command]
pub fn configure_environment(app: AppHandle) -> Result<String, String> {
   // Write the necessary environment configuration to a file 
    let (_, stdout, _) = call_bash_function("reconstruct_user_environment", &[]).map_err(|e| format!("Failed to reconstruct user environment: {}", e))?;

    let env_data: Environment = serde_json::from_str(&stdout).map_err(|e| format!("Failed to parse bash output: {}. Raw: {}", e, stdout))?;

    let environment_path = app.path().resolve("environment.json", BaseDirectory::AppData).map_err(|e| format!("Failed to resolve environment path: {}", e))?;

    let pretty_json = serde_json::to_string_pretty(&env_data).map_err(|e| format!("Failed to format JSON: {}", e))?;

    std::fs::write(&environment_path, pretty_json).map_err(|e| format!("Failed to write environment.json: {}", e))?;

    Ok(format!("Wrote validated environment to {:?}", environment_path))
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Profile {
    ip: String,
    port: String,
    protocol: String,
    cmd: String,
    namespace: String,
    tun_interface: String,
    tun_ip: String,
    veth_host: String,
    veth_ns: String,
    veth_host_ip: String,
    veth_ns_ip: String,
    dns: String,
}

#[tauri::command]
pub fn save_profile(app: AppHandle, name: String, ip: String, port: String, protocol: String, cmd: String, dns: String) -> Result<String, String> {
    let profile = Profile {
        ip,
        port,
        protocol,
        cmd,
        dns,
        // Defaults
        namespace: "proxied".to_string(),
        tun_interface: "tun0".to_string(),
        tun_ip: "10.0.0.2".to_string(),
        veth_host: "veth_host".to_string(),
        veth_ns: "veth_ns".to_string(),
        veth_host_ip: "10.200.1.1".to_string(),
        veth_ns_ip: "10.200.1.2".to_string(),
    };

    let app_data_dir = app.path().resolve("profiles", BaseDirectory::AppData).map_err(|e| format!("Failed to resolve app data path: {}", e))?;

    if !app_data_dir.exists() {
        std::fs::create_dir_all(&app_data_dir).map_err(|e| format!("Failed to create profiles directory: {}", e))?;
    }

    let filename = format!("{}.json", name.replace(" ", "_"));
    let file_path = app_data_dir.join(filename);
    let json_content = serde_json::to_string_pretty(&profile).map_err(|e| format!("Failed to serialize profile: {}", e))?;

    std::fs::write(&file_path, json_content).map_err(|e| format!("Failed to write file: {}", e))?;

    Ok(format!("Profile saved to {:?}", file_path))
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
