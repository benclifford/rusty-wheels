use rand::seq::SliceRandom;
use rand::Rng;

pub struct Jumbler<T> {
    content: Vec<T>,
}

impl<T> Jumbler<T> {
    pub fn new(mut content: Vec<T>) -> Jumbler<T> {
        content.shuffle(&mut rand::thread_rng());
        Jumbler { content: content }
    }
}

impl<T: Copy> Iterator for Jumbler<T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        let next_index = rand::thread_rng().gen_range(0, self.content.len() / 2);

        let entry = self.content.remove(next_index);

        self.content.push(entry);

        Some(entry)
    }
}
