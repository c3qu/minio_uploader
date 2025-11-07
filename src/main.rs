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

#[derive(Debug, Deserialize)]
struct Settings {
    endpoint: String,
    access_key: String,
    secret_key: String,
    bucket: String,
}

impl Settings {
    pub fn new() -> Result<Self> {
        let mut config_path = env::current_exe()?;
        config_path.pop();
        config_path.push("Settings.toml");

        if !config_path.exists() {
            show_error_dialog(&format!(
                "Configuration file not found. Please create 'Settings.toml' in the same directory as the executable.\n\nExpected path: {}",
                config_path.display()
            ));
            return Err(anyhow::anyhow!("Config file not found"));
        }

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

async fn run() -> Result<()> {
    let settings = Settings::new()?;

    let args: Vec<String> = env::args().collect();
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
    client
        .put_object_content(&settings.bucket, file_name, content)
        .send()
        .await?;

    // Build object URL and copy to clipboard (best-effort)
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
    match Clipboard::new().and_then(|mut c| c.set_text(object_url.clone())) {
        Ok(_) => {}
        Err(e) => {
            // Don't fail the upload if clipboard fails; show a dialog instead
            show_error_dialog(&format!("Uploaded but failed to copy to clipboard: {}\nURL: {}", e, object_url));
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("Error: {:?}", e);
        std::process::exit(1);
    }
}