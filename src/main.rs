use std::process;

use anyhow::Result;
use clap::Parser;
use log::LevelFilter;

mod downloader;
mod utils;

#[derive(Parser)]
struct Args {
    #[clap(long, help = "Keep the downloaded chunk files")]
    keep_chunks: bool,
    #[clap(long, default_value = "output.txt", help = "Output filename")]
    output: String,
    #[clap(long, help = "Enable debug mode")]
    debug: bool,
    #[clap(required = true, help = "Chunk URLs")]
    chunk_urls: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let log_level = if args.debug {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    };
    env_logger::builder().filter_level(log_level).init();

    if let Err(err) =
        downloader::download_file(&args.chunk_urls, args.keep_chunks, &args.output).await
    {
        log::error!("Error: {}", err);
        process::exit(1);
    }

    log::info!("File downloaded successfully!");
    Ok(())
}
