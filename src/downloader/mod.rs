use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::sync::Arc;

use anyhow::{Context, Result};
use futures::future::join_all;
use tokio::fs;
use tokio::sync::Mutex;

use crate::utils::*;

const CHUNK_FOLDER: &str = "./cache";

pub async fn download_file(
    chunk_urls: &[String],
    persistent_chunk: bool,
    output_filename: &str,
) -> Result<()> {
    let num_chunks = chunk_urls.len();
    let chunk_hashes = vec![String::new(); num_chunks];

    let chunk_hashes = Arc::new(Mutex::new(chunk_hashes));

    let futures = chunk_urls.iter().enumerate().map(|(i, chunk_url)| {
        let chunk_hashes = Arc::clone(&chunk_hashes);
        async move {
            let chunk_meta = get_file_meta(chunk_url).await?;
            chunk_hashes.lock().await[i] = chunk_meta.hash;
            Ok(())
        }
    });

    let results = join_all(futures).await;

    for result in results {
        if let Err(err) = result {
            log::error!("Failed to get size of chunk: {}", err);
            return Err(err);
        }
    }

    let output_file = create_output_file(output_filename)
        .await
        .context("Failed to create output file")?;

    download_and_write_chunks(
        chunk_urls,
        &chunk_hashes.lock().await,
        persistent_chunk,
        output_file,
    )
    .await?;

    Ok(())
}

async fn download_and_write_chunks(
    chunk_urls: &[String],
    chunk_hashes: &[String],
    persistent_chunk: bool,
    mut output_file: File,
) -> Result<()> {
    let num_chunks = chunk_urls.len();
    let chunk_data = Arc::new(Mutex::new(vec![None; num_chunks]));

    let futures = chunk_urls.iter().enumerate().map(|(i, chunk_url)| {
        let chunk_data = Arc::clone(&chunk_data);
        async move {
            let chunk_filename = Path::new(chunk_url).file_name().unwrap().to_str().unwrap();
            let chunk_file_path = Path::new(CHUNK_FOLDER).join(chunk_filename);

            if persistent_chunk {
                if let Ok(data) = fs::read(&chunk_file_path).await {
                    chunk_data.lock().await[i] = Some(data);
                    return Ok(());
                }
            }

            log::debug!("Downloading chunk {} from {}", i, chunk_url);

            let data = make_http_request(chunk_url).await?;
            chunk_data.lock().await[i] = Some(data.clone());

            if persistent_chunk {
                if let Err(err) = fs::write(&chunk_file_path, data).await {
                    log::error!("Error saving chunk file {}: {}", chunk_filename, err);
                }
            }

            Ok(())
        }
    });

    let results = join_all(futures).await;

    for result in results {
        if let Err(err) = result {
            log::error!("Error occurred during chunk download: {}", err);
            return Err(err);
        }
    }

    let chunk_data = chunk_data.lock().await;
    for (i, data) in chunk_data.iter().enumerate() {
        if let Some(data) = data {
            if !chunk_hashes[i].is_empty() {
                verify_chunk_hash(data, &chunk_hashes[i], i);
            }

            output_file
                .write_all(data)
                .context("Failed to write chunk data to output file")?;
        }
    }

    Ok(())
}

fn verify_chunk_hash(data: &[u8], expected_hash: &str, chunk_index: usize) {
    let (hash, err) = match expected_hash.len() {
        32 => (calculate_file_md5(data), None),
        40 => (calculate_file_sha1(data), None),
        64 => (calculate_file_sha256(data), None),
        _ => (String::new(), Some("Unknown hash length".to_string())),
    };

    if let Some(err) = err {
        log::warn!("Chunk {}: {}", chunk_index, err);
        return;
    }

    if let Some(err_msg) = err {
        log::warn!("Chunk {}: {}", chunk_index, err_msg);
        return;
    }

    if hash != expected_hash {
        log::warn!(
            "Hash mismatch for chunk {}: expected {}, actual {}",
            chunk_index,
            expected_hash,
            hash
        );
        return;
    }

    log::info!("Hash matched for chunk {}", chunk_index);
}
