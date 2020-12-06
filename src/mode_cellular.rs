use crate::leds;
use crate::structs::{FrameState, Mode};
use rand::Rng;
use std::io;

struct CellularState {
    cells: [bool; 23],
}

impl Mode for CellularState {
    fn render(
        &self,
        side: usize,
        leds: &mut leds::WheelLEDs,
        _frame: &FrameState,
    ) -> io::Result<()> {
        for led in 0..23 {
            let colour = if self.cells[led] {
                (0, 32, 64)
            } else {
                (0, 0, 0)
            };
            leds.set(side, led, colour);
        }

        Ok(())
    }

    fn step(&mut self, _frame: &FrameState) -> io::Result<()> {
        let automata_number = 146;

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
            let new_state = (automata_number >> bit) & 0b1;

            new_cells[cell] = new_state == 1;
        }

        self.cells = new_cells;

        Ok(())
    }
}

pub fn construct_cellular() -> Box<dyn Mode> {
    let mut cells = [false; 23];

    for n in 0..23 {
        let r = rand::thread_rng().gen_range(0, 2);
        cells[n] = r == 1;
    }

    Box::new(CellularState { cells: cells })
}
