use clap::Parser;
use color_eyre::eyre::{eyre, Report, Result};
use reqwest::Client;
use rs_uptobox::Uptobox as UptoboxApi;
use std::path::Path;

mod cli;
mod command;
mod config;
mod download;
mod file;
mod matroska;
mod title;
mod upload;
mod uptobox;

use cli::*;
use command::Command;
use config::Config;
use download::Download;
use file::File;
use title::Title;
use upload::Upload;
use uptobox::Uptobox;

enum UploadType {
    Show,
    Film,
}

#[tokio::main]
async fn main() -> Result<(), Report> {
    color_eyre::install()?;

    Command::check()?;

    let config = Config::init()?;

    let args = Cli::parse();

    let api_key: &'static str = Box::leak(config.api_key.clone().into_boxed_str());

    let uptobox = UptoboxApi::new(api_key);
    let client = Client::new();

    match args.subcmd {
        CliSubCmd::Backup(backup) => match backup.subcmd {
            CliBackupCmd::Show(show) => {
                for link in &show.links {
                    backup_files(
                        (&uptobox, &client, &config),
                        &show,
                        &UploadType::Show,
                        link.trim(),
                    )
                    .await?;
                }
            }
            CliBackupCmd::Film(film) => {
                for link in &film.links {
                    backup_files(
                        (&uptobox, &client, &config),
                        &film,
                        &UploadType::Film,
                        link.trim(),
                    )
                    .await?;
                }
            }
        },
        CliSubCmd::Upload(upload) => match upload.subcmd {
            CliUploadCmd::Show(show) => {
                for path in &show.paths {
                    upload_files(
                        (&uptobox, &client, &config),
                        &show,
                        &UploadType::Show,
                        path.trim(),
                    )
                    .await?;
                }
            }
            CliUploadCmd::Film(film) => {
                for path in &film.paths {
                    upload_files(
                        (&uptobox, &client, &config),
                        &film,
                        &UploadType::Film,
                        path.trim(),
                    )
                    .await?;
                }
            }
        },
    };

    Ok(())
}

/// Backup files section
async fn backup_files(
    (uptobox, client, config): (&UptoboxApi, &Client, &Config),
    args: &CliBackup,
    upload_type: &UploadType,
    link: &str,
) -> Result<()> {
    // File informations
    let file: File = File::new(uptobox, link, &config.local_path).await?;

    // Check if it is a matroska file
    check_file(&file.name)?;

    // Check if the episode can be parsed online
    if let UploadType::Show = upload_type {
        Title::get_episode(&file.path)?;
    }

    // Download the file
    Download::start(client, &file.url, &file.path).await?;

    // Generate the file name
    let title = match upload_type {
        UploadType::Show => Title::generate_show_title(
            &file.path,
            &file.name,
            &args.title,
            &args.languages,
            &args.sources,
        )?,
        UploadType::Film => {
            Title::generate_film_title(&file.path, &args.title, &args.languages, &args.sources)?
        }
    };

    // Update the title
    Command::update_title(&file.path, &title, true)?;

    // Upload the file
    println!("\n\nUploading '{}'\n", title);
    Upload::start(client, uptobox, &file.path, &title).await?;

    // Get the files
    let files = Uptobox::get_files(uptobox).await?;

    // Get the uploaded file
    let uploaded_file = Uptobox::get_uploaded_file(&files, &title)?;

    // Get destination folder
    let destination = Uptobox::get_destination_directory(uptobox, &config.destination_path).await?;

    // Move file to destination
    Uptobox::move_files_to_destination(
        uptobox,
        &uploaded_file.file_code,
        destination.current_folder.fld_id,
    )
    .await?;

    Ok(())
}

/// Upload files section
async fn upload_files(
    (uptobox, client, config): (&UptoboxApi, &Client, &Config),
    args: &CliUpload,
    upload_type: &UploadType,
    path: &str,
) -> Result<()> {
    let file_name = Path::new(path)
        .file_name()
        .ok_or_else(|| eyre!("Unable to extract file name"))?
        .to_str()
        .ok_or_else(|| eyre!("Unable to extract file name"))?;

    // Check if it is a matroska file
    check_file(file_name)?;

    // Generate the file name
    let title = match upload_type {
        UploadType::Show => Title::generate_show_title(
            path,
            file_name,
            &args.title,
            &args.languages,
            &args.sources,
        )?,
        UploadType::Film => {
            Title::generate_film_title(path, &args.title, &args.languages, &args.sources)?
        }
    };

    // Update the title
    Command::update_title(path, &title, true)?;

    // Upload the file
    println!("\n\nUploading '{}'\n", title);
    Upload::start(client, uptobox, path, &title).await?;

    // Get the files
    let files = Uptobox::get_files(uptobox).await?;

    // Get the uploaded file
    let uploaded_file = Uptobox::get_uploaded_file(&files, &title)?;

    // Get destination folder
    let destination = Uptobox::get_destination_directory(uptobox, &config.destination_path).await?;

    // Move file to destination
    Uptobox::move_files_to_destination(
        uptobox,
        &uploaded_file.file_code,
        destination.current_folder.fld_id,
    )
    .await?;

    Ok(())
}

fn check_file(name: &str) -> Result<()> {
    if !name.ends_with(".mkv") {
        Err(eyre!(
            "Downtobox is only compatible with matroska (.mkv) files"
        ))
    } else {
        Ok(())
    }
}
