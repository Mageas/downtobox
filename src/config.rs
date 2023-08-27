use directories::{ProjectDirs, UserDirs};
use serde::{de::Error, Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::Write,
    path::PathBuf,
};

use crate::{eyre, Context, Result};

#[derive(Deserialize, Serialize, Debug)]
pub struct Config {
    #[serde(deserialize_with = "deserialize_local_path")]
    pub local_path: String,
    pub destination_path: String,
    pub api_key: String,
}

impl Config {
    pub fn init() -> Result<Config> {
        let dir = ProjectDirs::from("dev", "latruiterouge", "downtobox")
            .ok_or_else(|| eyre!("Unable to determine the config path"))?;
        let toml = format!("{}/config.toml", dir.config_dir().display());

        // Create a config if it does not exist
        if !dir.config_dir().exists() {
            fs::create_dir_all(dir.config_dir()).context("Unable to create the config file")?;
            let mut file = File::create(&toml).context("Unable to create the config file")?;
            let toml_config =
                toml::to_string(&Self::default()).context("Unable to create the config file")?;
            file.write_all(toml_config.as_bytes())
                .context("Unable to create the config file")?;
        }

        // Load config
        let config = fs::read_to_string(&toml).context("Unable to open the config file")?;
        let config: Config = toml::from_str(&config).context("Unable to parse the config file")?;

        Ok(config)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            local_path: String::from("~/Downloads"),
            destination_path: String::from("//"),
            api_key: String::from("uptobox_api_key"),
        }
    }
}

fn deserialize_local_path<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;
    if value.starts_with("~/") {
        match UserDirs::new() {
            Some(user) => {
                let user = user.home_dir();
                let path = PathBuf::from(value.trim_start_matches("~/"));
                let path = user.join(path);
                Ok(path.to_string_lossy().to_string())
            }
            None => Err(Error::custom(
                "Unable to parse 'local_path' from the config",
            )),
        }
    } else {
        Ok(value)
    }
}
