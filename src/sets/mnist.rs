use std::path::{Path, PathBuf};

use futures_util::{future::join_all, join};

use crate::utils::DatasetError;

use super::base::{DatasetRemote, FileFormat};

pub struct Mnist {
    train_data: DatasetRemote,
    train_label: DatasetRemote,
    test_data: DatasetRemote,
    test_label: DatasetRemote,
    cache_dir: PathBuf,
}

impl Default for Mnist {
    fn default() -> Self {
        Self::new(
            DatasetRemote::new(
                "http://yann.lecun.com/exdb/mnist/train-images-idx3-ubyte.gz",
                9912422,
                Some(String::from("f68b3c2dcbeaaa9fbdd348bbdeb94873")),
            ),
            DatasetRemote::new(
                "http://yann.lecun.com/exdb/mnist/train-labels-idx1-ubyte.gz",
                28881,
                Some(String::from("d53e105ee54ea40749a09fcbcd1e9432")),
            ),
            DatasetRemote::new(
                "http://yann.lecun.com/exdb/mnist/t10k-images-idx3-ubyte.gz",
                1648877,
                Some(String::from("9fb629c4189551a2d022fa330f9573f3")),
            ),
            DatasetRemote::new(
                "http://yann.lecun.com/exdb/mnist/t10k-labels-idx1-ubyte.gz",
                4542,
                Some(String::from("ec29112dd5afa0611ce80d1b7f02629c")),
            ),
            &Path::new("./data"),
        )
    }
}

impl Mnist {
    pub fn new(
        train_data: DatasetRemote,
        train_label: DatasetRemote,
        test_data: DatasetRemote,
        test_label: DatasetRemote,
        cache_dir: &Path,
    ) -> Self {
        Self {
            train_data,
            train_label,
            test_data,
            test_label,
            cache_dir: cache_dir.to_path_buf(),
        }
    }

    pub async fn prepare(&self) -> Result<MnistDataset, DatasetError> {
        let dir = self.cache_dir.join("mnist");
        let all_remotes = vec![
            &self.train_data,
            &self.train_label,
            &self.test_data,
            &self.test_label,
        ];

        let all_files: Vec<_> = all_remotes
            .iter()
            .map(|remote| remote.download_to(&dir))
            .collect();
        let all_files: Result<Vec<_>, _> = join_all(all_files).await.into_iter().collect();
        let all_files = all_files?;

        MnistDataset::new(
            all_files[0].clone(),
            all_files[1].clone(),
            all_files[2].clone(),
            all_files[3].clone(),
        )
    }
}

pub struct MnistDataset {
    train_data: PathBuf,
    train_labels: Vec<String>,
    test_data: PathBuf,
    test_labels: Vec<String>,
}

impl MnistDataset {
    pub fn new(
        train_data: PathBuf,
        train_label: PathBuf,
        test_data: PathBuf,
        test_label: PathBuf,
    ) -> Result<Self, DatasetError> {
        Ok(Self {
            train_data,
            train_labels: Self::read_labels(train_label)?,
            test_data,
            test_labels: Self::read_labels(test_label)?,
        })
    }

    fn read_labels(file: PathBuf) -> Result<Vec<String>, DatasetError> {
        Ok(vec![])
    }
}
