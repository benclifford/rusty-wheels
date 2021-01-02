use crate::leds::WheelLEDs;
use crate::structs::FrameState;
use std::io;

pub fn render_oval(
    side: usize,
    wheel_leds: &mut WheelLEDs,
    framestate: &FrameState,
) -> io::Result<()> {

    for led in 0..23 {
            wheel_leds.set(side, led, (0, 0, 0));
    }

    let clipped_spin_pos = framestate.spin_pos.max(0.0).min(1.0);

    let red_led = led_from_spinpos(clipped_spin_pos);
    wheel_leds.set(side, red_led, (255, 0, 0));

    let green_led = led_from_spinpos((clipped_spin_pos + 0.3333) % 1.0);
    wheel_leds.set(side, green_led, (0, 255, 0));

    let blue_led = led_from_spinpos((clipped_spin_pos + 0.6666) % 1.0);
    wheel_leds.set(side, blue_led, (0, 0, 255));

    Ok(())
}

fn led_from_spinpos(pos: f32) -> usize {

    let frac_radius = if pos < 0.5 {
        pos * 2.0
    } else {
        (1.0 - pos) * 2.0
    };

    let radius = 0.2 + frac_radius * 0.8;

    let led = ((radius * 23.0) as usize).min(22);

    led
}
