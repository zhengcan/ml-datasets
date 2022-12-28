use std::str::FromStr;

use bytes::{Bytes, BytesMut};
use futures_util::stream::StreamExt;
use md5::{Digest, Md5};
use reqwest::Url;

#[derive(Debug)]
pub enum DatasetError {
    ParseError(url::ParseError),
    DownloadError(reqwest::Error),
    IoError(std::io::Error),
    ShapeError(ndarray::ShapeError),
    ValidateError,
}

impl From<url::ParseError> for DatasetError {
    fn from(value: url::ParseError) -> Self {
        Self::ParseError(value)
    }
}

impl From<reqwest::Error> for DatasetError {
    fn from(value: reqwest::Error) -> Self {
        Self::DownloadError(value)
    }
}

impl From<std::io::Error> for DatasetError {
    fn from(value: std::io::Error) -> Self {
        Self::IoError(value)
    }
}

impl From<ndarray::ShapeError> for DatasetError {
    fn from(value: ndarray::ShapeError) -> Self {
        Self::ShapeError(value)
    }
}

pub async fn download(url: &str, size: Option<u64>) -> Result<Bytes, DatasetError> {
    let response = reqwest::get(url).await?;
    match response.error_for_status() {
        Ok(response) => {
            let content_length = response.content_length();
            if let Some(content_length) = content_length {
                if let Some(size) = size {
                    if size != content_length {
                        return Err(DatasetError::ValidateError);
                    }
                }
            }

            let mut stream = response.bytes_stream();
            let mut bytes = BytesMut::new();
            let mut percent = 0u64;
            while let Some(item) = stream.next().await {
                if let Ok(chunk) = item {
                    bytes.extend(chunk);
                    if let Some(limit) = content_length {
                        let new_percent = bytes.len() as u64 * 100 / limit;
                        if new_percent > percent {
                            percent = new_percent;
                            println!("{} / {} ({}%)", bytes.len(), limit, percent);
                        }
                    }
                }
            }
            Ok(bytes.freeze())
        }
        Err(error) => Err(error.into()),
    }
}

pub fn md5(plain: impl AsRef<[u8]>) -> String {
    let mut md5 = Md5::new();
    md5.update(plain);
    hex::encode(md5.finalize())
}
