use std::{
  collections::BTreeMap, fmt::{self, Display, Formatter}, path::{Path, PathBuf},
};

use quote::Tokens;
use regex::Regex;

use resource::{Resource, ResourceKind};

use super::*;

lazy_static! {
  static ref STEM_RE: Regex = Regex::new("^[a-zA-Z][a-zA-Z0-9_-]*$").unwrap();
}

#[derive(PartialEq, Debug)]
pub struct Module {
  children: BTreeMap<String, Resource>,
}

impl Module {
  pub fn from_path(module_path: impl AsRef<Path>) -> Result<Module, Error> {
    let module_path = module_path.as_ref();
    let metadata = module_path
      .metadata()
      .map_err(|io_error| (io_error, module_path))?;

    if !metadata.is_dir() {
      return Err(Error::IsNotDirectory {
        path: module_path.to_path_buf(),
      });
    }

    let mut children: BTreeMap<String, Resource> = BTreeMap::new();
    let mut identifiers: BTreeMap<String, PathBuf> = BTreeMap::new();

    for child in module_path
      .read_dir()
      .map_err(|io_error| (io_error, module_path))?
    {
      let child = child.map_err(|io_error| (io_error, module_path))?;
      let path = child.path().to_path_buf();
      let metadata = path.metadata().map_err(|io_error| (io_error, module_path))?;
      let filename = path.strip_prefix(module_path).unwrap();

      let stem = filename
        .file_stem()
        .unwrap()
        .to_str()
        .ok_or_else(|| Error::NonUnicodePath { path: path.clone() })?;

      if stem.starts_with(".") {
        continue;
      }

      if !STEM_RE.is_match(stem) {
        return Err(Error::FilenameNotValidRustIdentifier { path: path.clone() });
      }

      let extension = if let Some(extension) = filename.extension() {
        Some(extension
          .to_str()
          .ok_or_else(|| Error::NonUnicodePath { path: path.clone() })?)
      } else {
        None
      };

      let identifier = if metadata.is_dir() {
        stem.replace("-", "_").to_lowercase()
      } else {
        stem.replace("-", "_").to_uppercase()
      };

      if let Some(a) = identifiers.remove(&identifier) {
        return Err(Error::ConflictingIdentifiers {
          a,
          b: path.clone(),
          identifier: identifier,
        });
      }

      let kind = if metadata.is_dir() {
        ResourceKind::Module
      } else {
        match extension {
          None => return Err(Error::MissingExtension { path: path.clone() }),
          Some("blob") => ResourceKind::Blob,
          Some("jpg") | Some("jpeg") | Some("png") | Some("gif") | Some("webp") | Some("tif")
          | Some("tiff") | Some("tga") | Some("bmp") | Some("ico") | Some("hdr") | Some("pbm")
          | Some("pam") | Some("ppm") | Some("pgm") => ResourceKind::Image,
          Some(other) => return Err(Error::UnsupportedExtension { path: path.clone() }),
        }
      };

      children.insert(identifier, kind.resource(path.clone())?);
    }

    Ok(Module { children })
  }

  pub fn tokens(self) -> Tokens {
    let children = self
      .children
      .into_iter()
      .map(|(identifier, child)| child.tokens(identifier));

    quote! {
      use pxl::Image;

      #(#children)*
    }
  }
}

#[cfg(test)]
mod test {
  use super::*;

  use image::{Rgba, RgbaImage};
  use std::fs;
  use tempfile::{Builder, TempDir};

  fn tempdir() -> TempDir {
    Builder::new().prefix("pxl-build-test").tempdir().unwrap()
  }

  #[test]
  fn empty() {
    let tempdir = tempdir();

    let module = Module::from_path(tempdir.path()).unwrap();

    assert!(module.children.is_empty());
  }

  #[test]
  fn blob() {
    let tempdir = tempdir();

    let mut blob_path = tempdir.path().to_path_buf();
    blob_path.push("file.blob");

    fs::write(blob_path, "hello");

    let module = Module::from_path(tempdir.path()).unwrap();

    assert_eq!(module.children.len(), 1);
    assert_eq!(
      module.children["FILE"],
      Resource::Blob {
        bytes: b"hello".iter().cloned().collect()
      }
    );
  }

  #[test]
  fn image() {
    let tempdir = tempdir();

    let mut blob_path = tempdir.path().to_path_buf();
    blob_path.push("file.png");

    let image = RgbaImage::from_pixel(1, 1, Rgba { data: [1, 2, 3, 4] });

    image.save(blob_path);

    let module = Module::from_path(tempdir.path()).unwrap();

    let pixels = vec![1.0 / 255.0, 2.0 / 255.0, 3.0 / 255.0, 4.0 / 255.0];

    assert_eq!(module.children.len(), 1);
    assert_eq!(
      module.children["FILE"],
      Resource::Image {
        width: 1,
        height: 1,
        pixels
      }
    );
  }

  #[test]
  fn submodule() {
    let tempdir = tempdir();
    let path = tempdir.path();

    let mut submodule_path = path.to_path_buf();
    submodule_path.push("sub");

    fs::create_dir(&submodule_path).unwrap();

    let mut blob_path = submodule_path.clone();
    blob_path.push("file.blob");

    fs::write(blob_path, "hello");

    let module = Module::from_path(tempdir.path()).unwrap();

    assert_eq!(module.children.len(), 1);
    if let Resource::Module {
      module: Module { ref children },
    } = module.children["sub"]
    {
      assert_eq!(children.len(), 1);
      assert_eq!(
        children["FILE"],
        Resource::Blob {
          bytes: b"hello".iter().cloned().collect()
        }
      );
    } else {
      panic!("bad submodule");
    }
  }
}
