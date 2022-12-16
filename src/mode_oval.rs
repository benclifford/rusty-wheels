use crate::structs::{FrameState, RGB24};
use std::io;

pub fn render_oval(wheel_leds: &mut [RGB24], framestate: &FrameState) -> io::Result<()> {
    for led in 0..23 {
        wheel_leds[led] = (0, 0, 0);
    }

    let clipped_spin_pos = framestate.spin_pos.clamp(0.0, 1.0);

    let red_led = led_from_spinpos(clipped_spin_pos);
    wheel_leds[red_led] = (255, 0, 0);

    let green_led = led_from_spinpos((clipped_spin_pos + 0.3333) % 1.0);
    wheel_leds[green_led] = (0, 255, 0);

    let blue_led = led_from_spinpos((clipped_spin_pos + 0.6666) % 1.0);
    wheel_leds[blue_led] = (0, 0, 255);

    Ok(())
}

fn led_from_spinpos(pos: f32) -> usize {
    let frac_radius = if pos < 0.5 {
        pos * 2.0
    } else {
        (1.0 - pos) * 2.0
    };

    let radius = 0.2 + frac_radius * 0.8;

    ((radius * 23.0) as usize).min(22)
}
