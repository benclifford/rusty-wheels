use crate::leds;
use crate::structs::{FrameState, Mode};
use std::io;

pub fn render_bitmap(
    side: usize,
    wheel_leds: &mut leds::WheelLEDs,
    framestate: &FrameState,
) -> io::Result<()> {
    // render approx 50 pixels in half the rotation
    // or 100 pixels per full rotation
    let row: [u128; 7] = [
        0b01111101000100111001000100000001111000111001111000111000111000000,
        0b01000001000101000101001000000000100101000101000100010001000100000,
        0b01000001000101000001010000000000100101000101000100010001000000000,
        0b01111001000101000001100000000000111001000101111000010000111000000,
        0b01000001000101000001010000000000100101000101010000010000000100000,
        0b01000001000101000101001000000000100101000101001000010001000100000,
        0b01000000111000111001000100000001111000111001000100111000111000000,
    ];

    helper_render_bitmap(&row, side, wheel_leds, framestate)
}

struct PhraseMode {
    bitmap: [u128; 7],
}

pub fn construct_phrase_mode() -> Box<dyn Mode> {
    let mut bitmap: [u128; 7] = [0, 0, 0, 0, 0, 0, 0];

    println!("Iniialising phrase bitmap");
    let phrase = "@BENCLIFFORD";

    let font = bdf::open("/home/pi/src/rusty-wheels/font.bdf").expect("Valid font");

    for c in phrase.chars() {
        let glyph = font
            .glyphs()
            .get(&c)
            .unwrap_or_else(|| panic!("Cannot get glyph"));

        // first shift the existing display rightwards by the appropriate number of pixels
        let width = glyph.width() + 1;
        for row in 0..7 {
            // assume 7 rows here
            bitmap[row] <<= width;
        }

        // now render the glyph
        for row in (0 as usize)..7 {
            for col in 0..glyph.width() {
                if glyph.get(col, row as u32) {
                    bitmap[row] = bitmap[row] | 1 << (glyph.width() - col - 1);
                }
            }
        }
    }
    println!("Initialised phrase bitmap");

    Box::new(PhraseMode { bitmap: bitmap })
}

impl Mode for PhraseMode {
    fn render(
        &self,
        side: usize,
        leds: &mut leds::WheelLEDs,
        frame: &FrameState,
    ) -> io::Result<()> {
        helper_render_bitmap(&self.bitmap, side, leds, frame)
    }
}

/// This can render a 128 pixel wide, 7 bit pixel high bitmap
fn helper_render_bitmap(
    row: &[u128; 7],
    side: usize,
    wheel_leds: &mut leds::WheelLEDs,
    framestate: &FrameState,
) -> io::Result<()> {
    // establish a blank canvas
    for led in 0..23 {
        wheel_leds.set(side, led, (0, 0, 0));
    }
    let mut pixel;

    pixel = ((framestate.spin_pos % 1.0) * 128.0) as u8;

    // if spin pos too high, maybe we'll go over a limit

    if pixel > 127 {
        pixel = 127;
    }

    // flip pixels on other side because rotation is the other way round
    if side == 1 {
        pixel = 127 - pixel;
    }

    for n in 0..7 {
        let r = ((row[n] & (1 << pixel)) >> pixel) & 1;
        let colour = if r != 0 {
            match side {
                0 => (255, 32, 0), // amber
                1 => (56, 255, 0), // green - from wikipedia phosper wavelength converted to rgb
                _ => panic!("impossible side number"),
            }
        } else {
            (0, 0, 0)
        };
        wheel_leds.set(side, 22 - n, colour);
    }

    Ok(())
}
