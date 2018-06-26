use common::*;

use image;

/// Build errors
#[derive(Debug)]
pub enum Error {
  /// IO error
  Io {
    /// The underlying error
    io_error: io::Error,
    /// Path at which it occurred
    path: PathBuf,
  },
  /// Image decoding error
  Image {
    /// The underlying error
    image_error: image::ImageError,
    /// The path at which it occurred
    path: PathBuf,
  },
  /// The argument to `pxl_build::build` was not a directory
  IsNotDirectory {
    /// The argument to `pxl_build::build`
    path: PathBuf,
  },
  /// Indicates that a path could not be converted to unicode.
  ///
  /// Since `pxl_build` generates rust modules and statics with
  /// names derived from their paths, their paths must be valid
  /// unicode.
  NonUnicodePath {
    /// The offending path
    path: PathBuf,
  },
  /// Indicates that two resources mapped to identical identifiers
  /// in the same module.
  ///
  /// For example, `resources/PLAYER.png` and `resources/player.png`
  /// will both have the identifier `PLAYER`, and will trigger
  /// this error.
  ConflictingIdentifiers {
    /// The first path
    a: PathBuf,
    /// The second path
    b: PathBuf,
    /// The identifier that they path map to
    identifier: String,
  },
  /// Indicates that a filename contained characters that are
  /// not allowed inside of rust identifiers.
  ///
  /// Filename stems must match `[a-zA-Z][a-zA-Z0-9_-]*`.
  FilenameNotValidRustIdentifier {
    /// The offending path
    path: PathBuf,
  },
  /// Indicates that a resource filenmae had no extension.
  MissingExtension {
    /// The offending path
    path: PathBuf,
  },
  /// Indicates a filename had an unsupported extension.
  ///
  /// See the readme for supported extensions and filetypes.
  UnsupportedExtension {
    /// The offending path
    path: PathBuf,
    /// The offending extension
    extension: String,
  },
}

/// Converts from an `(io::Error, &Path)` tuple to a
/// pxl `Error`
impl<'path> From<(io::Error, &'path Path)> for Error {
  fn from(tuple: (io::Error, &'path Path)) -> Error {
    Error::Io {
      io_error: tuple.0,
      path: tuple.1.to_path_buf(),
    }
  }
}

/// Converts from an `(image::ImageError, &Path)` tuple to a
/// pxl `Error`
impl<'path> From<(image::ImageError, &'path Path)> for Error {
  fn from(tuple: (image::ImageError, &'path Path)) -> Error {
    Error::Image {
      image_error: tuple.0,
      path: tuple.1.to_path_buf(),
    }
  }
}
