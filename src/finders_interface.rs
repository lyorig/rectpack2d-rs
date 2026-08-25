use std::cmp::Ordering;

use crate::{
    best_bin_finder::{CallbackResult, Finder, Solver},
    empty_spaces::{EmptySpaces, EmptySpacesProvider},
    rect_structs::{RectWH, RectXYWH},
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
    let sortable = subjects
        .filter_map(|f| if f.area() > 0 { Some(f) } else { None })
        .collect::<Box<[&mut RectXYWH]>>();

    let max_bin = RectWH::new(input.max_bin_side, input.max_bin_side);
    let mut finder = Finder::new(max_bin);

    _ = finder.evaluate_order(
        root,
        // SAFETY: `&[&mut T]` and `&[&T]` have the same representation.
        unsafe { std::mem::transmute::<&[&mut RectXYWH], &[&RectXYWH]>(sortable.as_ref()) },
        max_bin,
        input.discard_step,
    );

    assert!(finder.best_order.is_some());

    root.reset(finder.best_bin);

    for rect in sortable {
        match root.insert(rect.into()) {
            Some(ret) => {
                *rect = ret;
                if let CallbackResult::AbortPacking = (input.handle_successful_insertion)(*rect) {
                    break;
                }
            }
            None => {
                if let CallbackResult::AbortPacking = (input.handle_unsuccessful_insertion)(*rect) {
                    break;
                }
            }
        }
    }

    root.rects_aabb()
}

/// Forwards to `find_best_packing_ordered` with the following functions:
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
    orders: &[fn(&RectXYWH, &RectXYWH) -> Ordering],
) -> RectWH {
    let mut solver = Solver::new(subjects);
    solver.find_best_packing_ordered(root, input, orders)
}
