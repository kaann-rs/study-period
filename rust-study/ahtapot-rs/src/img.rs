use crate::aht::AhtError;
use image::{DynamicImage, ImageReader, imageops};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub struct Image {
    img: DynamicImage,
}

impl Image {
    pub fn new(path: &Path) -> Result<Self, AhtError> {
        let img = ImageReader::open(path)?.decode()?;

        Ok(Self { img })
    }

    pub fn resize(mut self, width: u32, height: u32) -> Self {
        self.img = self
            .img
            .resize(width, height, imageops::FilterType::Lanczos3);

        self
    }

    pub fn save(&self, output: &Path, name: &str) -> Result<(), AhtError> {
        let mut save_path = PathBuf::new();

        if !(output.is_dir()) {
            fs::create_dir_all(output)?
        }

        save_path.push(output);
        save_path.push(Path::new(name).with_extension("png"));

        Ok(self.img.save(save_path)?)
    }
}
