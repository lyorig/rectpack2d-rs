use crate::rect_structs::RectWH;

pub enum CallbackResult {
    AbortPacking,
    ContinuePacking,
}

pub(crate) enum BinDimension {
    Both,
    Width,
    Height,
}

pub(crate) enum BestPackingReturn {
    TotalArea,
    Rect(RectWH),
}
