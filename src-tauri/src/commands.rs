use serde::{Deserialize, Serialize};
use std::env;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use tauri::path::BaseDirectory;
use tauri::AppHandle;
use tauri::Manager;

#[derive(Serialize, Deserialize, Debug)]
pub struct Profile {
    ip: String,
    port: String,
    protocol: String,
    namespace: String,
    username: String,
    password: String,
    tun_interface: String,
    tun_ip: String,
    veth_host: String,
    veth_ns: String,
    veth_host_ip: String,
    veth_ns_ip: String,
    dns: String,
}

#[derive(Serialize, Debug)]
pub struct ProfileEntry {
    filename: String,
    path: String,
    profile: Profile,
}

#[derive(Serialize, Debug)]
pub struct NamespaceInfo {
    name: String,
    processes: String,
}

pub fn call_bash_function(function_name: &str, args: &[&str]) -> Result<(i32, String, String), String> {
    // Call a bash function from functions.sh and supply it arguements, then capture its output
    let exe_path =
        env::current_exe().map_err(|e| format!("Failed to get current exe path: {e}"))?;
    let exe_dir = exe_path
        .parent()
        .ok_or("Failed to get exe parent directory")?;
    let functions_path: PathBuf = exe_dir.join("functions.sh");

    let bash_command = format!(
        "set -euo pipefail; source '{}' && {} \"$@\"",
        functions_path.display(),
        function_name
    );

    let mut command = Command::new("bash");
    command.arg("-c").arg(bash_command).arg("--");

    for arg in args {
        command.arg(arg);
    }

    let output = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| format!("Failed to run bash: {e}"))?;

    let return_code = output.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    println!("code={return_code}, stdout={stdout}, stderr={stderr}");

    Ok((return_code, stdout, stderr))
}

#[tauri::command]
pub async fn list_profiles(app: AppHandle) -> Result<Vec<ProfileEntry>, String> {
    // List all the profiles
    let app_data_dir = app.path().resolve("profiles", BaseDirectory::AppData).map_err(|e| e.to_string())?;
    
    if !app_data_dir.exists() {
        std::fs::create_dir_all(&app_data_dir).map_err(|e| format!("Failed to create profiles directory: {}", e))?;
    }

    let mut profiles = Vec::new();
    let entries = std::fs::read_dir(app_data_dir).map_err(|e| e.to_string())?;

    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
            match serde_json::from_str::<Profile>(&content) {
                Ok(profile) => {
                    profiles.push(ProfileEntry {
                        filename: path.file_stem().unwrap().to_string_lossy().to_string(),
                        path: path.to_string_lossy().to_string(),
                        profile,
                    });
                }
                Err(_) => continue,
            }
        }
    }
    Ok(profiles)
}

#[tauri::command]
pub async fn get_active_namespaces() -> Result<Vec<NamespaceInfo>, String> {
    // Read the currently active namespaces and their associated processes
    let (_return_code, stdout, _stderr) = call_bash_function("get_active_namespaces", &[]).map_err(|e| e.to_string())?;
    let ns_list: Vec<&str> = stdout.trim().split('\n').filter(|s| !s.is_empty()).collect();
    
    let mut infos = Vec::new();
    for ns in ns_list {
        let (_return_code, pstdout, _stderr) = call_bash_function("get_ns_pids", &[ns]).unwrap_or((0, "".to_string(), "".to_string()));
        infos.push(NamespaceInfo {
            name: ns.to_string(),
            processes: pstdout.trim().to_string(),
        });
    }
    Ok(infos)
}

#[tauri::command]
pub async fn setup_namespace(profile_path: String) -> Result<String, String> {
    // Create the namespace with all interfaces and initalize tun2socks
    let profile_data = fetch_profile(profile_path.clone()).await?;

    call_bash_function(
        "setup_namespace",
        &[
            &profile_data.ip,
            &profile_data.port,
            &profile_data.namespace,
            &profile_data.tun_interface,
            &profile_data.tun_ip,
            &profile_data.veth_host,
            &profile_data.veth_ns,
            &profile_data.veth_host_ip,
            &profile_data.veth_ns_ip,
            &profile_data.dns,
        ],
    )
    .map_err(|e| format!("Failed to setup namespace: {}", e))?;

    match profile_data.protocol.as_str() {
        "socks5" => {
            call_bash_function("tun2socks_socks5", &[
                &profile_data.ip,
                &profile_data.port,
                &profile_data.namespace,
                &profile_data.tun_interface,
                &profile_data.username,
                &profile_data.password,
            ]).map_err(|e| format!("Failed to setup SOCKS5: {}", e))?;
        }
        "socks4" => {
            call_bash_function("tun2socks_socks4", &[
                &profile_data.ip,
                &profile_data.port,
                &profile_data.namespace,
                &profile_data.tun_interface,
                &profile_data.username,
            ]).map_err(|e| format!("Failed to setup SOCKS4: {}", e))?;
        }
        "http" => {
            call_bash_function("tun2socks_http", &[
                &profile_data.ip,
                &profile_data.port,
                &profile_data.namespace,
                &profile_data.tun_interface,
            ]).map_err(|e| format!("Failed to setup HTTP: {}", e))?;
        }
        "shadowsocks" => {
            call_bash_function("tun2socks_shadowsocks", &[
                &profile_data.ip,
                &profile_data.port,
                &profile_data.namespace,
                &profile_data.tun_interface,
                &profile_data.username,
                &profile_data.password,
            ]).map_err(|e| format!("Failed to setup Shadowsocks: {}", e))?;
        }
        "relay" => {
            call_bash_function("tun2socks_relay", &[
                &profile_data.ip,
                &profile_data.port,
                &profile_data.namespace,
                &profile_data.tun_interface,
                &profile_data.username,
                &profile_data.password,
            ]).map_err(|e| format!("Failed to setup Relay: {}", e))?;
        }
        "direct" => {
            call_bash_function("tun2socks_direct", &[
                &profile_data.namespace,
                &profile_data.tun_interface,
            ]).map_err(|e| format!("Failed to setup Direct: {}", e))?;
        }
        "reject" => {
            call_bash_function("tun2socks_reject", &[
                &profile_data.namespace,
                &profile_data.tun_interface,
            ]).map_err(|e| format!("Failed to setup Reject: {}", e))?;
        }
        _ => {
            return Err(format!("Unsupported protocol: {}", profile_data.protocol));
        }
    };

    Ok(format!("Namespace created"))
}

#[tauri::command]
pub async fn run(profile_path: String, cmd: String) -> Result<String, String> {
    // Run command in namespace
    let profile_data = fetch_profile(profile_path.clone()).await?;

    call_bash_function("run_command_in_namespace", &[
        &profile_data.namespace,
        &cmd,
    ])
    .map_err(|e| format!("Failed to run profile: {}", e))?;

    Ok(format!("Profile {} is running", profile_path))
}

#[tauri::command]
pub async fn cleanup(profile_path: String) -> Result<String, String> {
    // Remove the namespace and associated interfaces
    let profile_data = fetch_profile(profile_path.clone()).await?;
    
    call_bash_function("cleanup", &[
        &profile_data.namespace,
        &profile_data.veth_host,
    ])
    .map_err(|e| format!("Failed to cleanup profile: {}", e))?;

    Ok(format!("Profile {} has been cleaned up", profile_path))
}

#[tauri::command]
pub async fn save_profile(
    app: AppHandle,
    name: String,
    ip: String,
    port: String,
    protocol: String,
    dns: String,
    namespace: String,
    username: String,
    password: String,
    tun_interface: String,
    tun_ip: String,
    veth_host: String,
    veth_ns: String,
    veth_host_ip: String,
    veth_ns_ip: String,
) -> Result<String, String> {
    // Save formatted JSON to appdata
    let profile = Profile {
        ip,
        port,
        protocol,
        dns,
        namespace,
        username,
        password,
        tun_interface,
        tun_ip,
        veth_host,
        veth_ns,
        veth_host_ip,
        veth_ns_ip,
    };

    let app_data_dir = app
        .path()
        .resolve("profiles", BaseDirectory::AppData)
        .map_err(|e| format!("Failed to resolve app data path: {}", e))?;

    let filename = format!("{}.json", name.replace(" ", "_"));
    let file_path = app_data_dir.join(filename);
    let json_content = serde_json::to_string_pretty(&profile).map_err(|e| format!("Failed to serialize profile: {}", e))?;

    std::fs::write(&file_path, json_content).map_err(|e| format!("Failed to write file: {}", e))?;

    Ok(format!("Profile saved to {:?}", file_path))
}

#[tauri::command]
pub async fn fetch_profile(profile_path: String) -> Result<Profile, String> {
    let profile = std::fs::read_to_string(profile_path).map_err(|e| format!("Failed to read profile file: {}", e))?;
    let profile_data: Profile = serde_json::from_str(&profile).map_err(|e| format!("Failed to parse profile JSON: {}", e))?;
    Ok(profile_data)
}

#[tauri::command]
pub async fn ping(ip: &str) -> Result<String, String> {
    // Check the latency of the server
    let (_return_code, stdout, _stderr) = call_bash_function("ping_test", &[ip]).map_err(|e| format!("Failed to ping server: {}", e))?;

    let avg_ping = stdout
        .lines()
        .find(|line| line.starts_with("rtt "))
        .and_then(|line| {
            line.split(' ')
                .nth(3)?
                .split('/')
                .nth(1)?
                .parse::<String>()
                .ok()
        })
        .unwrap_or("0".to_string());

        if avg_ping == "0" {
            return Err("Server is unreachable".to_string());
        }

    Ok(avg_ping)
}

#[tauri::command]
pub async fn port(ip: &str, port: &str) -> Result<String, String> {
    // Check if the port is open
    let (_return_code, stdout, _stderr) = call_bash_function("port_test", &[ip, port]).map_err(|e| e.to_string())?;
    
    let words: Vec<&str> = stdout.split_whitespace().collect();

    let result = if words.len() >= 3 {
        words[words.len() - 3..].join(" ")
    } else {
        stdout.trim().to_string()
    };

    Ok(result)
}