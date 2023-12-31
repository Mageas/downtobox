use regex::Regex;
use rs_uptobox::{GetDownloadUrl, GetDownloadUrlResponse, Uptobox};
use std::{fs::DirBuilder, path::Path};

use crate::{eyre, Result};

pub struct File {
    pub name: String,
    pub url: String,
    pub dir: String,
    pub path: String,
}

/// File informations
impl File {
    pub async fn new(uptobox: &Uptobox, url: &str, dir: &str) -> Result<Self> {
        let file_code = Self::parse_file_code(url)?;

        // Get the file name
        let name = uptobox
            .get_files_informations(vec![&file_code])
            .await
            .map_err(|e| eyre!("Unable to retreive informations for '{url}' ({e})"))?;
        let name = &name
            .get(0)
            .ok_or_else(|| eyre!("Unable to retreive informations for '{url}'"))?
            .file_name;

        // Get the file url
        let url = uptobox
            .get_download_url(GetDownloadUrl::new(&file_code))
            .await
            .map_err(|e| eyre!("Unable to fetch the download link for '{url}' ({e})"))?;
        let url = match url {
            GetDownloadUrlResponse::Link(url) => url.dl_link,
            GetDownloadUrlResponse::Wait(_) => {
                return Err(eyre!("A premium account is needed to use this software"));
            }
        };

        let dir = Self::check_output_dir(dir)?;

        Ok(Self {
            name: name.clone(),
            url,
            dir: dir.clone(),
            path: format!("{}/{}", dir, name),
        })
    }

    /// Parse the file code from the url
    fn parse_file_code(url: &str) -> Result<String> {
        let regex = Regex::new(
            r#"https://(?:uptobox|uptostream).[a-zA-Z]+/(?P<file_code>[a-zA-Z0-9]{12})"#,
        )?;
        Ok(regex
            .captures(url)
            .ok_or_else(|| eyre!("Unable to parse the file code for '{url}' (regex: https://(?:uptobox|uptostream).[a-zA-Z]+/(?P<file_code>[a-zA-Z0-9]{{12}}) )"))?
            .name("file_code")
            .ok_or_else(|| eyre!("Unable to parse the file code for '{url}' (regex: https://(?:uptobox|uptostream).[a-zA-Z]+/(?P<file_code>[a-zA-Z0-9]{{12}}) )"))?
            .as_str()
            .to_string())
    }

    /// Check if the output directory exists
    fn check_output_dir(path: &str) -> Result<String> {
        let mut display = String::from(path);
        let path = Path::new(&path);
        if !path.exists() {
            DirBuilder::new().create(path)?;
        }

        if !path.is_dir() {
            return Err(eyre!("The local path '{display}' is not a directory"));
        }

        if display.ends_with('/') {
            display.pop();
        }

        Ok(display)
    }
}
