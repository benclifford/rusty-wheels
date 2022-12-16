use crate::leds;
use crate::structs::{FrameState, Mode};
use std::io;

struct LineTracker {
    led: usize,
    func: for<'r> fn(&'r FrameState) -> usize,
}

impl<const LEDS: usize> Mode<LEDS> for LineTracker {
    fn render(
        &self,
        side: leds::Side,
        leds: &mut leds::WheelLEDs<LEDS>,
        _frame: &FrameState,
    ) -> io::Result<()> {
        for led in 0..LEDS {
            leds.set(side, led, (0, 0, 0));
        }
        leds.set(side, self.led, (255, 8, 0));
        Ok(())
    }

    fn step(&mut self, frame: &FrameState) -> io::Result<()> {
        let target = (self.func)(frame);

        if target > self.led {
            self.led += 1;
        } else if target < self.led {
            self.led -= 1;
        }

        Ok(())
    }
}

fn spiral_out<const LEDS: usize>(frame: &FrameState) -> usize {
    (frame.spin_pos * (LEDS as f32)).clamp(0.0, (LEDS - 1) as f32) as usize
}

pub fn construct_spiral_out<const LEDS: usize>() -> Box<dyn Mode<LEDS>> {
    Box::new(LineTracker {
        led: 11,
        func: spiral_out::<LEDS>,
    })
}

fn squarewave_flower<const LEDS: usize>(frame: &FrameState) -> usize {
    let phase = (frame.spin_pos * 3.0) % 1.0;
    if phase > 0.5 {
        LEDS
    } else {
        0
    }
}

pub fn construct_squarewave_flower<const LEDS: usize>() -> Box<dyn Mode<LEDS>> {
    Box::new(LineTracker {
        led: 11,
        func: squarewave_flower::<LEDS>,
    })
}

fn squarewave<const LEDS: usize>(frame: &FrameState) -> usize {
    let phase = (frame.spin_pos * 3.0) % 1.0;
    if phase > 0.5 {
        LEDS
    } else {
        LEDS * 2 / 3
    }
}

pub fn construct_squarewave<const LEDS: usize>() -> Box<dyn Mode<LEDS>> {
    Box::new(LineTracker {
        led: 11,
        func: squarewave::<LEDS>,
    })
}
