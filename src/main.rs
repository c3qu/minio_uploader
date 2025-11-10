#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]
use anyhow::Result;
use config::{Config, File};
use native_dialog::{MessageDialog, MessageType};
use serde::Deserialize;
use std::env;
use std::path::Path;
use tokio::fs::File as TokioFile;
use tokio::io::AsyncReadExt;

use minio::s3::builders::ObjectContent;
use minio::s3::client::ClientBuilder;
use minio::s3::creds::StaticProvider;
use minio::s3::http::BaseUrl;
use arboard::Clipboard;
use urlencoding::encode;
#[cfg(windows)]
use winreg::{enums::HKEY_CURRENT_USER, RegKey};

#[derive(Debug, Deserialize)]
struct Settings {
    endpoint: String,
    access_key: String,
    secret_key: String,
    bucket: String,
}

impl Settings {
    pub fn new() -> Result<Self> {
        // Priority 1: %APPDATA%/MinioUploader/Settings.toml
        let appdata_config = dirs::data_dir()
            .map(|mut path| {
                path.push("MinioUploader");
                path.push("Settings.toml");
                path
            })
            .filter(|p| p.exists());

        // Priority 2: Executable directory/Settings.toml
        let exe_dir_config = env::current_exe()
            .ok()
            .map(|mut path| {
                path.pop();
                path.push("Settings.toml");
                path
            })
            .filter(|p| p.exists());

        // Try appdata first, then exe directory
        let config_path = appdata_config
            .or(exe_dir_config)
            .ok_or_else(|| {
                let appdata_path = dirs::data_dir()
                    .map(|mut p| {
                        p.push("MinioUploader");
                        p.push("Settings.toml");
                        p.display().to_string()
                    })
                    .unwrap_or_else(|| "%APPDATA%\\MinioUploader\\Settings.toml".to_string());
                
                let exe_path = env::current_exe()
                    .ok()
                    .map(|mut p| {
                        p.pop();
                        p.push("Settings.toml");
                        p.display().to_string()
                    })
                    .unwrap_or_else(|| "<executable_dir>\\Settings.toml".to_string());

                let error_msg = format!(
                    "Configuration file not found. Please create 'Settings.toml' in one of the following locations:\n\n1. {} (recommended)\n2. {}",
                    appdata_path, exe_path
                );
                show_error_dialog(&error_msg);
                anyhow::anyhow!("Config file not found")
            })?;

        let builder = Config::builder().add_source(File::from(config_path.as_path()));
        let settings = builder.build()?.try_deserialize()?;
        Ok(settings)
    }
}

fn show_error_dialog(message: &str) {
    MessageDialog::new()
        .set_title("Minio Uploader Error")
        .set_text(message)
        .set_type(MessageType::Error)
        .show_alert()
        .unwrap();
}

fn show_info_dialog(message: &str) {
    MessageDialog::new()
        .set_title("Minio Uploader")
        .set_text(message)
        .set_type(MessageType::Info)
        .show_alert()
        .unwrap();
}

async fn run() -> Result<()> {
    // Parse args first, in case we need to uninstall without requiring Settings.toml
    let args: Vec<String> = env::args().collect();

    #[cfg(windows)]
    if args.iter().any(|a| a.eq_ignore_ascii_case("--uninstall") || a.eq_ignore_ascii_case("/uninstall")) {
        match remove_context_menu_registration() {
            Ok(_) => {
                show_info_dialog("已移除右键菜单 (Current User)。");
                return Ok(());
            }
            Err(e) => {
                show_error_dialog(&format!("移除右键菜单失败: {:?}", e));
                return Err(e);
            }
        }
    }

    #[cfg(windows)]
    {
        if let Err(e) = ensure_context_menu_registered() {
            // Non-fatal; show dialog to inform the user
            show_error_dialog(&format!("Failed to register context menu: {:?}", e));
        }
    }

    let settings = Settings::new()?;

    if args.len() < 2 {
        let exe_name = args.get(0).map_or("minio_uploader.exe", |s| s.as_str());
        let msg = format!("No file path provided.\n\nUsage: Drag a file onto {} or use the context menu.", exe_name);
        show_error_dialog(&msg);
        return Err(anyhow::anyhow!("No file path provided"));
    }

    let file_path_str = &args[1];
    let file_path = Path::new(file_path_str);

    if !file_path.exists() {
        show_error_dialog(&format!("File does not exist: {}", file_path_str));
        return Err(anyhow::anyhow!("File not found"));
    }

    let file_name = file_path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown_file");

    let base_url: BaseUrl = settings.endpoint.parse()?;
    let client = ClientBuilder::new(base_url)
        .provider(Some(Box::new(StaticProvider::new(
            &settings.access_key,
            &settings.secret_key,
            None,
        ))))
        .build()?;

    let mut file = TokioFile::open(&file_path).await?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).await?;

    let content = ObjectContent::from(buffer);
    let result = client
        .put_object_content(&settings.bucket, file_name, content)
        .send()
        .await;

    // Build object URL
    let mut endpoint = settings.endpoint.trim().to_string();
    if endpoint.ends_with('/') {
        endpoint.pop();
    }
    let object_url = format!(
        "{}/{}/{}",
        endpoint,
        &settings.bucket,
        encode(file_name)
    );
    match result {
        Ok(_) => {
            let mut copied = true;
            if let Err(e) = Clipboard::new().and_then(|mut c| c.set_text(object_url.clone())) {
                copied = false;
                show_error_dialog(&format!("上传成功，但复制到剪切板失败: {}\nURL: {}", e, object_url));
            }
            if copied {
                show_info_dialog(&format!("上传成功，链接已复制到剪切板:\n{}", object_url));
            }
            Ok(())
        }
        Err(e) => {
            show_error_dialog(&format!("上传失败: {}", e));
            Err(anyhow::anyhow!(e))
        }
    }
}

#[cfg(windows)]
fn ensure_context_menu_registered() -> Result<()> {
    // Create HKCU\Software\Classes\*\shell\MinIO Uploader\command
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let base_path = "Software\\Classes\\*\\shell\\MinIO Uploader";
    let command_path = format!("{}\\command", base_path);

    // If command key exists, assume already registered
    if hkcu.open_subkey(&command_path).is_ok() {
        return Ok(());
    }

    let exe = env::current_exe()?;
    let exe_str = exe.display().to_string();

    // Create main key
    let (key, _) = hkcu.create_subkey(base_path)?;
    key.set_value("", &"Upload to MinIO")?;
    key.set_value("Icon", &exe_str)?;

    // Create command key with quoted path and %1
    let (cmd_key, _) = hkcu.create_subkey(command_path)?;
    let command = format!("\"{}\" \"%1\"", exe_str);
    cmd_key.set_value("", &command)?;

    Ok(())
}

#[cfg(windows)]
fn remove_context_menu_registration() -> Result<()> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let base_path = "Software\\Classes\\*\\shell\\MinIO Uploader";
    if hkcu.open_subkey(base_path).is_err() {
        return Ok(());
    }
    hkcu.delete_subkey_all(base_path)?;
    Ok(())
}

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("Error: {:?}", e);
        std::process::exit(1);
    }
}