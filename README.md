# Minio Uploader for Windows

This is a simple Rust application that integrates with the Windows context menu (right-click menu) to allow users to quickly upload files to a Minio (S3-compatible) server.

## Features

* **Context Menu Integration**: Right-click any file in Windows Explorer to see an "Upload to Minio" option.
* **Configurable**: Minio server details (endpoint, access key, secret key, bucket) are configured via a `Settings.toml` file.
* **Error Handling**: Displays a native Windows dialog for any upload errors or missing configuration.

## Prerequisites

* **Rust Toolchain**: You need to have Rust and Cargo installed. Follow the instructions on [rustup.rs](https://rustup.rs/).
* **Minio Server**: A running Minio server or any S3-compatible object storage service.

## Setup and Installation

Follow these steps to set up and install the Minio Uploader:

### 1. Clone the Repository (or create the project)

If you haven't already, create the project structure:

```bash
car go new minio_uploader
cd minio_uploader
```

### 2. Configure `Cargo.toml`

Ensure your `Cargo.toml` file has the necessary dependencies:

```toml
[package]
name = "minio_uploader"
version = "0.1.0"
edition = "2021"
description = "A simple Windows context menu tool to upload files to a Minio server."
license = "MIT"
repository = "https://github.com/your-username/your-repo" # Update with your GitHub repo URL

[dependencies]
minio = "0.3.0"
tokio = { version = "1", features = ["full"] }
config = "0.13"
serde = { version = "1.0", features = ["derive"] }
anyhow = "1.0"
native-dialog = "0.6"
url = "2.5"
```

### 3. Create `Settings.toml`

Create a file named `Settings.toml` in the `minio_uploader` directory (the same directory as `Cargo.toml`) with your Minio server details. **Replace the placeholder values with your actual Minio configuration.**

```toml
# Minio/S3 Server Endpoint URL.
# For example: "http://127.0.0.1:9000" or "https://s3.amazonaws.com"
endpoint = "http://your-minio-server:9000"

# Your S3 Access Key
access_key = "YOUR_ACCESS_KEY"

# Your S3 Secret Key
secret_key = "YOUR_SECRET_KEY"

# The name of the bucket to upload files to.
bucket = "your-bucket-name"
```

### 4. Build the Application

Navigate to the `minio_uploader` directory in your terminal and build the Rust application in release mode:

```bash
car go build --release
```

This will generate an executable file at `target/release/minio_uploader.exe`.

### 5. Deploy Configuration File

**Crucially**, copy the `Settings.toml` file you created in step 3 into the same directory as the executable:

`C:\Users\c3qu\Desktop\upload\minio_uploader\target\release\Settings.toml`

### 6. Add to Windows Context Menu

To add the "Upload to Minio" option to your right-click context menu, you need to import a `.reg` file into your Windows Registry.

Create a file named `add_context_menu.reg` in your project root (`C:\Users\c3qu\Desktop\upload\`) with the following content:

```reg
Windows Registry Editor Version 5.00

[HKEY_CLASSES_ROOT\*\shell\UploadToMinio]
@="Upload to Minio"
"Icon"="imageres.dll,7"

[HKEY_CLASSES_ROOT\*\shell\UploadToMinio\command]
@="\"C:\\Users\\c3qu\\Desktop\\upload\\minio_uploader\\target\\release\\minio_uploader.exe\" \"1\""
```

**Important**: Double-check the path to `minio_uploader.exe` in the `.reg` file. It must be the absolute path to your compiled executable.

Double-click the `add_context_menu.reg` file and confirm the prompts to add the entry to your registry.

### 7. Usage

Right-click any file in Windows Explorer. You should now see an "Upload to Minio" option. Click it to upload the file to your configured Minio bucket.

## Uninstallation

To remove the context menu entry, create a file named `remove_context_menu.reg` in your project root (`C:\Users\c3qu\Desktop\upload\`) with the following content:

```reg
Windows Registry Editor Version 5.00

[-HKEY_CLASSES_ROOT\*\shell\UploadToMinio]
```

Double-click this file and confirm the prompts to remove the entry from your registry.

## License

This project is licensed under the MIT License. See the `LICENSE` file for details (if you create one).
