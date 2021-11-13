use crate::helpers::fraction_to_rgb;
use crate::leds;
use crate::structs::{FrameState, Mode};
use rand::prelude::SliceRandom;
use rand::Rng;
use std::io;

struct CellularState {
    automata_number: u8,
    rgb: (u8, u8, u8),
    cells: [bool; 23],
}

impl<const LEDS: usize> Mode<LEDS> for CellularState {
    fn render(
        &self,
        side: leds::Side,
        leds: &mut leds::WheelLEDs<LEDS>,
        _frame: &FrameState,
    ) -> io::Result<()> {
        for led in 0..23 {
            let colour = if self.cells[led] { self.rgb } else { (0, 0, 0) };
            leds.set(side, led, colour);
        }

        Ok(())
    }

    fn step(&mut self, _frame: &FrameState) -> io::Result<()> {
        let mut new_cells = self.cells;

        for cell in 0..23 {
            let downcell = if cell < 1 {
                self.cells[22]
            } else {
                self.cells[cell - 1]
            };
            let upcell = if cell > 21 {
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

        Ok(())
    }
}

/// These look good in rotating mode
const PRETTY_AUTOMATA: &[u8] = &[73, 105, 146];

pub fn construct_cellular<const LEDS: usize>() -> Box<dyn Mode<LEDS>> {
    let mut cells = [false; 23];

    for n in 0..23 {
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
        rgb: rgb,
        automata_number: a_n,
        cells: cells,
    })
}
