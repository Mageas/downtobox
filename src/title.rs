use itertools::Itertools;
use regex::Regex;

use crate::{
    eyre,
    matroska::{Matroska, MatroskaLang, MatroskaSource},
    Result,
};

pub struct Title {}

impl Title {
    /// Generate the file name
    pub fn generate_film_title(
        path: &str,
        title: &str,
        langs: &str,
        source: &str,
    ) -> Result<String> {
        let file = Matroska::new(path)?;
        let title = title.trim().replace(' ', ".");
        let langs = Self::get_langs(langs)?;
        let srouce = Self::get_source(source);
        let resolution = file.get_resolution()?;
        let audio_codecs = file.get_audio_codecs()?;
        let video_codecs = file.get_video_codecs()?;

        let output = [
            title,
            langs,
            resolution,
            srouce,
            audio_codecs,
            video_codecs,
            String::from("mkv"),
        ]
        .iter()
        .filter(|t| !t.is_empty())
        .join(".");

        Ok(output)
    }

    // (?:S|s)[0-9]{1,3}(?:E|e)[0-9]{1,3}|(?:E|e)[0-9]{1,3}

    /// Generate the show name
    pub fn generate_show_title(
        path: &str,
        file_name: &str,
        title: &str,
        langs: &str,
        source: &str,
    ) -> Result<String> {
        let file = Matroska::new(path)?;
        let title = title.trim().replace(' ', ".");
        let langs = Self::get_langs(langs)?;
        let episode = Self::get_episode(file_name)?;
        let srouce = Self::get_source(source);
        let resolution = file.get_resolution()?;
        let audio_codecs = file.get_audio_codecs()?;
        let video_codecs = file.get_video_codecs()?;

        let output = [
            title,
            episode,
            langs,
            resolution,
            srouce,
            audio_codecs,
            video_codecs,
            String::from("mkv"),
        ]
        .iter()
        .filter(|t| !t.is_empty())
        .join(".");

        Ok(output)
    }

    /// Parse the episode from the file name
    pub fn get_episode(file_name: &str) -> Result<String> {
        let regex = Regex::new(r#"(?:S|s)[0-9]{1,3}(?:E|e)[0-9]{1,3}|(?:E|e)[0-9]{1,3}"#)?;
        Ok(regex
            .captures(file_name)
            .ok_or_else(|| eyre!("Unable to parse the episode from '{file_name}' (regex: (?:S|s)[0-9]{{1,3}}(?:E|e)[0-9]{{1,3}}|(?:E|e)[0-9]{{1,3}} )"))?
            .get(0)
            .ok_or_else(|| eyre!("Unable to parse the episode from '{file_name}' (regex: (?:S|s)[0-9]{{1,3}}(?:E|e)[0-9]{{1,3}}|(?:E|e)[0-9]{{1,3}} )"))?
            .as_str()
            .to_string())
    }

    /// Langs
    pub fn get_langs(langs: &str) -> Result<String> {
        Ok(langs
            .trim()
            .split(' ')
            .map(MatroskaLang::from)
            .map(|l| l.to_string())
            .join("."))
    }

    /// Rip
    pub fn get_source(source: &str) -> String {
        let mut source: Vec<String> = source
            .trim()
            .split(' ')
            .map(MatroskaSource::from)
            .map(|r| r.to_string())
            .collect();
        source.sort();
        source.iter().map(|m| m.to_string()).join(".").to_string()
    }
}
