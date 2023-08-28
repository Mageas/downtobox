use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use reqwest::Client;
use std::{cmp::min, fs::File, io::Write};

use crate::{eyre, Result};

pub struct Download {}

/// Download the file from uptobox
impl Download {
    pub async fn start(client: &Client, url: &str, path: &str) -> Result<()> {
        let res = client
            .get(url)
            .send()
            .await
            .map_err(|e| eyre!("Unable to download '{url}' ({e})"))?;

        let size = res
            .content_length()
            .ok_or_else(|| eyre!("Failed to get content length from '{}'", &url))?;

        let pb = Self::set_progress_bar(size, url)?;

        let mut file =
            File::create(path).map_err(|e| eyre!("Failed to create the file '{path}' ({e})"))?;
        let mut downloaded: u64 = 0;
        let mut stream = res.bytes_stream();

        while let Some(bytes) = stream.next().await {
            let chunk = bytes.map_err(|e| eyre!("Error while downloading '{url}' ({e})"))?;
            Write::write_all(&mut file, &chunk)
                .map_err(|e| eyre!("Error while savin '{url}' ({e})"))?;
            let new = min(downloaded + (chunk.len() as u64), size);
            downloaded = new;
            pb.set_position(new);
        }

        pb.finish_with_message(format!("Downloaded {} to {}", url, path));

        Ok(())
    }

    /// Set the progress bar
    fn set_progress_bar(size: u64, url: &str) -> Result<ProgressBar> {
        let pb = ProgressBar::new(size);
        pb.set_style(ProgressStyle::with_template("{msg}\n {spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta}) {bytes_per_sec}")
            .map_err(|e| eyre!("The progress bar cannot be initialized ({e})"))?
            .with_key("eta", |state: &ProgressState, w: &mut dyn std::fmt::Write| write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap())
            .progress_chars("#>-"));
        pb.set_message(format!("Downloading {}", url));
        Ok(pb)
    }
}
