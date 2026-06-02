use tauri_plugin_shell::ShellExt;
use walkdir::WalkDir;
use std::fs;
use std::path::Path;
use tempfile::tempdir;
use serde::Serialize;

#[derive(Serialize)]
struct Device {
    serial: String,
    status: String,
}

#[tauri::command]
async fn list_devices(app: tauri::AppHandle) -> Result<Vec<Device>, String> {
    let shell = app.shell();
    let output = shell
        .sidecar("adb")
        .map_err(|e| e.to_string())?
        .args(["devices"])
        .output()
        .await
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut devices = Vec::new();
    
    // Skip the first line "List of devices attached"
    for line in stdout.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() == 2 {
            devices.push(Device {
                serial: parts[0].to_string(),
                status: parts[1].to_string(),
            });
        }
    }
    
    Ok(devices)
}

#[tauri::command]
async fn install_apk(app: tauri::AppHandle, serial: String, path: String) -> Result<String, String> {
    let shell = app.shell();
    let output = shell
        .sidecar("adb")
        .map_err(|e| e.to_string())?
        .args(["-s", &serial, "install", "-r", &path])
        .output()
        .await
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

fn is_junk(path: &Path) -> bool {
    let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    file_name == ".DS_Store" || file_name.starts_with("._") || file_name == ".Trashes"
}

#[tauri::command]
async fn push_files(app: tauri::AppHandle, serial: String, source_paths: Vec<String>, dest_path: String) -> Result<String, String> {
    let shell = app.shell();
    let temp_dir = tempdir().map_err(|e| e.to_string())?;
    let staging_path = temp_dir.path();

    for source in source_paths {
        let source_path = Path::new(&source);
        if !source_path.exists() {
            continue;
        }

        let target_name = source_path.file_name().ok_or("Invalid filename")?;
        let target_path = staging_path.join(target_name);

        if source_path.is_dir() {
            for entry in WalkDir::new(source_path) {
                let entry = entry.map_err(|e| e.to_string())?;
                let rel_path = entry.path().strip_prefix(source_path).map_err(|e| e.to_string())?;
                let dest_file_path = target_path.join(rel_path);

                if is_junk(entry.path()) {
                    continue;
                }

                if entry.path().is_dir() {
                    fs::create_dir_all(&dest_file_path).map_err(|e| e.to_string())?;
                } else {
                    if let Some(parent) = dest_file_path.parent() {
                        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
                    }
                    fs::copy(entry.path(), &dest_file_path).map_err(|e| e.to_string())?;
                }
            }
        } else {
            if !is_junk(source_path) {
                fs::copy(source_path, &target_path).map_err(|e| e.to_string())?;
            }
        }
    }

    let push_source = format!("{}/.", staging_path.to_string_lossy());
    let output = shell
        .sidecar("adb")
        .map_err(|e| e.to_string())?
        .args(["-s", &serial, "push", &push_source, &dest_path])
        .output()
        .await
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

#[tauri::command]
async fn list_remote_dirs(app: tauri::AppHandle, serial: String, path: String) -> Result<Vec<String>, String> {
    let shell = app.shell();
    let output = shell
        .sidecar("adb")
        .map_err(|e| e.to_string())?
        .args(["-s", &serial, "shell", "ls", "-F", &path])
        .output()
        .await
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut dirs = Vec::new();
        // Add ".." for navigation if not at root
        if path != "/" && !path.is_empty() {
            dirs.push("..".to_string());
        }
        for line in stdout.lines() {
            if line.ends_with('/') {
                dirs.push(line.to_string());
            }
        }
        Ok(dirs)
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![list_devices, install_apk, push_files, list_remote_dirs])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
