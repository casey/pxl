use common::*;

use image;

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
}

impl Resource {
  pub fn from_path_and_extension(
    path: impl AsRef<Path>,
    extension: &str,
  ) -> Result<Resource, Error> {
    let path = path.as_ref();
    println!("rerun-if-changed={}", path.display());
    match extension {
      "blob" => {
        let bytes = fs::read(path).map_err(|io_error| (io_error, path))?;
        Ok(Resource::Blob { bytes })
      }
      "jpg" | "jpeg" | "png" => {
        let image = image::open(path)
          .map_err(|image_error| (image_error, path))?
          .to_rgba();
        let dimensions = image.dimensions();
        let width = dimensions.0 as usize;
        let height = dimensions.0 as usize;
        let pixels = image
          .pixels()
          .flat_map(|pixel| {
            pixel
              .data
              .iter()
              .cloned()
              .map(|subpixel| f32::from(subpixel) / 255.0)
          })
          .collect();
        Ok(Resource::Image {
          width,
          height,
          pixels,
        })
      }
      other => Err(Error::UnsupportedExtension {
        path: path.to_path_buf(),
        extension: other.to_string(),
      }),
    }
  }

  pub fn type_tokens(&self) -> TokenStream {
    match self {
      Resource::Blob { .. } => quote!(&[u8]),
      Resource::Image { .. } => quote!(Image),
    }
  }

  pub fn tokens(self) -> TokenStream {
    match self {
      Resource::Blob { bytes } => {
        let bytes = bytes
          .into_iter()
          .map(|b| LitInt::new(u64::from(b), IntSuffix::None, Span::call_site()));
        quote! {
          &[#(#bytes,)*]
        }
      }
      Resource::Image {
        width,
        height,
        pixels,
      } => {
        let pixels = (0..width * height).map(|n| {
          let i = n * 4;
          let red = LitFloat::new(f64::from(pixels[i]), FloatSuffix::None, Span::call_site());
          let green = LitFloat::new(
            f64::from(pixels[i + 1]),
            FloatSuffix::None,
            Span::call_site(),
          );
          let blue = LitFloat::new(
            f64::from(pixels[i + 2]),
            FloatSuffix::None,
            Span::call_site(),
          );
          let alpha = LitFloat::new(
            f64::from(pixels[i + 3]),
            FloatSuffix::None,
            Span::call_site(),
          );
          quote! {
            Pixel {
              red: #red,
              green: #green,
              blue: #blue,
              alpha: #alpha,
            }
          }
        });

        let width = LitInt::new(width as u64, IntSuffix::None, Span::call_site());
        let height = LitInt::new(height as u64, IntSuffix::None, Span::call_site());
        quote! {
          Image {
            width: #width,
            height: #height,
            pixels: &[#(#pixels,)*],
          }
        }
      }
    }
  }
}
