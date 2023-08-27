use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use reqwest::Client;
use std::{cmp::min, fs::File, io::Write};

use crate::{eyre, Context, Result};

pub struct Download {}

/// Download the file from uptobox
impl Download {
    pub async fn start(client: &Client, url: &str, path: &str) -> Result<()> {
        let res = client
            .get(url)
            .send()
            .await
            .context(eyre!("Unable to download '{url}'"))?;

        let size = res
            .content_length()
            .ok_or_else(|| eyre!("Failed to get content length from '{}'", &url))?;

        let pb = ProgressBar::new(size);
        pb.set_style(ProgressStyle::with_template("{msg}\n {spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta}) {bytes_per_sec}")
            .context("Cannot initialize the progress bar")?
            .with_key("eta", |state: &ProgressState, w: &mut dyn std::fmt::Write| write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap())
            .progress_chars("#>-"));
        pb.set_message(format!("Downloading {}", url));

        let mut file = File::create(path).context(eyre!("Failed to create file '{}'", path))?;
        let mut downloaded: u64 = 0;
        let mut stream = res.bytes_stream();

        while let Some(bytes) = stream.next().await {
            let chunk = bytes.context(eyre!("Error while downloading file"))?;
            Write::write_all(&mut file, &chunk).context(eyre!("Error while writing to file"))?;
            let new = min(downloaded + (chunk.len() as u64), size);
            downloaded = new;
            pb.set_position(new);
        }

        pb.finish_with_message(format!("Downloaded {} to {}", url, path));

        Ok(())
    }
}
