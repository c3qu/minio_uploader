# MinIO Uploader

A simple Windows context menu tool to upload files to a MinIO (S3 compatible) server.

## Features
- Drag-and-drop or context menu to upload a single file
- Auto-creates a right-click context menu on first run (Current User)
- Hides console window on Windows (GUI subsystem)
- Copies the uploaded object URL to clipboard on success
- Friendly error dialogs for common issues

## Requirements
- Windows 10+
- MinIO/S3 endpoint accessible from your machine

## Configuration
Create a `Settings.toml` next to the executable with:

```toml
endpoint = "http://127.0.0.1:9000"  # or https://play.min.io
access_key = "minioadmin"
secret_key = "minioadmin"
bucket = "my-bucket"
```

## Usage
- After first run, a context menu item "Upload to MinIO" is added for files.
- Right-click any file → Upload to MinIO.
- On success, the object URL is copied to clipboard.

## Build
```bash
cargo build --release
```
The binary will be at `target/release/minio_uploader.exe`.

## Notes
- Context menu registry path: `HKCU\Software\Classes\*\shell\MinIO Uploader\command`
- To support folders, also add a similar key under `HKCU\Software\Classes\Directory\shell\...`.

## License
MIT
