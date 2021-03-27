use crate::leds;
use crate::structs::{FrameState, Mode};
use std::io;

struct EdgeStrobe {
    last_spin_pos: f32,
}

impl<const LEDS: usize> Mode<LEDS> for EdgeStrobe {
    fn render(
        &self,
        side: leds::Side,
        leds: &mut leds::WheelLEDs<LEDS>,
        frame: &FrameState,
    ) -> io::Result<()> {
        let colour = if frame.spin_pos < self.last_spin_pos {
            (255, 64, 0)
        } else {
            (0, 0, 0)
        };
        for led in 0..LEDS {
            leds.set(side, led, colour);
        }
        Ok(())
    }

    fn step(&mut self, frame: &FrameState) -> io::Result<()> {
        self.last_spin_pos = frame.spin_pos;
        Ok(())
    }
}

pub fn construct_edge_strobe<const LEDS: usize>() -> Box<dyn Mode<LEDS>> {
    Box::new(EdgeStrobe { last_spin_pos: 0.0 })
}
