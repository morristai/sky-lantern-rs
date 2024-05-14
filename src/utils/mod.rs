use std::fs::File;
use std::path::Path;
use std::time::Duration;

use anyhow::{Context, Result};
use md5::{Digest as Md5Digest, Md5};
use reqwest::header;
use reqwest::Client;
use sha1::{Digest, Sha1};
use sha2::Sha256;
use tokio::fs;

pub async fn make_http_request(url: &str) -> Result<Vec<u8>> {
    let timeout = Duration::from_secs(10);
    let client = Client::builder()
        .timeout(timeout)
        .build()
        .context("Failed to build HTTP client")?;

    let response = client
        .get(url)
        .send()
        .await
        .context("Failed to send HTTP request")?;

    if response.status() != reqwest::StatusCode::OK {
        return Err(anyhow::anyhow!(
            "Invalid status code: {}",
            response.status()
        ));
    }

    let data = response
        .bytes()
        .await
        .context("Failed to read response body")?
        .to_vec();
    Ok(data)
}

#[allow(dead_code)]
pub struct FileMeta {
    pub size: i64,
    pub hash: String,
}

pub async fn get_file_meta(url: &str) -> Result<FileMeta> {
    let timeout = Duration::from_secs(10);
    let client = Client::builder()
        .timeout(timeout)
        .build()
        .context("Failed to build HTTP client")?;

    let response = client
        .head(url)
        .send()
        .await
        .context("Failed to send HEAD request")?;

    if response.status() != reqwest::StatusCode::OK {
        log::error!("Invalid status code: {}", response.status());
        return Err(anyhow::anyhow!(
            "Invalid status code: {}",
            response.status()
        ));
    }

    let size = response
        .headers()
        .get(header::CONTENT_LENGTH)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse().ok())
        .context("Invalid content length")?;

    let hash = response
        .headers()
        .get(header::ETAG)
        .and_then(|value| value.to_str().ok())
        .map(|value| value.to_string())
        .unwrap_or_default();

    log::debug!(
        "File metadata - URL: {}, Size: {}, Hash: {}",
        url,
        size,
        hash
    );

    Ok(FileMeta { size, hash })
}

pub fn calculate_file_sha256(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let hash_bytes = hasher.finalize();
    hex::encode(hash_bytes)
}

pub fn calculate_file_sha1(data: &[u8]) -> String {
    let mut hasher = Sha1::new();
    hasher.update(data);
    let hash_bytes = hasher.finalize();
    hex::encode(hash_bytes)
}

pub fn calculate_file_md5(data: &[u8]) -> String {
    let mut hasher = Md5::new();
    hasher.update(data);
    let hash_bytes = hasher.finalize();
    hex::encode(hash_bytes)
}

pub async fn create_output_file(output_filename: &str) -> Result<File> {
    let output_path = Path::new(output_filename);
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)
            .await
            .context("Failed to create parent directory")?;
    }
    let output_file = File::create(output_path).context("Failed to create output file")?;
    Ok(output_file)
}
