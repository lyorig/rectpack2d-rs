pub type TotalAreaType = i32;

#[derive(Clone, Copy, Default, Debug)]
pub struct RectWH {
    pub w: i32,
    pub h: i32,
}

impl RectWH {
    pub fn new(w: i32, h: i32) -> Self {
        Self { w, h }
    }

    pub fn from_xywh(xywh: RectXYWH) -> Self {
        Self::new(xywh.w, xywh.h)
    }

    pub fn max_side(&self) -> i32 {
        self.w.max(self.h)
    }

    pub fn min_side(&self) -> i32 {
        self.w.min(self.h)
    }

    pub fn area(&self) -> i32 {
        self.w * self.h
    }

    pub fn perimeter(&self) -> i32 {
        2 * self.w + 2 * self.h
    }

    pub fn expand_with(&mut self, r: RectXYWH) {
        self.w = self.w.max(r.x + r.w);
        self.h = self.h.max(r.y + r.h);
    }
}

#[derive(Clone, Copy, Default, Debug)]
pub struct RectXYWH {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

impl RectXYWH {
    pub fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        Self { x, y, w, h }
    }

    pub fn from_wh(wh: RectWH) -> Self {
        Self::new(0, 0, wh.w, wh.h)
    }

    pub fn area(&self) -> i32 {
        self.w * self.h
    }

    pub fn perimeter(&self) -> i32 {
        2 * self.w + 2 * self.h
    }
}

impl From<&RectXYWH> for RectWH {
    fn from(value: &RectXYWH) -> Self {
        Self::new(value.w, value.h)
    }
}

impl From<&mut RectXYWH> for RectWH {
    fn from(value: &mut RectXYWH) -> Self {
        Self::new(value.w, value.h)
    }
}
