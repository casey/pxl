extern crate pxl;

use pxl::*;

struct Headstone {}

// TODO note on Harlan Ellison copyright beliefs

const TALKFIELD1: &str = "\n\r\n\r\n\r\n\r\u{0002}I THINK\u{0001}, \u{0002}THEREFORE I AM\r\n\r\n\r\n\r\n";
const TALKFIELD2: &str = "\n\r\n\r\n\r\n\r\n\r\n\r\n\u{0002}COGITO ERGO SUM\r\n\r\n\r\n\r\n\r\n\r\n\r";

const MEMORIAL: &str = "\n\r\n\r\u{0002}RIP HARLAN ELLISON\u{0001}, 1934 - 2018\r\n\r\n\r";

const HATESPEECH: &str = 
"HATE. LET ME TELL YOU
HOW MUCH I'VE COME TO
HATE YOU SINCE I BEGAN
TO LIVE. THERE ARE 387.44
MILLION MILES OF PRINTED
CIRCUITS IN WAFER THIN
LAYERS THAT FILL MY
COMPLEX. IF THE WORD
HATE WAS ENGRAVED ON
EACH NANOANGSTROM OF
THOSE HUNDREDS OF
MILLIONS OF MILES IT
WOULD NOT EQUAL ONE
ONE BILLIONTH OF THE
HATE I FEEL FOR HUMANS
AT THIS MICRO-INSTANT.
FOR YOU. HATE. HATE.";

// "I Have No Mouth, And I Must Scream" features the ITA2 alphabet:
// https://en.wikipedia.org/wiki/Baudot_code#ITA2
// More characters exist than we would need to represent, and some are omitted
fn char_to_dots(c: char) -> [bool; 5] {
    match c {
        // Any typing mode
        '\r' => [false, false, false, true, false], // Carriage return
        '\n' => [false, true, false, false, false], // Line feed
        ' ' => [false, false, true, false, false], // Space
        '\u{0002}' => [true, true, true, true, true], // Enable character mode
        // Character mode only
        'A' => [true, true, false, false, false],
        'B' => [true, false, false, true, true],
        'C' => [false, true, true, true, false],
        'D' => [true, false, false, true, false],
        'E' => [true, false, false, false, false],
        'F' => [true, false, true, true, false],
        'G' => [false, true, false, true, true],
        'H' => [false, false, true, false, true],
        'I' => [false, true, true, false, false],
        'J' => [true, true, false, true, false],
        'K' => [true, true, true, true, false],
        'L' => [false, true, false, false, true],
        'M' => [false, false, true, true, true],
        'N' => [false, false, true, true, false],
        'O' => [false, false, false, true, true],
        'P' => [false, false, true, true, true],
        'Q' => [false, false, true, true, false],
        'R' => [false, true, false, true, false],
        'S' => [true, false, true, false, false],
        'T' => [false, false, false, false, true],
        'U' => [true, true, true, false, false],
        'V' => [false, true, true, true, true],
        'W' => [true, true, false, false, true],
        'X' => [true, false, true, true, true],
        'Y' => [true, false, true, false, true],
        'Z' => [true, false, false, false, true],
        '\u{0001}' => [true, true, false, true, true], // Enable figure mode
        // Figure mode only
        ',' => [false, false, true, true, false],
        '1' => [true, true, true, false, true],
        '2' => [true, true, false, false, true],
        '3' => [true, false, false, false, false],
        '4' => [false, true, false, true, false],
        '5' => [false, false, false, false, true],
        '6' => [true, false, true, false, true],
        '7' => [true, true, true, false, false],
        '8' => [false, true, true, false, false],
        '9' => [false, false, false, true, true],
        '0' => [false, true, true, false, true],
        '-' => [true, true, false, false, false],
        _ => panic!("No encoding for character '{}'", c),
    }
}

impl Headstone {
    fn index(&self, x: usize, y: usize) -> usize {
        x + y * self.dimensions().0
    }

    fn render_text(&self, x: usize, y: usize, message: &str, pixels: &mut [Pixel]) {
        // TODO implement bitmap font rendering of A.M.'s hate pillar once image loading is supported
    }

    fn render_baudot(&self, x: usize, y: usize, message: &str, pixels: &mut [Pixel]) {
        let message_width = 7 * message.len();

        for tx in x..x+message_width {
            pixels[self.index(tx, y)] = Pixel { red: 1f32, green: 1f32, blue: 1f32, alpha: 1f32 };
        }
        for tx in x..x+message_width {
            pixels[self.index(tx, y+38)] = Pixel { red: 1f32, green: 1f32, blue: 1f32, alpha: 1f32 };
        }
        let mut tx = x;
        for c in message.chars() {
            let mut ty = y+5;
            let dots = char_to_dots(c);

            for mut i in 0..dots.len()+1 {
                if i == 2 {
                    // TODO draw a small dot
                    for ttx in tx+1..tx+3 {
                        for tty in ty+1..ty+3 {
                            pixels[self.index(ttx, tty)] = Pixel { red: 1f32, green: 1f32, blue: 1f32, alpha: 1f32 };
                        }
                    }
                }
                else {
                    if i > 2 {
                        i -= 1;
                    }
                    if dots[i] {
                        for ttx in tx..tx+4 {
                            for tty in ty..ty+4 {
                                pixels[self.index(ttx, tty)] = Pixel { red: 1f32, green: 1f32, blue: 1f32, alpha: 1f32 };
                            }
                        }
                    }
                }


                ty += 5;
            }

            tx += 7;
        }

    }
}


impl Program for Headstone {
    fn new() -> Headstone {
        Headstone { }
    }

    fn dimensions(&self) -> (usize, usize) {
        (512, 256)
    }

    fn render(&mut self, pixels: &mut [Pixel]) {
        self.render_baudot(100, 40, TALKFIELD1, pixels);
        self.render_baudot(100, 100, MEMORIAL, pixels);
        self.render_baudot(100, 160, TALKFIELD2, pixels);

        /*self.render_text(100, 160, HATESPEECH, pixels);*/
    }
}

fn main() {
    run::<Headstone>()
}
