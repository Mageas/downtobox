use rand::{distributions::Alphanumeric, Rng};
use std::process::Command as Execute;

use crate::{eyre, Context, Result};

pub struct Command {}

impl Command {
    /// Check if mkvpropedit is installed
    pub fn check() -> Result<()> {
        match Execute::new("mkvpropedit").arg("--version").output() {
            Ok(_) => Ok(()),
            Err(_) => Err(eyre!("mkvpropedit is needed to use this program")),
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
            .context(eyre!("Cannot update the title for '{path}'"))?;

        if cmd.status.success() {
            Ok(())
        } else {
            Err(eyre!("Unable to rename '{}' for '{hash}'", title))
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
            .context(eyre!("Cannot get file informations for '{path}'"))?;
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}
