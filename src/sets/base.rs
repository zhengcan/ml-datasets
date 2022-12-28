use std::{
    fs::{create_dir_all, read, write},
    io::Read,
    path::PathBuf,
    str::FromStr,
};

use bytes::{Buf, Bytes};
use flate2::{read::GzDecoder, GzBuilder};
use reqwest::Url;

use crate::utils::{self, download, DatasetError};

pub enum FileFormat {
    Raw,
    Tar,
    TarGzip,
    Gzip,
}

pub struct DatasetRemote {
    pub url: String,
    pub size: u64,
    pub md5: Option<String>,
}

impl DatasetRemote {
    pub fn new(url: impl ToString, size: u64, md5: Option<String>) -> Self {
        Self {
            url: url.to_string(),
            size,
            md5,
        }
    }

    pub async fn download_to(&self, dst: &PathBuf) -> Result<PathBuf, DatasetError> {
        let url = Url::from_str(&self.url)?;
        let file_name = url.path_segments().unwrap().last().unwrap();
        let file = dst.join(file_name);
        if self.is_exists(&file) {
            return Ok(file);
        }

        let bytes = download(&self.url, Some(self.size)).await?;
        if let Some(md5) = &self.md5 {
            if &utils::md5(&bytes) != md5 {
                return Err(DatasetError::ValidateError);
            }
        }

        if let Some(parent) = file.parent() {
            create_dir_all(parent)?;
        }
        write(file.as_path(), &bytes)?;
        Ok(file)
        // match self.file_format {
        //     FileFormat::Raw => {
        //         write(dst, &bytes)?;
        //     }
        //     FileFormat::Gzip => {
        //         let mut gz = GzDecoder::new(bytes.reader());
        //         let mut buf = vec![];
        //         gz.read(&mut buf)?;
        //         write(dst, &buf)?;
        //     }
        //     FileFormat::Tar => {
        //         let mut archive = tar::Archive::new(bytes.reader());
        //         archive.unpack(dst)?;
        //     }
        //     FileFormat::TarGzip => {
        //         let gz = GzDecoder::new(bytes.reader());
        //         let mut archive = tar::Archive::new(gz);
        //         archive.unpack(dst)?;
        //     }
        // }
        // Ok(dst.clone())
    }

    fn is_exists(&self, file: &PathBuf) -> bool {
        if !file.exists() {
            return false;
        }

        if let Ok(metadata) = file.metadata() {
            if metadata.len() != self.size {
                return false;
            }
        } else {
            return false;
        }

        if let Some(md5) = &self.md5 {
            if let Ok(bytes) = read(file) {
                if utils::md5(&bytes).as_str() != md5 {
                    return false;
                }
            } else {
                return false;
            }
        }

        return true;
    }
}

pub struct DatasetCache {}

impl DatasetCache {
    pub fn new() -> Self {
        Self {}
    }
}

#[cfg(test)]
mod tests {}
