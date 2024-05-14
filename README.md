# Sky Lantern

## Overview

Sky Lantern is an HTTP/HTTPS based multi-source downloader that can download a file from multiple sources simultaneously.
This project simulates a scenario where a file is split into multiple chunks and distributed across different servers (e.g. BitTorrent).

**Since we don't have tracker servers in this project, the user must provide the URLs of all the chunk URLs to simulate a manifest file.**
(This project also not support `Transfer-Encoding: chunked` response as our chunks are on different servers)

## Usage

To download a file, run the following command:

```bash
# Support arguments:
# --keep-chunks: Keep the downloaded chunks as cache, will not download the same chunk again.
# --output: The output file path
# --debug: Print debug information

# Example:
cargo run -- --keep-chunks --output result.txt --debug <chunk1_url> <chunk2_url> <chunk3_url> ...
# or use the provided run.sh script with sample URLs
bash ./run.sh
```

## Project Structure

- `main.rs`: The entry point of the application, handling user input and starting the download process.
- `downloader/mod.rs`: Contains the main logic for the multi-source downloader, including chunk reassembly and hash verification.
- `utils/mod.rs`: Provides utility functions, such as making HTTP requests and calculating file hashes.
