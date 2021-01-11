use crate::helpers::{fraction_to_rgb, spinpos_to_rgb};
use crate::structs::FrameState;
use rand::Rng;
use std::cmp;
use std::f32::consts::TAU;
use std::io;

/// This renders the first side of the wheel with:
///  * an 8 pixel rainbow around the wheel
///  * a constant blue LED
///  * green and purple LEDs that tick once per frame
///    to show the size of a rotational-pixel
pub fn render_rainbows(wheel_leds: &mut [(u8, u8, u8)], framestate: &FrameState) -> io::Result<()> {
    for led in 0..8 {
        wheel_leds[led] = (0, 0, 0);
    }

    let rainbow_colour = spinpos_to_rgb(framestate);

    for led in 8..16 {
        wheel_leds[led] = rainbow_colour;
    }

    wheel_leds[16] = (0, 0, 0);

    wheel_leds[17] = (0, 0, 255);

    wheel_leds[18] = (0, 0, 0);

    let counter_phase = framestate.loop_counter % 6;
    if counter_phase == 0 {
        wheel_leds[19] = (0, 0, 0);
        wheel_leds[20] = (0, 255, 0);
        wheel_leds[21] = (0, 0, 0);
        wheel_leds[22] = (0, 64, 0);
    } else if counter_phase == 3 {
        wheel_leds[19] = (32, 0, 32);
        wheel_leds[20] = (0, 0, 0);
        wheel_leds[21] = (128, 0, 128);
        wheel_leds[22] = (0, 0, 0);
    } else {
        wheel_leds[19] = (0, 0, 0);
        wheel_leds[20] = (0, 0, 0);
        wheel_leds[21] = (0, 0, 0);
        wheel_leds[22] = (0, 0, 0);
    }

    Ok(())
}

/// This renders a 3 pixel rainbox rim, using the
/// three LEDs as separate RGB channels
pub fn render_rainbow_rgb_rim(
    wheel_leds: &mut [(u8, u8, u8)],
    framestate: &FrameState,
) -> io::Result<()> {
    for led in 0..23 {
        wheel_leds[led] = (0, 0, 0);
    }

    let (r, g, b) = spinpos_to_rgb(framestate);
    wheel_leds[20] = (r, 0, 0);
    wheel_leds[21] = (0, g, 0);
    wheel_leds[22] = (0, 0, b);

    Ok(())
}

pub fn render_rainbow_rgb_speckle_rim(
    wheel_leds: &mut [(u8, u8, u8)],
    framestate: &FrameState,
) -> io::Result<()> {
    for led in 0..23 {
        wheel_leds[led] = (0, 0, 0);
    }

    let (r, g, b) = spinpos_to_rgb(framestate);
    wheel_leds[18] = (r, g, b);

    match framestate.loop_counter % 3 {
        0 => wheel_leds[19] = (r, 0, 0),
        1 => wheel_leds[20] = (0, g, 0),
        _ => wheel_leds[21] = (0, 0, b),
    };

    wheel_leds[22] = (r, g, b);

    Ok(())
}

/// This renders a 6 pixel rainbox rim, with
/// each LED being one of the six possible
/// subsets of RGB.
pub fn render_rainbow_rgb_plus_rim(
    wheel_leds: &mut [(u8, u8, u8)],
    framestate: &FrameState,
) -> io::Result<()> {
    for led in 0..23 {
        wheel_leds[led] = (0, 0, 0);
    }

    let (r, g, b) = spinpos_to_rgb(framestate);
    wheel_leds[17] = (r, 0, b);
    wheel_leds[18] = (r, 0, 0);
    wheel_leds[19] = (r, g, 0);
    wheel_leds[20] = (0, g, 0);
    wheel_leds[21] = (0, g, b);
    wheel_leds[22] = (0, 0, b);

    Ok(())
}

/// This renders the first side of the wheel with
/// an 8 pixel rainbow around the rim of wheel
pub fn render_rainbow_rim(
    wheel_leds: &mut [(u8, u8, u8)],
    framestate: &FrameState,
) -> io::Result<()> {
    for led in 0..15 {
        wheel_leds[led] = (0, 0, 0);
    }

    let rainbow_colour = spinpos_to_rgb(framestate);

    for led in 15..23 {
        wheel_leds[led] = rainbow_colour;
    }

    Ok(())
}

pub fn render_rainbow_rim_sine(
    wheel_leds: &mut [(u8, u8, u8)],
    framestate: &FrameState,
) -> io::Result<()> {
    for led in 0..15 {
        wheel_leds[led] = (0, 0, 0);
    }

    let r = ((0.5 + 0.5 * ( 7.0 * framestate.spin_pos * 6.28).sin()).powf(2.0) * 255.0) as u8;
    let g = ((0.5 + 0.5 * ( 15.0 * framestate.spin_pos * 6.28).sin()).powf(2.0) * 255.0) as u8;
    let b = ((0.5 + 0.5 * ( 3.0 * framestate.spin_pos * 6.28).sin()).powf(2.0) * 255.0) as u8;

    wheel_leds[22] = (r, 0, 0);
    wheel_leds[21] = (0, g, 0);
    wheel_leds[20] = (0, 0, b);

    Ok(())
}

pub fn render_rainbow_rim_sine_overlay(
    wheel_leds: &mut [(u8, u8, u8)],
    framestate: &FrameState,
) -> io::Result<()> {
    for led in 0..15 {
        wheel_leds[led] = (0, 0, 0);
    }

    let r = ((0.5 + 0.5 * ( 7.0 * framestate.spin_pos * 6.28).sin()).powf(2.0) * 255.0) as u8;
    let g = ((0.5 + 0.5 * ( 15.0 * framestate.spin_pos * 6.28).sin()).powf(2.0) * 255.0) as u8;
    let b = ((0.5 + 0.5 * ( 3.0 * framestate.spin_pos * 6.28).sin()).powf(2.0) * 255.0) as u8;

    wheel_leds[22] = (r, g, 0);
    wheel_leds[21] = (0, g, b);
    wheel_leds[20] = (r, 0, b);
    wheel_leds[19] = (r, g, 0);
    wheel_leds[18] = (0, g, b);
    wheel_leds[17] = (r, 0, b);
    wheel_leds[16] = (r, g, b);

    Ok(())
}

pub fn render_rainbow_rim_spaced(
    wheel_leds: &mut [(u8, u8, u8)],
    framestate: &FrameState,
) -> io::Result<()> {
    for led in 0..15 {
        wheel_leds[led] = (0, 0, 0);
    }

    let r = ((0.5 + 0.5 * ( 7.0 * framestate.spin_pos * 6.28).sin()).powf(2.0) * 255.0) as u8;
    let g = ((0.5 + 0.5 * ( 15.0 * framestate.spin_pos * 6.28).sin()).powf(2.0) * 255.0) as u8;
    let b = ((0.5 + 0.5 * ( 3.0 * framestate.spin_pos * 6.28).sin()).powf(2.0) * 255.0) as u8;

    wheel_leds[22] = (r, 0, 0);
    wheel_leds[20] = (r, 0, 0);
    wheel_leds[17] = (0, g, 0);
    wheel_leds[12] = (0, g, 0);
    wheel_leds[4] = (0, 0, b);
    wheel_leds[0] = (0, 0, b);

    Ok(())
}

pub fn render_rainbow_rim_spaced2(
    wheel_leds: &mut [(u8, u8, u8)],
    framestate: &FrameState,
) -> io::Result<()> {
    for led in 0..15 {
        wheel_leds[led] = (0, 0, 0);
    }

    let r = ((0.5 + 0.5 * ( 7.0 * framestate.spin_pos * 6.28).sin()).powf(2.0) * 255.0) as u8;
    let r2 = ((0.5 + 0.5 * ( 8.0 * framestate.spin_pos * 6.28).sin()).powf(2.0) * 255.0) as u8;
    let g = ((0.5 + 0.5 * ( 15.0 * framestate.spin_pos * 6.28).sin()).powf(2.0) * 255.0) as u8;
    let g2 = ((0.5 + 0.5 * ( 16.0 * framestate.spin_pos * 6.28).sin()).powf(2.0) * 255.0) as u8;
    let b = ((0.5 + 0.5 * ( 3.0 * framestate.spin_pos * 6.28).sin()).powf(2.0) * 255.0) as u8;
    let b2 = ((0.5 + 0.5 * ( 4.0 * framestate.spin_pos * 6.28).sin()).powf(2.0) * 255.0) as u8;

    wheel_leds[22] = (r, 0, 0);
    wheel_leds[21] = (r2, 0, 0);
    wheel_leds[18] = (0, g, 0);
    wheel_leds[17] = (0, g2, 0);
    wheel_leds[14] = (0, 0, b);
    wheel_leds[13] = (0, 0, b2);

    Ok(())
}

pub fn render_random_rim(
    wheel_leds: &mut [(u8, u8, u8)],
    _framestate: &FrameState,
) -> io::Result<()> {
    for led in 0..20 {
        wheel_leds[led] = (0, 0, 0);
    }

    // starting at 1 avoids having all three bits off
    // (the 0 position) so there will always at least
    // be one LED on in each frame
    let n = rand::thread_rng().gen_range(1, 8);

    for led in 0..3 {
        if n & (1 << led) != 0 {
            wheel_leds[20 + led] = (255, 0, 0);
        } else {
            wheel_leds[20 + led] = (0, 0, 0);
        }
    }

    Ok(())
}

pub fn render_random_rim_red_yellow(
    wheel_leds: &mut [(u8, u8, u8)],
    _framestate: &FrameState,
) -> io::Result<()> {
    for led in 0..20 {
        wheel_leds[led] = (0, 0, 0);
    }

    // starting at 1 avoids having all three bits off
    // (the 0 position) so there will always at least
    // be one LED on in each frame
    let n = rand::thread_rng().gen_range(1, 8);

    for led in 0..3 {
        if n & (1 << led) != 0 {
            let yellow_amount = (2.0_f32).powf(rand::thread_rng().gen_range(0.0, 7.5)) as u8;
            wheel_leds[20 + led] = (255, yellow_amount, 0);
        } else {
            wheel_leds[20 + led] = (0, 0, 0);
        }
    }

    Ok(())
}

pub fn render_spin_rim(
    wheel_leds: &mut [(u8, u8, u8)],
    framestate: &FrameState,
) -> io::Result<()> {
    for led in 0..23 {
        wheel_leds[led] = (0, 0, 0);
    }

    // one rotation every 4 seconds
    // which, with 3-way symmetry, means
    // symmetry every just over 1 second.
    let t = (framestate.now.as_millis() as f32) / 1000.0 / 4.0;

    let k = (framestate.spin_pos * 3.0 + t) % 1.0;

    if k < 0.33 {

        wheel_leds[22] = (255, 64, 0);
        wheel_leds[21] = (255, 128, 0);
        wheel_leds[20] = (255, 64, 0);
    }

    Ok(())
}


pub fn render_pulsed_rainbow(
    wheel_leds: &mut [(u8, u8, u8)],
    framestate: &FrameState,
) -> io::Result<()> {
    for led in 0..15 {
        wheel_leds[led] = (0, 0, 0);
    }

    for led in 15..23 {
        let led_n = led - 15;
        let frac: f32 = ((led - 15) as f32) / 8.0;
        let v1 = (framestate.spin_pos + frac) % 1.0;
        let v2 = (v1 * (led_n as f32 + 2.0)) % 1.0;
        let v3 = if v2 > 0.5 { 1.0 } else { 0.0 };
        let rainbow_colour = fraction_to_rgb(frac, Some(v3));
        wheel_leds[led] = rainbow_colour;
    }

    Ok(())
}

/// This renders the second side of the wheel two overlaid patterns:
///  * a green time-based line
///  * a magenta spin position line
pub fn render_sliders(wheel_leds: &mut [(u8, u8, u8)], framestate: &FrameState) -> io::Result<()> {
    let now_millis = framestate.now.as_millis();

    // this should range from 0..23 over the period of 1 second, which is
    // around the right time for one wheel spin
    let back_led: usize = ((now_millis % 1000) * 23 / 1000) as usize;

    let spin_back_led: usize = (framestate.spin_pos * 23.0) as usize;

    for l in 0..23 {
        let g;
        if l == back_led {
            g = 255
        } else {
            g = 0
        }

        let r;
        if l == spin_back_led {
            r = 255
        } else if l == (spin_back_led + 8) % 23 {
            r = 255
        } else if l == (spin_back_led + 16) % 23 {
            r = 255
        } else {
            r = 0
        }

        wheel_leds[l] = (r, g, r);
    }
    Ok(())
}

/// This renders three slices with black between them, each slice being one
/// of red, green or blue
pub fn render_rgb_trio(wheel_leds: &mut [(u8, u8, u8)], framestate: &FrameState) -> io::Result<()> {
    for led in 0..23 {
        // led 0 should be dimmest
        // led 22 the brightest
        // this will exponentially scale up to 128 max
        let brightness = 1 << (led / 3);
        let colour: (u8, u8, u8);

        if framestate.spin_pos < 0.16 {
            colour = (brightness, 0, 0);
        } else if framestate.spin_pos < 0.32 {
            colour = (0, 0, 0);
        } else if framestate.spin_pos < 0.48 {
            colour = (0, brightness, 0);
        } else if framestate.spin_pos < 0.64 {
            colour = (0, 0, 0);
        } else if framestate.spin_pos < 0.80 {
            colour = (0, 0, brightness);
        } else {
            colour = (0, 0, 0);
        }

        wheel_leds[led] = colour;
    }

    Ok(())
}

// renders the middle pixels on each side bright red, with the
// edges (outer and hubwards) fading down to black
pub fn render_centre_red(
    wheel_leds: &mut [(u8, u8, u8)],
    _framestate: &FrameState,
) -> io::Result<()> {
    // establish a blank canvas
    for led in 0..23 {
        wheel_leds[led] = (0, 0, 0);
    }

    for n in 0..8 {
        let colour = (1 << (7 - n), 0, 0);
        wheel_leds[12 + n] = colour;
        wheel_leds[11 - n] = colour;
    }

    Ok(())
}

// renders the middle pixels on each side bright red, with the
// edges (outer and hubwards) fading down to black
pub fn render_fib_concentric(
    wheel_leds: &mut [(u8, u8, u8)],
    _framestate: &FrameState,
) -> io::Result<()> {
    // establish a blank canvas
    for led in 0..23 {
        wheel_leds[led] = (0, 0, 0);
    }

    let amber = (128, 16, 0);
    wheel_leds[23 - 1] = amber;
    wheel_leds[23 - 2] = amber;
    wheel_leds[23 - 3] = amber;
    wheel_leds[23 - 5] = amber;
    wheel_leds[23 - 8] = amber;
    wheel_leds[23 - 13] = amber;
    wheel_leds[23 - 21] = amber;

    Ok(())
}

pub fn render_sine(wheel_leds: &mut [(u8, u8, u8)], framestate: &FrameState) -> io::Result<()> {
    // establish a blank canvas
    for led in 0..23 {
        wheel_leds[led] = (0, 0, 0);
    }

    let phase = (framestate.spin_pos * TAU * 10.0).sin();

    // beware of casting to unsigned when there could still be
    // negatives around
    let led = (17.0 + phase * 5.0) as usize;

    wheel_leds[led] = (0, 255, 0);

    Ok(())
}

pub fn render_helix(wheel_leds: &mut [(u8, u8, u8)], framestate: &FrameState) -> io::Result<()> {
    // establish a blank canvas
    for led in 0..23 {
        wheel_leds[led] = (0, 0, 0);
    }

    let phase = (framestate.spin_pos * TAU * 10.0).sin();

    // beware of casting to unsigned when there could still be
    // negatives around
    let led = cmp::min((17.0 + phase * 6.0) as usize, 22);
    wheel_leds[led] = (64, 0, 64);

    let led = cmp::min((17.0 - phase * 6.0) as usize, 22);
    wheel_leds[led] = (0, 255, 0);

    Ok(())
}

pub fn render_sine_full(
    wheel_leds: &mut [(u8, u8, u8)],
    framestate: &FrameState,
) -> io::Result<()> {
    // establish a blank canvas
    for led in 0..23 {
        wheel_leds[led] = (0, 0, 0);
    }

    let phase = (framestate.spin_pos * TAU * 10.0).sin();

    // beware of casting to unsigned when there could still be
    // negatives around
    let led = (17.0 + phase * 5.0) as usize;

    wheel_leds[led] = (0, 255, 0);

    let phase2 = (framestate.spin_pos * TAU * 7.0).sin();
    let led2 = (8.0 + phase2 * 3.0) as usize;
    wheel_leds[led2] = (255, 0, 0);

    let phase3 = (framestate.spin_pos * TAU * 3.0).sin();
    let led3 = (3.0 + phase3 * 2.0) as usize;
    wheel_leds[led3] = (0, 0, 255);

    Ok(())
}

pub fn render_graycode_rim(
    wheel_leds: &mut [(u8, u8, u8)],
    framestate: &FrameState,
) -> io::Result<()> {
    // establish a blank canvas
    for led in 0..23 {
        wheel_leds[led] = (0, 0, 0);
    }

    let segment = (framestate.spin_pos * 8.0) as u8; // could go over 8 because spinpos can go over 1

    let gray = segment ^ (segment >> 1);

    let amber = (255, 32, 0);

    if (gray & 0b001) != 0 {
        wheel_leds[22] = amber;
        wheel_leds[21] = amber;
        wheel_leds[20] = amber;
    }

    if (gray & 0b010) != 0 {
        wheel_leds[19] = amber;
        wheel_leds[18] = amber;
        wheel_leds[17] = amber;
    }

    if (gray & 0b100) != 0 {
        wheel_leds[16] = amber;
        wheel_leds[15] = amber;
        wheel_leds[14] = amber;
    }

    Ok(())
}

pub fn render_radial_stripes(
    wheel_leds: &mut [(u8, u8, u8)],
    framestate: &FrameState,
) -> io::Result<()> {
    // establish a blank canvas
    for led in 0..23 {
        wheel_leds[led] = (0, 0, 0);
    }

    let segment = (framestate.spin_pos * 32.0) as u8; // could go over 32 because spinpos can go over 1

    if segment % 2 == 1 {
        for led in 12..23 {
            wheel_leds[led] = (64, 64, 64);
        }
    }

    Ok(())
}

pub fn render_europa(wheel_leds: &mut [(u8, u8, u8)], framestate: &FrameState) -> io::Result<()> {
    // establish a blue canvas
    for led in 0..23 {
        wheel_leds[led] = (0, 0, 32);
    }

    let segment = (framestate.spin_pos * 12.0) % 1.0; // could go over 12 because spinpos can go over 1

    if segment < 0.08 || (segment >= 0.16 && segment < 0.24) {
        wheel_leds[18] = (255, 255, 0);
    } else if segment < 0.16 {
        for led in 17..20 {
            wheel_leds[led] = (255, 255, 0);
        }
    }

    Ok(())
}

pub fn render_fade_spirals(
    wheel_leds: &mut [(u8, u8, u8)],
    framestate: &FrameState,
) -> io::Result<()> {
    // establish a blank canvas

    let s1 = cmp::min(22, (23.0 * framestate.spin_pos) as i8);
    let s2 = cmp::min(22, (23.0 * ((framestate.spin_pos + 0.5) % 1.0)) as i8);

    for led in 0..23 {
        let dist_s1 = (s1 - led).abs() as u8;
        let dist_s2 = (s2 - led).abs() as u8;
        if dist_s1 < dist_s2 {
            if dist_s1 > 7 {
                wheel_leds[led as usize] = (0, 0, 0);
            } else {
                let v = (2 as u8).pow((7 - dist_s1) as u32);
                wheel_leds[led as usize] = (0, v, 0);
            }
        } else {
            if dist_s2 > 7 {
                wheel_leds[led as usize] = (0, 0, 0);
            } else {
                let v = (2 as u8).pow((7 - dist_s2) as u32);
                wheel_leds[led as usize] = (v, 0, v);
            }
        }
    }

    Ok(())
}

pub fn render_fade_quarters(
    wheel_leds: &mut [(u8, u8, u8)],
    framestate: &FrameState,
) -> io::Result<()> {
    let fade_frac = (framestate.spin_pos * 4.0) % 1.0;

    // some gamma correction
    let brightness = fade_frac.powf(3.0).min(1.0);

    let pix_brightness_red = (255.0 * brightness) as u8;
    let pix_brightness_green = (64.0 * brightness) as u8;

    for led in 0..11 {
        wheel_leds[led as usize] = (0, 0, 0);
    }
    for led in 11..23 {
        wheel_leds[led as usize] = (pix_brightness_red, pix_brightness_green, 0);
    }

    Ok(())
}
