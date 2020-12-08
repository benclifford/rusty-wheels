use crate::leds;
use crate::structs::{Mode, FrameState};
use rand::Rng;
use std::io;

struct RandomWalkDot {
    led: usize,
}

impl Mode for RandomWalkDot {
    fn render(
        &self,
        side: usize,
        leds: &mut leds::WheelLEDs,
        frame: &FrameState,
    ) -> io::Result<()> {
 
        for led in 0..23 {
            leds.set(side, led, (0,0,0));
        }
        leds.set(side, self.led, (255,8,0));
        Ok(())
    }

    fn step(&mut self, frame: &FrameState) -> io::Result<()> {
        let choice = rand::thread_rng().gen_range(0,3);

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
