use std::{fs, path::PathBuf};

use image;
use quote::Tokens;

use super::*;

#[derive(PartialEq, Debug)]
pub enum Resource {
  Blob {
    bytes: Vec<u8>,
  },
  Image {
    width: usize,
    height: usize,
    pixels: Vec<f32>,
  },
  Module {
    module: Module,
  },
}

impl Resource {
  pub fn tokens(self, _identifier: String) -> Tokens {
    quote!()
  }
}

pub enum ResourceKind {
  Blob,
  Image,
  Module,
}

impl ResourceKind {
  pub fn resource(self, path: impl AsRef<Path>) -> Result<Resource, Error> {
    let path = path.as_ref();
    match self {
      ResourceKind::Blob => {
        let bytes = fs::read(path).map_err(|io_error| (io_error, path))?;
        Ok(Resource::Blob { bytes })
      }
      ResourceKind::Module => {
        let module = Module::from_path(path)?;
        Ok(Resource::Module { module })
      }
      ResourceKind::Image => {
        let image = image::open(path)
          .map_err(|image_error| (image_error, path))?
          .to_rgba();
        let dimensions = image.dimensions();
        let width = dimensions.0 as usize;
        let height = dimensions.0 as usize;
        let pixels = image
          .pixels()
          .flat_map(|pixel| pixel.data.iter().map(|&subpixel| subpixel as f32 / 255.0))
          .collect();
        Ok(Resource::Image {
          width,
          height,
          pixels,
        })
      }
    }
  }
}
