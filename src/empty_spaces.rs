use crate::{
    insert_and_split::{CreatedSplits, insert_and_split},
    rect_structs::{RectWH, RectXYWH},
};

pub trait EmptySpacesProviderTrait: Default {
    fn reset(&mut self);
    fn get(&self, i: usize) -> RectXYWH;
    fn get_count(&self) -> usize;
    fn remove(&mut self, i: usize);
    fn add(&mut self, rect: RectXYWH) -> bool;
}

#[derive(Default)]
pub struct EmptySpaces<EmptySpacesProvider: EmptySpacesProviderTrait> {
    current_aabb: RectWH,
    spaces: EmptySpacesProvider,
}

impl<EmptySpacesProvider: EmptySpacesProviderTrait> EmptySpaces<EmptySpacesProvider> {
    pub fn new(r: RectWH) -> Self {
        let mut ret = Self {
            current_aabb: RectWH::default(),
            spaces: Default::default(),
        };

        ret.reset(r);

        ret
    }

    pub fn insert(&mut self, image_rectangle: RectWH) -> Option<RectXYWH> {
        for i in (0..self.spaces.get_count()).rev() {
            let candidate_space = self.spaces.get(i);
            let normal = Self::try_to_insert(image_rectangle, candidate_space);

            if normal.is_valid() {
                // println!("insert_fn: returning self.accept_result @ iteration {i}");
                return self.accept_result(i, image_rectangle, candidate_space, &normal);
            }
        }

        None
    }

    fn accept_result(
        &mut self,
        i: usize,
        image_rectangle: RectWH,
        candidate_space: RectXYWH,
        splits: &CreatedSplits,
    ) -> Option<RectXYWH> {
        self.spaces.remove(i);

        for s in 0..splits.count {
            if !self
                .spaces
                .add(unsafe { splits.spaces[s as usize].assume_init() })
            {
                return None;
            }
        }

        let result = RectXYWH::new(
            candidate_space.x,
            candidate_space.y,
            image_rectangle.w,
            image_rectangle.h,
        );

        self.current_aabb.expand_with(result);

        Some(result)
    }

    pub fn reset(&mut self, r: RectWH) {
        self.current_aabb = Default::default();

        self.spaces.reset();
        self.spaces.add(RectXYWH::new(0, 0, r.w, r.h));
    }

    fn try_to_insert(img: RectWH, candidate_space: RectXYWH) -> CreatedSplits {
        insert_and_split(img, candidate_space)
    }

    pub fn get_rects_aabb(&self) -> RectWH {
        self.current_aabb
    }

    pub fn get_spaces(&self) -> &EmptySpacesProvider {
        &self.spaces
    }
}
