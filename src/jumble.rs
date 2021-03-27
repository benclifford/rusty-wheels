use rand::seq::SliceRandom;
use rand::Rng;

use crate::structs::Mode;

/// I feel like this should be generic over the content type
/// but I couldn't get the generics working right with requiring
/// the content be Copy (so I can both return it and keep it in
/// the list.
///
/// TODO: this jumble method doesn't give the second half of the
/// initial list a fair chance to be picked. Maybe the content
/// list should be shuffled at creation time?

pub struct Jumbler<const LEDS: usize> {
    content: Vec<fn() -> Box<dyn Mode<LEDS>>>,
}

impl<const LEDS: usize> Jumbler<LEDS> {
    pub fn new(mut content: Vec<fn() -> Box<dyn Mode<LEDS>>>) -> Jumbler<LEDS> {
        content.shuffle(&mut rand::thread_rng());
        Jumbler { content: content }
    }
}

impl<const LEDS: usize> Iterator for Jumbler<LEDS> {
    type Item = fn() -> Box<dyn Mode<LEDS>>;

    fn next(&mut self) -> Option<fn() -> Box<dyn Mode<LEDS>>> {
        let next_index = rand::thread_rng().gen_range(0, self.content.len() / 2);

        let entry = self.content.remove(next_index);

        self.content.push(entry);

        Some(entry)
    }
}
