use std::cmp::Ordering;

use crate::{
    best_bin_finder::CallbackResult,
    empty_spaces::{EmptySpaces, EmptySpacesProvider},
    rect_structs::{RectWH, RectXYWH},
    state::State,
};

pub struct Input<F: Fn(RectXYWH) -> CallbackResult, G: Fn(RectXYWH) -> CallbackResult> {
    pub max_bin_side: i32,
    pub discard_step: i32,
    pub handle_successful_insertion: F,
    pub handle_unsuccessful_insertion: G,
}

impl<F: Fn(RectXYWH) -> CallbackResult, G: Fn(RectXYWH) -> CallbackResult> Input<F, G> {
    pub fn new(
        max_bin_side: i32,
        discard_step: i32,
        handle_successful_insertion: F,
        handle_unsuccessful_insertion: G,
    ) -> Self {
        Self {
            max_bin_side,
            discard_step,
            handle_successful_insertion,
            handle_unsuccessful_insertion,
        }
    }
}

pub fn find_best_packing_dont_sort<
    'a,
    EmptySpacesType: EmptySpacesProvider,
    T: Iterator<Item = &'a mut RectXYWH>,
    F: Fn(RectXYWH) -> CallbackResult,
    G: Fn(RectXYWH) -> CallbackResult,
>(
    root: &mut EmptySpaces<EmptySpacesType>,
    subjects: T,
    input: &Input<F, G>,
) -> RectWH {
    let mut state = State::new(subjects);
    state.find_best_packing_dont_sort(root, input)
}

/// Forwards to [`find_best_packing_ordered`] with the following functions:
/// - `|l, r| l.area().cmp(&r.area())`
/// - `|l, r| l.perimeter().cmp(&r.perimeter())`
/// - `|l, r| l.w.max(l.h).cmp(&r.w.max(r.h))`
/// - `|l, r| l.w.cmp(&r.w)`
/// - `|l, r| l.h.cmp(&r.h)`
pub fn find_best_packing<
    'a,
    EmptySpacesType: EmptySpacesProvider,
    T: Iterator<Item = &'a mut RectXYWH>,
    F: Fn(RectXYWH) -> CallbackResult,
    G: Fn(RectXYWH) -> CallbackResult,
>(
    root: &mut EmptySpaces<EmptySpacesType>,
    subjects: T,
    input: &Input<F, G>,
) -> RectWH {
    find_best_packing_ordered(
        root,
        subjects,
        input,
        &[
            |l, r| l.area().cmp(&r.area()),
            |l, r| l.perimeter().cmp(&r.perimeter()),
            |l, r| l.w.max(l.h).cmp(&r.w.max(r.h)),
            |l, r| l.w.cmp(&r.w),
            |l, r| l.h.cmp(&r.h),
        ],
    )
}

/// Finds the best packing for a set of rectangles.
/// Accepts any iterator that returns `&mut RectXYWH`, but its implementation
/// of [`Iterator::size_hint`] **must return a value as part of its upper bound**.
/// This is important for optimizing allocations, the function panics otherwise.
///
/// * `root` - Auxiliary storage for the algorithm.
/// * `subjects` - The rectangles to pack. Their `x` and `y` components are filled in.
/// * `input` - Settings for the algorithm.
/// * `orders` - Ordering functions to use.
pub fn find_best_packing_ordered<
    'a,
    EmptySpacesType: EmptySpacesProvider,
    T: Iterator<Item = &'a mut RectXYWH>,
    F: Fn(RectXYWH) -> CallbackResult,
    G: Fn(RectXYWH) -> CallbackResult,
>(
    root: &mut EmptySpaces<EmptySpacesType>,
    subjects: T,
    input: &Input<F, G>,
    orders: &[fn(RectXYWH, RectXYWH) -> Ordering],
) -> RectWH {
    let mut state = State::new(subjects);
    state.find_best_packing_ordered(root, input, orders)
}
