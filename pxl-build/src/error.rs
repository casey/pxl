use std::{
  io, path::{Path, PathBuf},
};

use image;

#[derive(Debug)]
pub enum Error {
  Io {
    io_error: io::Error,
    path: PathBuf,
  },
  Image {
    image_error: image::ImageError,
    path: PathBuf,
  },
  IsNotDirectory {
    path: PathBuf,
  },
  NonUnicodePath {
    path: PathBuf,
  },
  ConflictingIdentifiers {
    a: PathBuf,
    b: PathBuf,
    identifier: String,
  },
  FilenameNotValidRustIdentifier {
    path: PathBuf,
  },
  MissingExtension {
    path: PathBuf,
  },
  UnsupportedExtension {
    path: PathBuf,
  },
}

impl<'path> From<(io::Error, &'path Path)> for Error {
  fn from(tuple: (io::Error, &'path Path)) -> Error {
    Error::Io {
      io_error: tuple.0,
      path: tuple.1.to_path_buf(),
    }
  }
}

impl<'path> From<(image::ImageError, &'path Path)> for Error {
  fn from(tuple: (image::ImageError, &'path Path)) -> Error {
    Error::Image {
      image_error: tuple.0,
      path: tuple.1.to_path_buf(),
    }
  }
}
