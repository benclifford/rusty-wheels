use crate::helpers::fraction_to_rgb;
use crate::leds;
use crate::structs::{FrameState, Mode, RGB24};
use rand::prelude::SliceRandom;
use rand::Rng;
use std::io;
use std::time::Duration;

struct CellularState<const LEDS: usize> {
    automata_number: u8,
    rgb: RGB24,
    cells: [bool; LEDS],
    last_now: Duration,
}

impl<const LEDS: usize> CellularState<LEDS> {
    fn step_cells(&mut self) {
        let mut new_cells = self.cells;

        for cell in 0..LEDS {
            let downcell = if cell < 1 {
                self.cells[LEDS - 1]
            } else {
                self.cells[cell - 1]
            };
            let upcell = if cell >= LEDS - 1 {
                self.cells[0]
            } else {
                self.cells[cell + 1]
            };

            let mut bit: u8 = 0;
            if downcell {
                bit = bit | 0b001;
            }
            if self.cells[cell] {
                bit = bit | 0b010;
            }
            if upcell {
                bit = bit | 0b100;
            }

            // bit now identifies a bit number in the automata number
            let new_state = (self.automata_number >> bit) & 0b1;

            new_cells[cell] = new_state == 1;
        }

        self.cells = new_cells;
    }

    fn render_leds(&self, side: leds::Side, leds: &mut leds::WheelLEDs<LEDS>) {
        for led in 0..LEDS {
            let colour = if self.cells[led] { self.rgb } else { (0, 0, 0) };
            leds.set(side, led, colour);
        }
    }
}

impl<const LEDS: usize> Mode<LEDS> for CellularState<LEDS> {
    fn render(
        &self,
        side: leds::Side,
        leds: &mut leds::WheelLEDs<LEDS>,
        _frame: &FrameState,
    ) -> io::Result<()> {
        self.render_leds(side, leds);
        Ok(())
    }

    fn step(&mut self, frame: &FrameState) -> io::Result<()> {
        let timestep = frame.spin_length / 128;

        let next_now = self.last_now + timestep;

        if frame.now > next_now {
            self.step_cells();
            self.last_now = frame.now;
        }
        Ok(())
    }
}

/// These look good in rotating mode
const PRETTY_AUTOMATA: &[u8] = &[18, 73, 105, 146];

pub fn construct_cellular<const LEDS: usize>() -> Box<dyn Mode<LEDS>> {
    let mut cells = [false; LEDS];

    for n in 0..LEDS {
        let r = rand::thread_rng().gen_range(0, 2);
        cells[n] = r == 1;
    }

    let a_n = match PRETTY_AUTOMATA.choose(&mut rand::thread_rng()) {
        Some(x) => *x,
        None => panic!("Could not choose an automata number"),
    };

    let hue = rand::thread_rng().gen_range(0.0, 1.0);
    let rgb = fraction_to_rgb(hue, Some(0.25));

    Box::new(CellularState {
        rgb,
        automata_number: a_n,
        cells,
        last_now: Default::default(),
    })
}
