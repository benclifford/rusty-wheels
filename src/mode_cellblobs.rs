use crate::leds;
use crate::structs::{FrameState, Mode};
use rand::Rng;
use std::io;
use std::time::Duration;

struct CellBlobs<const LEDS: usize> {
    cells: [bool; LEDS],
    last_now: Duration,
}

pub fn create_cellblobs<const LEDS: usize>() -> Box<dyn Mode<LEDS>> {
    let mut cells = [false; LEDS];

    for n in 0..LEDS {
        let r = rand::thread_rng().gen_range(0, 2);
        cells[n] = r == 1;
    }

    Box::new(CellBlobs {
        cells: cells,
        last_now: Default::default(),
    })
}

impl<const LEDS: usize> Mode<LEDS> for CellBlobs<LEDS> {
    fn render(
        &self,
        side: leds::Side,
        leds: &mut leds::WheelLEDs<LEDS>,
        _frame: &FrameState,
    ) -> io::Result<()> {
        let mut new_cells = [false; LEDS];
        let mut old_cells = self.cells;
        let mut changed = true;

        while changed {
            changed = false;
            for led in 1..LEDS - 1 {
                let before = if old_cells[led - 1] { 1 } else { 0 };
                let here = if old_cells[led] { 1 } else { 0 };
                let after = if old_cells[led + 1] { 1 } else { 0 };

                let s = before + here + after;

                new_cells[led] = if s >= 2 { true } else { false };
                if new_cells[led] != old_cells[led] {
                    changed = true
                }
            }
            old_cells = new_cells;
        }

        for led in 0..LEDS {
            let colour = if new_cells[led] {
                (255, 0, 0)
            } else {
                (0, 0, 0)
            };
            leds.set(side, led, colour);
        }

        Ok(())
    }

    fn step(&mut self, frame: &FrameState) -> io::Result<()> {
        let timestep = frame.spin_length / 256;

        let next_now = self.last_now + timestep;

        if frame.now <= next_now {
            return Ok(());
        }

        self.last_now = frame.now;

        let c = rand::thread_rng().gen_range(0, LEDS);
        self.cells[c] = !self.cells[c];
        Ok(())
    }
}
