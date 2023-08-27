use reqwest::{
    multipart::{Form, Part},
    Body, Client, Response,
};
use tokio::fs::File;
use tokio_util::codec::{BytesCodec, FramedRead};

use rs_uptobox::Uptobox;

use crate::{eyre, Context, Result};

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
            .context(eyre!("Cannot access '{path}'"))?;

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
            .context(eyre!("Unable to upload '{path}'"))?;

        if res.status().is_success() {
            Ok(res)
        } else {
            Err(eyre!("Unable to upload '{title}'"))
        }
    }
}
