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

pub fn find_best_packing_dont_sort<
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
{
    find_best_packing_impl(
        root,
        subjects.iter_mut().map(|i| i.into() as _).collect(),
        subjects.len(),
        input,
    )
}

/// Forwards to `find_best_packing_ordered` with the following functions:
/// - `|l, r| l.area().cmp(&r.area())`
/// - `|l, r| l.perimeter().cmp(&r.perimeter())`
/// - `|l, r| l.w.max(l.h).cmp(&r.w.max(r.h))`
/// - `|l, r| l.w.cmp(&r.w)`
/// - `|l, r| l.h.cmp(&r.h)`
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
{
    find_best_packing_ordered(
        root,
        subjects,
        input,
        [
            |l, r| l.area().cmp(&r.area()),
            |l, r| l.perimeter().cmp(&r.perimeter()),
            |l, r| l.w.max(l.h).cmp(&r.w.max(r.h)),
            |l, r| l.w.cmp(&r.w),
            |l, r| l.h.cmp(&r.h),
        ],
    )
}

/// Finds the best packing for a set of rectangles.
///
/// * `root` - Auxiliary storage for the algorithm.
/// * `subjects` - The rectangles to pack. Their `x` and `y` components are filled in.
/// * `input` - Settings for the algorithm.
/// * `orders` - Ordering functions to use.
pub fn find_best_packing_ordered<
    EmptySpacesType: EmptySpacesProviderTrait,
    T,
    F: Fn(RectXYWH) -> CallbackResult,
    G: Fn(RectXYWH) -> CallbackResult,
    const N: usize,
>(
    root: &mut EmptySpaces<EmptySpacesType>,
    subjects: &mut [T],
    input: &Input<F, G>,
    orders: [fn(RectXYWH, RectXYWH) -> Ordering; N],
) -> RectWH
where
    for<'a> &'a mut T: Into<&'a mut RectXYWH>,
{
    find_best_packing_impl(root, create_box(subjects, orders), subjects.len(), input)
}

/// Creates a `Box` containing all orders, contiguously.
/// This should be a performance improvement over having
/// several `Vec`s separately.
fn create_box<T, const N: usize>(
    subjects: &mut [T],
    orderers: [fn(RectXYWH, RectXYWH) -> Ordering; N],
) -> Box<[*mut RectXYWH]>
where
    for<'a> &'a mut T: Into<&'a mut RectXYWH>,
{
    let l = subjects.len();
    let mut orders = Box::<[*mut RectXYWH]>::new_uninit_slice(l * N);

    for (i, s) in subjects.iter_mut().enumerate() {
        let r: &mut RectXYWH = s.into();

        if r.area() > 0 {
            orders[i].write(r as *mut RectXYWH);
        }
    }

    let (src, tgt) = orders.split_at_mut(l);
    src.sort_by(unsafe { s(orderers[0]) });

    for (chunk, o) in tgt
        .chunks_exact_mut(l)
        .zip(orderers.iter().skip(1).copied())
    {
        chunk.copy_from_slice(src);
        chunk.sort_by(unsafe { s(o) });
    }

    unsafe { orders.assume_init() }
}

/// Convenience function that converts a "user-facing" sort function
/// to one that can be used with a slice of `MaybeUninit`. In other
/// words, converts from `Fn(RectXYWH, RectXYWH) -> Ordering` to
/// `Fn(&MaybeUninit<RectXYWH>, &MaybeUninit<RectXYWH>) -> Ordering`.
///
/// # Safety
/// This function assumes that the `MaybeUninit` slice you're
/// sorting with the returned function is fully initialized.
unsafe fn s(
    func: fn(RectXYWH, RectXYWH) -> Ordering,
) -> impl Fn(&MaybeUninit<*mut RectXYWH>, &MaybeUninit<*mut RectXYWH>) -> Ordering {
    move |lhs, rhs| func(unsafe { *lhs.assume_init() }, unsafe { *rhs.assume_init() }).reverse()
}
