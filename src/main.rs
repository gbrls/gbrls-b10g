use anyhow::{Context, Ok, Result};
use clap::{Parser, Subcommand};
use serde::Deserialize;
use serde_json::json;
use std::{collections::HashMap, error, fmt::format, io::Write, path::PathBuf, str::FromStr};

///! This module is responsible for syncing changes to the blog's contents and blog's configuration
mod blogfs;
mod dropbox;
mod markdown;

#[derive(Debug, Subcommand)]
enum Command {
    DownloadZip {
        #[arg(short, long)]
        source_path: String,
        #[arg(short, long)]
        dest_path: String,
    },
    MovePublished {
        #[arg(short, long)]
        zipfile: String,
        #[arg(short, long)]
        dest_dir: String,
    },
    DownloadAndPublish {
        #[arg(short = 's', long)]
        dropbox_source: String,
        #[arg(short, long)]
        dest_dir: String,
    },
}

#[derive(Parser, Debug)]
struct CliArgs {
    #[command(subcommand)]
    subcommand: Command,
}

fn tmpzip() -> PathBuf {
    std::env::temp_dir().join("data.zip")
}

async fn download_and_extract<P: AsRef<std::path::Path>>(
    source_path: &str,
    dest_path: P,
) -> Result<()> {
    let token = dropbox::fetch_sl_token_with_refresh().await?;
    let _file = dropbox::fetch_api_zip(&token, &source_path, &tmpzip()).await?;
    zip_extract::extract(
        &std::fs::File::open(&tmpzip()).context("Error opening FD downloaded from dropbox")?,
        dest_path.as_ref(),
        true,
    )
    .context("Error extracting dropbox zipfile")?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = CliArgs::parse();
    println!("{:?}", args);

    dotenv::dotenv().ok();

    match args.subcommand {
        Command::DownloadZip {
            source_path,
            dest_path,
        } => {
            let token = dropbox::fetch_sl_token_with_refresh().await?;
            let _file = dropbox::fetch_api_zip(&token, &source_path, &dest_path).await?;
            zip_extract::extract(
                &std::fs::File::open(&dest_path)
                    .context("Error opening FD downloaded from dropbox")?,
                &PathBuf::from("./tmp"),
                true,
            )
            .context("Error extracting dropbox zipfile")?;
        }

        Command::MovePublished { zipfile, dest_dir } => {
            blogfs::move_published(zipfile, dest_dir).context("Error moving published files")?;
        }
        Command::DownloadAndPublish {
            dropbox_source,
            dest_dir,
        } => {
            let tmpdir = std::env::temp_dir().join("journal-tmp");
            download_and_extract(&dropbox_source, &tmpdir).await?;
            blogfs::move_published(&tmpdir, &std::path::PathBuf::from_str(&dest_dir)?)?;

            std::fs::remove_dir_all(tmpdir)?;
        }
    }

    Ok(())
}
