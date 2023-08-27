use reqwest::{
    multipart::{Form, Part},
    Body, Client, Response,
};
use tokio::fs::File;
use tokio_util::codec::{BytesCodec, FramedRead};

use rs_uptobox::Uptobox;

use crate::{eyre, Result};

pub struct Upload {}

/// Upload the file to uptobox
impl Upload {
    pub async fn start<'a>(
        client: &Client,
        uptobox: &Uptobox,
        path: &str,
        title: &str,
    ) -> Result<Response> {
        let url = uptobox.get_upload_url().await?.upload_link;
        let url = format!("https://{}", &url[2..]);

        let file: File = File::open(path)
            .await
            .map_err(|e| eyre!("Unable to open the file '{path}' ({e})"))?;

        let stream = FramedRead::new(file, BytesCodec::new());
        let body = Body::wrap_stream(stream);

        let file = Part::stream(body)
            .file_name(title.to_owned())
            .mime_str("video/x-matroska")?;

        let form = Form::new().part("file", file);

        let res = client
            .post(url)
            .multipart(form)
            .send()
            .await
            .map_err(|e| eyre!("Unable to upload the file '{path}' ({e})"))?;

        if res.status().is_success() {
            Ok(res)
        } else {
            Err(eyre!("Unable to upload the file '{path}' (Unknown error)"))
        }
    }
}
