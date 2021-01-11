use crate::leds;
use crate::structs::{FrameState, Mode};
use std::io;

struct Stepper {
    radius: usize,
    last_spin_pos: f32,
}

impl Mode for Stepper {
    fn render(
        &self,
        side: leds::Side,
        leds: &mut leds::WheelLEDs,
        _frame: &FrameState,
    ) -> io::Result<()> {
        for led in 0..23 {
            leds.set(side, led, (0, 0, 0));
        }
        leds.set(side, self.radius, (255, 128, 0));
        Ok(())
    }

    fn step(&mut self, frame: &FrameState) -> io::Result<()> {
        if frame.spin_pos < self.last_spin_pos {
            self.radius = (self.radius + 1) % 23;
        }
        self.last_spin_pos = frame.spin_pos;
        Ok(())
    }
}

pub fn construct_stepper() -> Box<dyn Mode> {
    Box::new(Stepper {
        radius: 0,
        last_spin_pos: 0.0,
    })
}
