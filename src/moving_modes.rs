use crate::stateless_mode;
use crate::stateless_mode_b;
use crate::structs::Mode;

use crate::mode_bitmap_text;
use crate::mode_cellblobs;
use crate::mode_cellular;
use crate::mode_dither;
use crate::mode_edge_strobe;
use crate::mode_flames;
use crate::mode_linetracker;
use crate::mode_misc;
use crate::mode_oval;
use crate::mode_rainbow;
use crate::mode_randomwalk;
use crate::mode_rgb_dither;
use crate::mode_speckles;
use crate::mode_stepper;
use crate::mode_trails;

pub fn modes<const LEDS: usize>() -> &'static [fn() -> Box<dyn Mode<LEDS>>] {
    &[
        mode_randomwalk::create_fork_lightning,
        mode_randomwalk::create_lightning,
        stateless_mode_b!(mode_misc::render_spin_rim::<LEDS>),
        stateless_mode!(mode_misc::render_rainbow_rim_spaced2),
        stateless_mode!(mode_misc::render_rainbow_rim_spaced),
        stateless_mode!(mode_misc::render_rainbow_rim_sine_overlay),
        stateless_mode!(mode_misc::render_rainbow_rim_sine),
        mode_linetracker::construct_squarewave,
        mode_linetracker::construct_squarewave_flower,
        mode_linetracker::construct_spiral_out,
        mode_stepper::construct_stepper,
        stateless_mode!(mode_misc::render_rainbow_rgb_speckle_rim),
        stateless_mode!(mode_misc::render_rainbow_rgb_plus_rim),
        stateless_mode!(mode_misc::render_rainbow_rgb_rim),
        stateless_mode!(mode_oval::render_oval),
        mode_bitmap_text::construct_phrase_mode_hello,
        mode_rgb_dither::create_dither,
        mode_dither::create_dither,
        mode_trails::construct_hue_trails_sparse,
        mode_trails::construct_hue_trails,
        mode_trails::construct_white_trails,
        mode_randomwalk::create_float_spray,
        mode_randomwalk::create_random_walk_dot,
        // discrete-like modes
        mode_cellular::construct_cellular,
        stateless_mode!(mode_misc::render_graycode_rim),
        stateless_mode!(mode_misc::render_random_rim),
        stateless_mode!(mode_misc::render_random_rim_red_yellow),
        mode_cellblobs::create_cellblobs,
        // pulsing modes
        mode_edge_strobe::construct_edge_strobe,
        stateless_mode!(mode_misc::render_fade_quarters),
        stateless_mode!(mode_misc::render_radial_stripes),
        stateless_mode!(mode_misc::render_rgb_trio),
        // speckle modes
        stateless_mode!(mode_speckles::render_mod_speckle),
        stateless_mode!(mode_speckles::render_speckle_onepix),
        stateless_mode!(mode_speckles::render_speckle_random),
        stateless_mode!(mode_speckles::render_rainbow_speckle),
        // text modes
        mode_bitmap_text::construct_phrase_fuck_boris,
        mode_bitmap_text::construct_phrase_mode,
        mode_bitmap_text::construct_speedo_mode,
        // solid image-like modes
        stateless_mode!(mode_misc::render_centre_red),
        // stateless_mode!(mode_misc::render_europa),
        // rainbows and squiggles
        stateless_mode!(mode_misc::render_helix),
        stateless_mode!(mode_misc::render_pulsed_rainbow),
        stateless_mode!(mode_misc::render_rainbow_rim),
        stateless_mode!(mode_misc::render_fade_spirals),
        stateless_mode!(mode_misc::render_sine_full),
        stateless_mode!(mode_misc::render_sine),
        stateless_mode!(mode_misc::render_rainbows),
        stateless_mode!(mode_misc::render_sliders),
        mode_rainbow::construct_rainbow_on_off,
        stateless_mode!(mode_misc::render_fib_concentric),
        stateless_mode!(mode_flames::render_hub_white),
        mode_flames::create_hub_rainbow,
    ]
}
