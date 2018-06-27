extern crate pxl;

include!(concat!(env!("OUT_DIR"), "/resources.rs"));

#[test]
fn image() {
  assert_eq!(RGBA.width, 2);
  assert_eq!(RGBA.height, 2);
  assert_eq!(RGBA.pixels.len(), RGBA.width * RGBA.height);
  assert_eq!(
    RGBA.pixels[0],
    Pixel {
      red: 1.0,
      green: 0.0,
      blue: 0.0,
      alpha: 1.0
    }
  );
  assert_eq!(
    RGBA.pixels[1],
    Pixel {
      red: 0.0,
      green: 1.0,
      blue: 0.0,
      alpha: 1.0
    }
  );
  assert_eq!(
    RGBA.pixels[2],
    Pixel {
      red: 0.0,
      green: 0.0,
      blue: 1.0,
      alpha: 1.0
    }
  );
  assert_eq!(
    RGBA.pixels[3],
    Pixel {
      red: 0.0,
      green: 0.0,
      blue: 0.0,
      alpha: 0.0
    }
  );
}

#[test]
fn blob() {
  assert_eq!(ABCD, b"abcd\n");
}
