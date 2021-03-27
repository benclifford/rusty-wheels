use crate::leds;
use crate::structs::{FrameState, Mode};
use lazy_static::lazy_static;
use std::default::Default;
use std::io;
use std::time::Duration;

lazy_static! {
    static ref FONT: bdf::Font =
        bdf::open("/home/pi/src/rusty-wheels/font.bdf").expect("Valid font");
}

struct PhraseMode {
    bitmap: [u128; 7],
}

pub fn construct_phrase_mode<const LEDS: usize>() -> Box<dyn Mode<LEDS>> {
    println!("Iniialising phrase bitmap");
    let phrase = "@BENCLIFFORD";

    let bitmap = str_to_bitmap(phrase);

    println!("Initialised phrase bitmap");

    Box::new(PhraseMode { bitmap: bitmap })
}

pub fn construct_phrase_mode_hello<const LEDS: usize>() -> Box<dyn Mode<LEDS>> {
    println!("Iniialising phrase bitmap");
    let phrase = " HELLO  HELLO  HELLO ";

    let bitmap = str_to_bitmap(phrase);

    println!("Initialised phrase bitmap");

    Box::new(PhraseMode { bitmap: bitmap })
}

pub fn construct_phrase_fuck_boris<const LEDS: usize>() -> Box<dyn Mode<LEDS>> {
    println!("Iniialising phrase bitmap");
    let phrase = " FUCK BORIS ";

    let bitmap = str_to_bitmap(phrase);

    println!("Initialised phrase bitmap");

    Box::new(PhraseMode { bitmap: bitmap })
}

impl<const LEDS: usize> Mode<LEDS> for PhraseMode {
    fn render(
        &self,
        side: leds::Side,
        leds: &mut leds::WheelLEDs<LEDS>,
        frame: &FrameState,
    ) -> io::Result<()> {
        helper_render_bitmap(&self.bitmap, side, leds, frame)
    }
}

struct SpeedoMode {
    canvas: PhraseMode,
    last_change: Duration,
    last_spin_pos: f32,
    counter: u32,
}

pub fn construct_speedo_mode<const LEDS: usize>() -> Box<dyn Mode<LEDS>> {
    println!("Initialising speedo phrase bitmap: constructing phrase");
    let phrase = "  - KM/H";

    println!("Initialising speedo phrase bitmap: rendering text");
    let bitmap = str_to_bitmap(phrase);

    println!("Initialising speedo phrase bitmap: complete");

    Box::new(SpeedoMode {
        canvas: PhraseMode { bitmap: bitmap },
        last_change: Default::default(),
        last_spin_pos: 0.0,
        counter: 0,
    })
}

impl<const LEDS: usize> Mode<LEDS> for SpeedoMode {
    fn render(
        &self,
        side: leds::Side,
        leds: &mut leds::WheelLEDs<LEDS>,
        frame: &FrameState,
    ) -> io::Result<()> {
        helper_render_bitmap(&self.canvas.bitmap, side, leds, frame)
    }

    fn step(&mut self, frame: &FrameState) -> io::Result<()> {
        // this will fire on the first spin boundary to occur after a second boundary.
        if self.last_change + Duration::from_secs(1) < frame.now
            && frame.spin_pos < self.last_spin_pos
        {
            // given spin_length as a duration
            //    that is time / rot

            let time_per_rot = frame.spin_length;

            let s_per_rot: f32 = (time_per_rot.as_millis() as f32) / 1000.0;

            // println!("s_per_rot = {}", s_per_rot);

            let h_per_rot: f32 = s_per_rot / 60.0 / 60.0;
            // println!("h_per_rot = {}", h_per_rot);

            // there could be an infinity here... if duration is 0
            let rot_per_hour = 1.0 / h_per_rot;
            // println!("rot_per_hour = {}", rot_per_hour);

            if rot_per_hour.is_infinite() {
                // println!("infinity path");
                let phrase = format!("XXX km/h");
                self.canvas.bitmap = str_to_bitmap(&phrase);
                self.counter += 1;
                self.last_change = frame.now;
            } else {
                // this is based on characteristics of the bike wheel
                // which is 20" for my bike
                const WHEEL_M_PER_ROT: f32 = 1.59;
                const WHEEL_KM_PER_ROT: f32 = WHEEL_M_PER_ROT / 1000.0;

                let kmh = WHEEL_KM_PER_ROT * rot_per_hour;

                let phrase = format!("{:>3.0} km/h", kmh);
                self.canvas.bitmap = str_to_bitmap(&phrase);
                self.counter += 1;
                self.last_change = frame.now;
            }
        }
        self.last_spin_pos = frame.spin_pos;
        Ok(())
    }
}

/// This can render a 128 pixel wide, 7 bit pixel high bitmap
fn helper_render_bitmap<const LEDS: usize>(
    row: &[u128; 7],
    side: leds::Side,
    wheel_leds: &mut leds::WheelLEDs<LEDS>,
    framestate: &FrameState,
) -> io::Result<()> {
    // establish a blank canvas
    for led in 0..LEDS {
        wheel_leds.set(side, led, (0, 0, 0));
    }
    let mut pixel;

    pixel = ((framestate.spin_pos % 1.0) * 128.0) as u8;

    // if spin pos too high, maybe we'll go over a limit

    if pixel > 127 {
        pixel = 127;
    }

    // flip pixels on other side because rotation is the other way round
    if side == leds::Side::Left {
        pixel = 127 - pixel;
    }

    for n in 0..7 {
        let r = ((row[n] & (1 << pixel)) >> pixel) & 1;
        let colour = if r != 0 {
            match side {
                leds::Side::Left => (255, 32, 0),  // amber
                leds::Side::Right => (56, 255, 0), // green - from wikipedia phosper wavelength converted to rgb
            }
        } else {
            (0, 0, 0)
        };
        wheel_leds.set(side, LEDS - 1 - n, colour);
    }

    Ok(())
}

fn str_to_bitmap(phrase: &str) -> [u128; 7] {
    let mut bitmap: [u128; 7] = [0, 0, 0, 0, 0, 0, 0];
    for c in phrase.chars() {
        let glyph = FONT
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

    bitmap
}
