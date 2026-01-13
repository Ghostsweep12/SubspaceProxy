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
pub async fn setup_namespace(app: AppHandle, profile_path: String) -> Result<String, String> {
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

    let pid_path = app
        .path()
        .resolve("pid.json", BaseDirectory::AppData)
        .map_err(|e| format!("Failed to resolve PID path: {}", e))?;

    let tun2socks_pid = match profile_data.protocol.as_str() {
        "socks5" => {
            let (_return_code, stdout, _stderr) = call_bash_function("tun2socks_socks5", &[
                &profile_data.ip,
                &profile_data.port,
                &profile_data.namespace,
                &profile_data.tun_interface,
                &profile_data.username,
                &profile_data.password,
            ]).map_err(|e| format!("Failed to setup SOCKS5: {}", e))?;
            stdout.trim().to_string()
        }
        "socks4" => {
            let (_return_code, stdout, _stderr) = call_bash_function("tun2socks_socks4", &[
                &profile_data.ip,
                &profile_data.port,
                &profile_data.namespace,
                &profile_data.tun_interface,
                &profile_data.username,
            ]).map_err(|e| format!("Failed to setup SOCKS4: {}", e))?;
            stdout.trim().to_string()
        }
        "http" => {
            let (_return_code, stdout, _stderr) = call_bash_function("tun2socks_http", &[
                &profile_data.ip,
                &profile_data.port,
                &profile_data.namespace,
                &profile_data.tun_interface,
            ]).map_err(|e| format!("Failed to setup HTTP: {}", e))?;
            stdout.trim().to_string()
        }
        "shadowsocks" => {
            let (_return_code, stdout, _stderr) = call_bash_function("tun2socks_shadowsocks", &[
                &profile_data.ip,
                &profile_data.port,
                &profile_data.namespace,
                &profile_data.tun_interface,
                &profile_data.username,
                &profile_data.password,
            ]).map_err(|e| format!("Failed to setup Shadowsocks: {}", e))?;
            stdout.trim().to_string()
        }
        "relay" => {
            let (_return_code, stdout, _stderr) = call_bash_function("tun2socks_relay", &[
                &profile_data.ip,
                &profile_data.port,
                &profile_data.namespace,
                &profile_data.tun_interface,
                &profile_data.username,
                &profile_data.password,
            ]).map_err(|e| format!("Failed to setup Relay: {}", e))?;
            stdout.trim().to_string()
        }
        "direct" => {
            let (_return_code, stdout, _stderr) = call_bash_function("tun2socks_direct", &[
                &profile_data.namespace,
                &profile_data.tun_interface,
            ]).map_err(|e| format!("Failed to setup Direct: {}", e))?;
            stdout.trim().to_string()
        }
        "reject" => {
            let (_return_code, stdout, _stderr) = call_bash_function("tun2socks_reject", &[
                &profile_data.namespace,
                &profile_data.tun_interface,
            ]).map_err(|e| format!("Failed to setup Reject: {}", e))?;
            stdout.trim().to_string()
        }
        _ => {
            return Err(format!("Unsupported protocol: {}", profile_data.protocol));
        }
    };

    std::fs::write(&pid_path, &tun2socks_pid).map_err(|e| format!("Failed to write pid.json: {}", e))?;

    Ok(format!("Namespace created, and tun2socks PID is {}", &tun2socks_pid))
}

#[tauri::command]
pub async fn run(profile_path: String, cmd: String) -> Result<String, String> {
    let profile_data = fetch_profile(profile_path.clone()).await?;

    call_bash_function("run_command_in_namespace", &[
        &profile_data.namespace,
        &cmd,
    ])
    .map_err(|e| format!("Failed to run profile: {}", e))?;

    Ok(format!("Profile {} is running", profile_path))
}

#[tauri::command]
pub async fn cleanup(profile_path: String, pid_path: String) -> Result<String, String> {
    let profile_data = fetch_profile(profile_path.clone()).await?;
    let tun2socks_pid = std::fs::read_to_string(&pid_path).map_err(|e| format!("Failed to read tun2socks PID file: {}", e))?;

    call_bash_function("cleanup", &[
        &profile_data.namespace,
        &tun2socks_pid,
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
    let profile = Profile {
        ip,
        port,
        protocol,
        dns: if dns.is_empty() { "8.8.8.8".to_string() } else { dns },
        namespace: if namespace.is_empty() { "namespace".to_string() } else { namespace },
        username: if username.is_empty() { "".to_string() } else { username },
        password: if password.is_empty() { "".to_string() } else { password },
        tun_interface: if tun_interface.is_empty() { "tun0".to_string() } else { tun_interface },
        tun_ip: if tun_ip.is_empty() { "10.0.0.2".to_string() } else { tun_ip },
        veth_host: if veth_host.is_empty() { "veth_host".to_string() } else { veth_host },
        veth_ns: if veth_ns.is_empty() { "veth_ns".to_string() } else { veth_ns },
        veth_host_ip: if veth_host_ip.is_empty() { "10.200.1.1".to_string() } else { veth_host_ip },
        veth_ns_ip: if veth_ns_ip.is_empty() { "10.200.1.2".to_string() } else { veth_ns_ip },
    };

    let app_data_dir = app
        .path()
        .resolve("profiles", BaseDirectory::AppData)
        .map_err(|e| format!("Failed to resolve app data path: {}", e))?;

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
                .parse::<f64>()
                .ok()
        })
        .unwrap_or(0.0);

        if avg_ping == 0.0 {
            return Err("Server is unreachable".to_string());
        }

    return Ok(format!("{} ms", avg_ping));
}
