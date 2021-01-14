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

pub struct Jumbler {
    content: Vec<fn() -> Box<dyn Mode>>
}

impl Jumbler {
    pub fn new(content: Vec<fn() -> Box<dyn Mode>>) -> Jumbler {
        Jumbler { content: content }
    }
}

impl Iterator for Jumbler {
    type Item = fn() -> Box<dyn Mode>;

    fn next(&mut self) -> Option<fn() -> Box<dyn Mode>> {

        let next_index = rand::thread_rng().gen_range(0, self.content.len()/2);

        let entry = self.content.remove(next_index);

        self.content.push(entry);

        Some(entry)
    }
}
