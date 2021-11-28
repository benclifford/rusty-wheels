use crate::leds;
use crate::leds::Side;
use std::io;
use std::time::Duration;

/// A FrameState contains information about the position and timing of
/// the bike wheel useful for rendering a frame.
pub struct FrameState {
    /// Duration since the executable started
    pub now: Duration,

    /// A count of the number of frames rendered. This will increase by one
    /// on each render, regardless of time or wheel rotation.
    pub loop_counter: u32,

    /// An estimate of the current position of the wheel, ranging from 0 to
    /// approximately 1. This might go above 1 if the bike is slowing down,
    /// so code needs to accept that.
    pub spin_pos: f32,

    /// An estimate of how long the wheel takes to rotate once, based on the
    /// last full rotation.
    pub spin_length: Duration,
}

/// render will be called to render each side
/// then step will be called to allow any state advancing to happen
pub trait Mode<const LEDS: usize> {
    fn render(
        &self,
        side: Side,
        leds: &mut leds::WheelLEDs<LEDS>,
        frame: &FrameState,
    ) -> io::Result<()>;

    /// runs before render calls on a frame
    fn pre_step(&mut self, _frame: &FrameState) -> io::Result<()> {
        Ok(())
    }

    /// runs after render calls on a frame
    fn step(&mut self, _frame: &FrameState) -> io::Result<()> {
        Ok(())
    }
}

pub struct StatelessMode<const LEDS: usize> {
    pub render_fn: fn(leds: &mut [(u8, u8, u8)], frame: &FrameState) -> io::Result<()>,
}

impl<const LEDS: usize> Mode<LEDS> for StatelessMode<LEDS> {
    fn render(
        &self,
        side: Side,
        leds: &mut leds::WheelLEDs<{ LEDS }>,
        frame: &FrameState,
    ) -> io::Result<()> {
        (self.render_fn)(leds.side_slice(side), frame)
    }
}

/// Lifts a render function into a mode which has no state
#[macro_export]
macro_rules! stateless_mode {
    ( $x:expr ) => {
        || Box::new(crate::structs::StatelessMode { render_fn: $x })
    };
}

pub struct StatelessModeB<const LEDS: usize> {
    pub render_fn:
        fn(side: Side, leds: &mut [(u8, u8, u8); LEDS], frame: &FrameState) -> io::Result<()>,
}

impl<const LEDS: usize> Mode<LEDS> for StatelessModeB<LEDS> {
    fn render(
        &self,
        side: Side,
        leds: &mut leds::WheelLEDs<{ LEDS }>,
        frame: &FrameState,
    ) -> io::Result<()> {
        (self.render_fn)(side, leds.side_slice_b(side), frame)
    }
}

/// Lifts a render function into a mode which has no state
/// but will have sides and maybe some more nuanced leds structure
#[macro_export]
macro_rules! stateless_mode_b {
    ( $x:expr ) => {
        || Box::new(crate::structs::StatelessModeB { render_fn: $x })
    };
}
