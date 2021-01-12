use crate::helpers::fraction_to_rgb;
use crate::leds;
use crate::structs::{FrameState, Mode};
use rand::Rng;
use std::io;

struct RandomWalkDot {
    led: usize,
}

impl Mode for RandomWalkDot {
    fn render(
        &self,
        side: leds::Side,
        leds: &mut leds::WheelLEDs,
        _frame: &FrameState,
    ) -> io::Result<()> {
        for led in 0..23 {
            leds.set(side, led, (0, 0, 0));
        }
        leds.set(side, self.led, (255, 8, 0));
        Ok(())
    }

    fn step(&mut self, _frame: &FrameState) -> io::Result<()> {
        let choice = rand::thread_rng().gen_range(0, 3);

        if choice == 1 && self.led < 22 {
            self.led += 1;
        } else if choice == 2 && self.led > 0 {
            self.led -= 1;
        }

        Ok(())
    }
}

pub fn create_random_walk_dot() -> Box<dyn Mode> {
    Box::new(RandomWalkDot { led: 11 })
}

struct Lightning {
    led: usize,
    hue: f32
}

impl Mode for Lightning {
    fn render(
        &self,
        side: leds::Side,
        leds: &mut leds::WheelLEDs,
        _frame: &FrameState,
    ) -> io::Result<()> {
        for led in 0..23 {
            leds.set(side, led, (0, 0, 0));
        }
        leds.set(side, self.led, fraction_to_rgb(self.hue, None));
        Ok(())
    }

    fn step(&mut self, _frame: &FrameState) -> io::Result<()> {
        let choice = rand::thread_rng().gen_range(0, 3);

        if choice == 1 && self.led < 22 {
            self.led += 1;
        } else if choice == 2 && self.led > 0 {
            self.led -= 1;
        } else if choice == 1 && self.led >= 22 {
            self.led = 11;
            self.hue = rand::thread_rng().gen_range(0.0, 1.0);
        } else if choice == 2 && self.led <= 0 {
            self.led = 11;
            self.hue = rand::thread_rng().gen_range(0.0, 1.0);
        }

        Ok(())
    }
}

pub fn create_lightning() -> Box<dyn Mode> {
    Box::new(Lightning { led: 11, hue: 0.0 })
}


struct FloatSpray {
    leds: [f32; 23],
}

impl Mode for FloatSpray {
    fn render(
        &self,
        side: leds::Side,
        leds: &mut leds::WheelLEDs,
        _frame: &FrameState,
    ) -> io::Result<()> {
        for led in 0..23 {
            let colour = ((self.leds[led].powf(3.0) * 255.0) as u8, 0, 0);
            leds.set(side, led, colour);
        }
        Ok(())
    }

    fn step(&mut self, _frame: &FrameState) -> io::Result<()> {
        for led in 0..22 {
            self.leds[led] = self.leds[led + 1]
        }

        self.leds[22] = rand::thread_rng().gen_range(0.0, 1.0);

        Ok(())
    }
}

pub fn create_float_spray() -> Box<dyn Mode> {
    Box::new(FloatSpray { leds: [0.0; 23] })
}
