use std::mem::MaybeUninit;

use crate::rect_structs::{RectWH, RectXYWH};

pub(crate) struct CreatedSplits {
    pub count: i32,
    pub spaces: [MaybeUninit<RectXYWH>; 2],
}

impl CreatedSplits {
    pub fn failed() -> Self {
        Self {
            count: -1,
            spaces: [MaybeUninit::uninit(); _],
        }
    }

    pub fn none() -> Self {
        Self {
            count: 0,
            spaces: [MaybeUninit::uninit(); _],
        }
    }

    pub fn unified(first: RectXYWH) -> Self {
        Self {
            count: 1,
            spaces: [MaybeUninit::new(first), MaybeUninit::uninit()],
        }
    }

    pub fn split(first: RectXYWH, second: RectXYWH) -> Self {
        Self {
            count: 2,
            spaces: [MaybeUninit::new(first), MaybeUninit::new(second)],
        }
    }

    pub fn is_valid(&self) -> bool {
        self.count != -1
    }
}

pub(crate) fn insert_and_split(im: RectWH, sp: RectXYWH) -> CreatedSplits {
    let free_w = sp.w - im.w;
    let free_h = sp.h - im.h;

    if free_w < 0 || free_h < 0 {
        return CreatedSplits::failed();
    }

    if free_w == 0 && free_h == 0 {
        return CreatedSplits::none();
    }

    if free_w > 0 && free_h == 0 {
        let mut r = sp;
        r.x += im.w;
        r.w -= im.w;

        return CreatedSplits::unified(r);
    }

    if free_w == 0 && free_h > 0 {
        let mut r = sp;
        r.y += im.h;
        r.h -= im.h;

        return CreatedSplits::unified(r);
    }

    if free_w > free_h {
        let bigger_split = RectXYWH::new(sp.x + im.w, sp.y, free_w, sp.h);
        let lesser_split = RectXYWH::new(sp.x, sp.y + im.h, im.w, free_h);

        return CreatedSplits::split(bigger_split, lesser_split);
    }

    let bigger_split = RectXYWH::new(sp.x, sp.y + im.h, sp.w, free_h);
    let lesser_split = RectXYWH::new(sp.x + im.w, sp.y, free_w, im.h);

    CreatedSplits::split(bigger_split, lesser_split)
}
