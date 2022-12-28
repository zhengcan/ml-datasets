use std::{
    fs::{read, read_to_string},
    path::{Path, PathBuf},
};

use bytes::Bytes;
use flate2::read;
use ndarray::{Array, Array1, Array2, ArrayView1, ArrayView2, Axis, Slice};

use crate::utils::DatasetError;

use super::base::{DatasetCache, DatasetRemote, FileFormat};

pub struct Cifar10 {
    remote: DatasetRemote,
    cache_dir: PathBuf,
}

impl Default for Cifar10 {
    fn default() -> Self {
        Self::new(
            DatasetRemote::new(
                "https://www.cs.toronto.edu/~kriz/cifar-10-binary.tar.gz",
                170052171,
                Some(String::from("c32a1d4ab5d03f1284b67883e8d87530")),
            ),
            &Path::new("./data"),
        )
    }
}

impl Cifar10 {
    pub fn new(remote: DatasetRemote, cache_dir: &Path) -> Self {
        Self {
            remote,
            cache_dir: cache_dir.to_path_buf(),
        }
    }

    pub async fn prepare(&self) -> Result<CifarDataset, DatasetError> {
        let dir = self.cache_dir.join("cifar");
        let base = dir.join("cifar-10-batches-bin");
        let label_files = ["batches.meta.txt"];
        let data_files = [
            "data_batch_1.bin",
            "data_batch_2.bin",
            "data_batch_3.bin",
            "data_batch_4.bin",
            "data_batch_5.bin",
        ];
        let test_files = ["test_batch.bin"];

        let mut all_files = vec![];
        all_files.extend_from_slice(&label_files);
        all_files.extend_from_slice(&data_files);
        all_files.extend_from_slice(&test_files);

        let cache_ready = all_files.iter().all(|file| base.join(file).exists());
        if !cache_ready {
            self.remote.download_to(&dir).await?;
        }

        CifarDataset::new(
            label_files.iter().map(|f| base.join(f)).collect(),
            data_files.iter().map(|f| base.join(f)).collect(),
            test_files.iter().map(|f| base.join(f)).collect(),
        )
    }
}

pub struct Cifar100 {
    remote: DatasetRemote,
    cache_dir: PathBuf,
}

impl Default for Cifar100 {
    fn default() -> Self {
        Self::new(
            DatasetRemote::new(
                "https://www.cs.toronto.edu/~kriz/cifar-100-binary.tar.gz",
                168513733,
                Some(String::from("03b5dce01913d631647c71ecec9e9cb8")),
            ),
            &Path::new("./data"),
        )
    }
}

impl Cifar100 {
    pub fn new(remote: DatasetRemote, cache_dir: &Path) -> Self {
        Self {
            remote,
            cache_dir: cache_dir.to_path_buf(),
        }
    }

    pub async fn prepare(&self) -> Result<CifarDataset, DatasetError> {
        let dir = self.cache_dir.join("cifar");
        let base = dir.join("cifar-100-binary");
        let label_files = ["coarse_label_names.txt", "fine_label_names.txt"];
        let train_files = ["train.bin"];
        let test_files = ["test.bin"];

        let mut all_files = vec![];
        all_files.extend_from_slice(&label_files);
        all_files.extend_from_slice(&train_files);
        all_files.extend_from_slice(&test_files);

        let cache_ready = all_files.iter().all(|file| base.join(file).exists());
        if !cache_ready {
            self.remote.download_to(&dir).await?;
        }

        CifarDataset::new(
            label_files.iter().map(|f| base.join(f)).collect(),
            train_files.iter().map(|f| base.join(f)).collect(),
            test_files.iter().map(|f| base.join(f)).collect(),
        )
    }
}

#[derive(Debug)]
pub struct CifarDataset {
    labels: Vec<Vec<String>>,
    train_files: Vec<PathBuf>,
    test_files: Vec<PathBuf>,
}

impl CifarDataset {
    pub fn new(
        label_files: Vec<PathBuf>,
        train_files: Vec<PathBuf>,
        test_files: Vec<PathBuf>,
    ) -> Result<Self, DatasetError> {
        Ok(Self {
            labels: Self::read_labels(label_files)?,
            train_files,
            test_files,
        })
    }

    fn read_labels(files: Vec<PathBuf>) -> Result<Vec<Vec<String>>, DatasetError> {
        let mut labels = vec![];
        for file in files {
            let data = read_to_string(file)?;
            if !data.is_empty() {
                let mut inner = vec![];
                data.lines().for_each(|line| {
                    if !line.is_empty() {
                        inner.push(line.to_string())
                    }
                });
                if !inner.is_empty() {
                    labels.push(inner);
                }
            }
        }
        Ok(labels)
    }

    pub fn get_train_data(&self) -> Result<(Array2<u8>, Array2<u8>), DatasetError> {
        self.read_data(&self.train_files)
    }

    pub fn get_test_data(&self) -> Result<(Array2<u8>, Array2<u8>), DatasetError> {
        self.read_data(&self.test_files)
    }

    fn read_data(&self, files: &Vec<PathBuf>) -> Result<(Array2<u8>, Array2<u8>), DatasetError> {
        let label_count = self.labels.len();
        let image_size = 32 * 32 * 3;
        let row_size = label_count + image_size;

        let mut full = Array::zeros((0, row_size));
        for file in files {
            let data = read(file)?;
            let array = Array::from_shape_vec((data.len() / row_size, row_size), data)?;
            full.append(Axis(0), array.view())?;
        }

        let label = full.slice_axis(Axis(1), Slice::new(0, Some(label_count as isize), 1));

        let data = full.slice_axis(Axis(1), Slice::new(label_count as isize, None, 1));

        Ok((label.to_owned(), data.to_owned()))
    }
}

#[cfg(test)]
mod tests {
    use log::LevelFilter;
    use ndarray::Array;

    use crate::sets::cifar::{Cifar10, Cifar100};

    #[tokio::test]
    async fn test_cifar_10() {
        let _ = env_logger::builder()
            .format_module_path(false)
            .filter_level(LevelFilter::Debug)
            .try_init();

        let cifar10 = Cifar10::default().prepare().await.unwrap();
        println!("cifar10 = {:?}", cifar10);

        cifar10.get_train_data().unwrap();
        cifar10.get_test_data().unwrap();
    }

    #[tokio::test]
    async fn test_cifar_100() {
        let _ = env_logger::builder()
            .format_module_path(false)
            .filter_level(LevelFilter::Debug)
            .try_init();

        let cifar100 = Cifar100::default().prepare().await.unwrap();
        println!("cifar100 = {:?}", cifar100);

        cifar100.get_train_data().unwrap();
        cifar100.get_test_data().unwrap();
    }

    #[test]
    fn test_ndarray() {
        let buf = vec![1, 2, 3, 4, 5, 6];
        let array = Array::from_shape_vec((2, 3), buf).unwrap();
        println!("array = {}", array);
    }
}
