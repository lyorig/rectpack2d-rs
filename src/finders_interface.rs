use std::{cmp::Ordering, mem::MaybeUninit};

use crate::{
    best_bin_finder::{CallbackResult, find_best_packing_impl},
    empty_spaces::{EmptySpaces, EmptySpacesProviderTrait},
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

pub fn find_best_packing<
    EmptySpacesType: EmptySpacesProviderTrait,
    T,
    F: Fn(RectXYWH) -> CallbackResult,
    G: Fn(RectXYWH) -> CallbackResult,
>(
    root: &mut EmptySpaces<EmptySpacesType>,
    subjects: &mut [T],
    input: &Input<F, G>,
) -> RectWH
where
    for<'a> &'a mut T: Into<&'a mut RectXYWH>,
    for<'a> &'a T: Into<&'a RectXYWH>,
{
    find_best_packing_impl(root, create_box(subjects), subjects.len(), input)
}

/// Creates a `Box` containing all orders, contiguously.
/// This should be a performance improvement over having
/// several `Vec`s separately.
fn create_box<T>(subjects: &mut [T]) -> Box<[*mut RectXYWH]>
where
    for<'a> &'a mut T: Into<&'a mut RectXYWH>,
    for<'a> &'a T: Into<&'a RectXYWH>,
{
    // Five orders in total.
    let l = subjects.len();
    let mut orders = Box::<[*mut RectXYWH]>::new_uninit_slice(l * 5);

    for (i, s) in subjects.iter_mut().enumerate() {
        let r: &mut RectXYWH = s.into();

        if r.area() > 0 {
            orders[i].write(r as *mut RectXYWH);
        }
    }

    let (src, tgt) = orders.split_at_mut(l);
    for chunk in tgt.chunks_exact_mut(l) {
        chunk.copy_from_slice(src);
    }

    // Shorthand function to
    let f = |i: usize| (l * i)..(l * (i + 1));

    orders[f(0)].sort_by(s(|l, r| l.area().cmp(&r.area())));
    orders[f(1)].sort_by(s(|l, r| l.perimeter().cmp(&r.perimeter())));
    orders[f(2)].sort_by(s(|l, r| l.w.max(l.h).cmp(&r.w.max(r.h))));
    orders[f(3)].sort_by(s(|l, r| l.w.cmp(&r.w)));
    orders[f(4)].sort_by(s(|l, r| l.h.cmp(&r.h)));

    unsafe { orders.assume_init() }
}

/// Sorter shorthand function.
fn s<F: Fn(RectXYWH, RectXYWH) -> Ordering>(
    func: F,
) -> impl Fn(&MaybeUninit<*mut RectXYWH>, &MaybeUninit<*mut RectXYWH>) -> Ordering {
    move |lhs, rhs| func(unsafe { *lhs.assume_init() }, unsafe { *rhs.assume_init() }).reverse()
}
