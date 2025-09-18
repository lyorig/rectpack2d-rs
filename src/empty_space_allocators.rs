use crate::{empty_spaces::EmptySpacesProviderTrait, rect_structs::RectXYWH};

#[derive(Default)]
pub struct DefaultEmptySpaces {
    empty_spaces: Vec<RectXYWH>,
}

impl EmptySpacesProviderTrait for DefaultEmptySpaces {
    fn reset(&mut self) {
        self.empty_spaces.clear();
    }

    fn get(&self, i: usize) -> RectXYWH {
        self.empty_spaces[i]
    }

    fn get_count(&self) -> usize {
        self.empty_spaces.len()
    }

    fn remove(&mut self, i: usize) {
        self.empty_spaces[i] = *self.empty_spaces.last().unwrap();
        self.empty_spaces.pop();
    }

    fn add(&mut self, rect: RectXYWH) -> bool {
        self.empty_spaces.push(rect);
        true
    }
}

pub struct StaticEmptySpaces<const MAX_SPACES: usize> {
    count_spaces: usize,
    empty_spaces: [RectXYWH; MAX_SPACES],
}

impl<const MAX_SPACES: usize> Default for StaticEmptySpaces<MAX_SPACES> {
    fn default() -> Self {
        Self {
            count_spaces: 0,
            empty_spaces: [RectXYWH::default(); MAX_SPACES],
        }
    }
}

impl<const MAX_SPACES: usize> EmptySpacesProviderTrait for StaticEmptySpaces<MAX_SPACES> {
    fn reset(&mut self) {
        self.count_spaces = 0
    }

    fn get(&self, i: usize) -> RectXYWH {
        self.empty_spaces[i]
    }

    fn get_count(&self) -> usize {
        self.count_spaces
    }

    fn remove(&mut self, i: usize) {
        self.empty_spaces[i] = self.empty_spaces[self.count_spaces - 1];
        self.count_spaces -= 1;
    }

    fn add(&mut self, r: RectXYWH) -> bool {
        if self.count_spaces >= self.empty_spaces.len() {
            return false;
        }

        self.empty_spaces[self.count_spaces] = r;
        self.count_spaces += 1;

        true
    }
}
