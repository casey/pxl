use common::*;

lazy_static! {
  static ref STEM_RE: Regex = Regex::new("^[a-zA-Z][a-zA-Z0-9_-]*$").unwrap();
}

#[derive(PartialEq, Debug)]
pub struct Module {
  items: BTreeMap<String, Item>,
}

#[derive(PartialEq, Debug)]
pub enum Item {
  Module { module: Module },
  Resource { resource: Resource },
}

impl Module {
  pub fn from_path(module_path: impl AsRef<Path>) -> Result<Module, Error> {
    let module_path = module_path.as_ref();
    println!("cargo:rerun-if-changed={}", module_path.display());
    let metadata = module_path
      .metadata()
      .map_err(|io_error| (io_error, module_path))?;

    if !metadata.is_dir() {
      return Err(Error::IsNotDirectory {
        path: module_path.to_path_buf(),
      });
    }

    let mut items: BTreeMap<String, Item> = BTreeMap::new();
    let mut identifiers: BTreeMap<String, PathBuf> = BTreeMap::new();

    for entry in module_path
      .read_dir()
      .map_err(|io_error| (io_error, module_path))?
    {
      let entry = entry.map_err(|io_error| (io_error, module_path))?;
      let path = entry.path().to_path_buf();
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

      let stem = stem.replace("-", "_");

      let extension = if let Some(extension) = filename.extension() {
        Some(extension
          .to_str()
          .ok_or_else(|| Error::NonUnicodePath { path: path.clone() })?)
      } else {
        None
      };

      let identifier = if metadata.is_dir() {
        stem.to_lowercase()
      } else {
        stem.to_uppercase()
      };

      if let Some(a) = identifiers.remove(&identifier) {
        return Err(Error::ConflictingIdentifiers {
          a,
          b: path.clone(),
          identifier: identifier,
        });
      }

      let item = if metadata.is_dir() {
        if let Some(extension) = extension {
          return Err(Error::UnsupportedExtension {
            path: path.clone(),
            extension: extension.to_string(),
          });
        }
        Item::Module {
          module: Module::from_path(&path)?,
        }
      } else {
        if let Some(extension) = extension {
          Item::Resource {
            resource: Resource::from_path_and_extension(&path, extension)?,
          }
        } else {
          return Err(Error::MissingExtension { path: path.clone() });
        }
      };
      items.insert(identifier, item);
    }

    Ok(Module { items })
  }

  pub fn tokens(self) -> Tokens {
    let items = self.items.into_iter().map(|(identifier, item)| {
      let ident = Ident::from(identifier);
      match item {
        Item::Resource { resource } => {
          let type_tokens = resource.type_tokens();
          let tokens = resource.tokens();
          quote! {
            pub static #ident: #type_tokens = #tokens;
          }
        }
        Item::Module { module } => {
          let tokens = module.tokens();
          quote! {
            mod #ident {
              #tokens
            }
          }
        }
      }
    });

    quote! {
      use pxl::{Pixel, Image};
      #(#items)*
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

    assert!(module.items.is_empty());
  }

  #[test]
  fn blob() {
    let tempdir = tempdir();

    let mut blob_path = tempdir.path().to_path_buf();
    blob_path.push("file.blob");

    fs::write(blob_path, "hello").unwrap();

    let module = Module::from_path(tempdir.path()).unwrap();

    assert_eq!(module.items.len(), 1);
    assert_eq!(
      module.items["FILE"],
      Item::Resource {
        resource: Resource::Blob {
          bytes: b"hello".iter().cloned().collect()
        }
      }
    );
  }

  #[test]
  fn image() {
    let tempdir = tempdir();

    let mut blob_path = tempdir.path().to_path_buf();
    blob_path.push("file.png");

    let image = RgbaImage::from_pixel(1, 1, Rgba { data: [1, 2, 3, 4] });

    image.save(blob_path).unwrap();

    let module = Module::from_path(tempdir.path()).unwrap();

    let pixels = vec![1.0 / 255.0, 2.0 / 255.0, 3.0 / 255.0, 4.0 / 255.0];

    assert_eq!(module.items.len(), 1);
    assert_eq!(
      module.items["FILE"],
      Item::Resource {
        resource: Resource::Image {
          width: 1,
          height: 1,
          pixels,
        }
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

    fs::write(blob_path, "hello").unwrap();

    let module = Module::from_path(tempdir.path()).unwrap();

    assert_eq!(module.items.len(), 1);
    if let Item::Module {
      module: Module { ref items },
    } = module.items["sub"]
    {
      assert_eq!(items.len(), 1);
      assert_eq!(
        items["FILE"],
        Item::Resource {
          resource: Resource::Blob {
            bytes: b"hello".iter().cloned().collect()
          }
        }
      );
    } else {
      panic!("bad submodule");
    }
  }
}
