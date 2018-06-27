//! common includes

pub use std::{
  collections::BTreeMap, env, fs::{self, File}, io::{self, prelude::*}, path::{Path, PathBuf},
};

pub use proc_macro2::Span;
pub use quote::Tokens;
pub use regex::Regex;
pub use syn::{FloatSuffix, Ident, IntSuffix, LitFloat, LitInt};

pub use error::Error;
pub use module::Module;
pub use resource::Resource;
