use rand::{distributions::Alphanumeric, Rng};
use std::process::Command as Execute;

use crate::{eyre, Result};

pub struct Command {}

impl Command {
    /// Check if mkvpropedit is installed
    pub fn check() -> Result<()> {
        match Execute::new("mkvpropedit").arg("--version").output() {
            Ok(_) => Ok(()),
            Err(_) => Err(eyre!("This program requires mkvpropedit")),
        }
    }

    /// Update the file title
    pub fn update_title(path: &str, title: &str, hash: bool) -> Result<()> {
        let hash = match hash {
            true => format!(
                " - {}",
                rand::thread_rng()
                    .sample_iter(&Alphanumeric)
                    .take(7)
                    .map(char::from)
                    .collect::<String>()
            ),
            false => String::new(),
        };

        let cmd = Execute::new("mkvpropedit")
            .arg(path)
            .arg("--edit")
            .arg("info")
            .arg("--set")
            .arg(format!(r#"title={title}{hash}"#))
            .output()
            .map_err(|e| eyre!("Unable to update the title of '{path}' ({e})"))?;

        if cmd.status.success() {
            Ok(())
        } else {
            Err(eyre!(
                "Unable to update the title of '{path}' (Unknown error)"
            ))
        }
    }

    /// Get mkv informations
    pub fn get_infos(path: &str) -> Result<String> {
        let output = Execute::new("mkvmerge")
            .arg("-F")
            .arg("json")
            .arg("--identify")
            .arg(path)
            .output()
            .map_err(|e| eyre!("Unable to retrieve file information for '{path}' ({e})"))?;
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}
