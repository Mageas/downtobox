use chrono::DateTime;
use rs_uptobox::{GetFiles, GetFilesFiles, GetFilesResponse, Uptobox as UptoboxApi};
use std::cmp::Ordering;

use crate::{eyre, Result};

pub struct Uptobox {}

impl Uptobox {
    /// Get files
    pub async fn get_files(uptobox: &UptoboxApi) -> Result<GetFilesResponse> {
        uptobox
            .get_files(&GetFiles::new("//"))
            .await
            .map_err(|e| eyre!("Unable to fetch files from Uptobox ({e})"))
    }

    /// Get the uploaded file
    pub fn get_uploaded_file<'a>(
        files: &'a GetFilesResponse,
        name: &str,
    ) -> Result<&'a GetFilesFiles> {
        files
            .files
            .iter()
            .filter(|f| f.file_name == name)
            .max_by(|a, b| {
                match (
                    DateTime::parse_from_str(&a.file_created, "%Y-%m-%d %H:%M:%S"),
                    DateTime::parse_from_str(&b.file_created, "%Y-%m-%d %H:%M:%S"),
                ) {
                    (Ok(a_date), Ok(b_date)) => a_date.cmp(&b_date),
                    _ => Ordering::Equal,
                }
            })
            .ok_or_else(|| eyre!("Unable to find the uploaded file on uptobox"))
    }

    /// Get destination directory
    pub async fn get_destination_directory(
        uptobox: &UptoboxApi,
        path: &str,
    ) -> Result<GetFilesResponse> {
        uptobox
            .get_files(&GetFiles::new(path))
            .await
            .map_err(|e| eyre!("Unable to fetch destination folder on uptobox ({e})"))
    }

    /// Move files to destination
    pub async fn move_files_to_destination(
        uptobox: &UptoboxApi,
        file_code: &str,
        destination: usize,
    ) -> Result<usize> {
        uptobox
            .move_files(vec![file_code], destination)
            .await
            .map_err(|e| {
                eyre!("Unable to move uploaded file to destination folder on uptobox ({e})")
            })
    }
}
