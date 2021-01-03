extern crate spidev;

use spidev::SpiModeFlags;
use spidev::Spidev;
use spidev::SpidevOptions;

use std::io;
use std::io::BufWriter;
use std::io::Write;

fn setup_leds() -> io::Result<BufWriter<Spidev>> {
    println!("Configuring LEDs");
    let spi = create_spi()?;
    Ok(BufWriter::new(spi))
}

fn create_spi() -> io::Result<Spidev> {
    let mut spi = Spidev::open("/dev/spidev0.0")?;

    let options = SpidevOptions::new()
        .bits_per_word(8)
        .max_speed_hz(1_000_000)
        .mode(SpiModeFlags::SPI_MODE_0)
        .build();
    spi.configure(&options)?;

    Ok(spi)
}

fn send_led(w: &mut BufWriter<Spidev>, m: u8, r: u8, g: u8, b: u8) -> io::Result<usize> {
    w.write(&[m, b, g, r])
}

fn send_rgb(w: &mut BufWriter<Spidev>, rgb: (u8, u8, u8)) -> io::Result<usize> {
    let (r, g, b) = rgb;
    send_led(w, 255, r, g, b)
}

/// A Side identifies a side of the physical wheel
#[derive(PartialEq, Eq, Copy, Clone)]
pub enum Side {
    /// The first set of LEDs on the string
    Left,
    /// The second set of LEDs on the string
    Right
}

pub const SIDES: [Side;2] = [Side::Left, Side::Right];

/// WheelLEDs provides some kind of array-like access to setting individual
/// LEDs which can then be dumped out in one frame.
/// It provides a mutable collection of RGB tuples, one entry for each LED,
/// structure in two dimensions by radial position and side
/// and a way to dump that array onto the physical LED array.

pub struct WheelLEDs {
    led_stream: BufWriter<Spidev>,

    /// The LEDs are stored in this array in the order that they should
    /// be sent down the SPI channel.
    leds: [(u8, u8, u8); 46],
}

impl WheelLEDs {
    /// set a pixel, side 0 or 1, pixel 0 ... 22
    /// pixel number starts at the centre of the wheel, on both
    /// sides.
    pub fn set(&mut self, side: Side, pixel: usize, rgb: (u8, u8, u8)) {
        assert!(pixel <= 22, "pixel number too large");
        match side {
            Side::Left => self.leds[pixel] = rgb,
            Side::Right => self.leds[23 + (22 - pixel)] = rgb
        }
    }

    /// Writes the stored LED values to the physical strip over SPI
    pub fn show(&mut self) -> io::Result<()> {
        // initialise LED strip to recieve values from the start
        send_led(&mut self.led_stream, 0, 0, 0, 0)?;

        for led in 0..46 {
            send_rgb(&mut self.led_stream, self.leds[led])?;
        }

        // padding for clocking purposes down-strip
        send_led(&mut self.led_stream, 0, 0, 0, 0)?;
        send_led(&mut self.led_stream, 0, 0, 0, 0)?;
        send_led(&mut self.led_stream, 0, 0, 0, 0)?;
        send_led(&mut self.led_stream, 0, 0, 0, 0)?;

        self.led_stream.flush()?;

        Ok(())
    }

    pub fn new() -> WheelLEDs {
        let led_stream = match setup_leds() {
            Ok(leds) => leds,
            Err(e) => panic!("LED setup returned an error: {}", e),
        };

        WheelLEDs {
            led_stream: led_stream,
            leds: [(0, 0, 0); 46],
        }
    }
}
