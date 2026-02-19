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
    'a,
    EmptySpacesType: EmptySpacesProviderTrait,
    T: Iterator<Item = &'a mut RectXYWH>,
    F: Fn(RectXYWH) -> CallbackResult,
    G: Fn(RectXYWH) -> CallbackResult,
>(
    root: &mut EmptySpaces<EmptySpacesType>,
    subjects: T,
    input: &Input<F, G>,
) -> RectWH {
    let sortable = subjects
        .filter_map(|f| {
            let r: &mut RectXYWH = f.into();
            if r.area() > 0 {
                Some(r as *mut RectXYWH)
            } else {
                None
            }
        })
        .collect::<Box<_>>();

    find_best_packing_impl(root, &sortable, sortable.len(), input)
}

/// Forwards to `find_best_packing_ordered` with the following functions:
/// - `|l, r| l.area().cmp(&r.area())`
/// - `|l, r| l.perimeter().cmp(&r.perimeter())`
/// - `|l, r| l.w.max(l.h).cmp(&r.w.max(r.h))`
/// - `|l, r| l.w.cmp(&r.w)`
/// - `|l, r| l.h.cmp(&r.h)`
pub fn find_best_packing<
    'a,
    EmptySpacesType: EmptySpacesProviderTrait,
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
/// Accepts any iterator that returns `&mut RectXYWH`, but its implementation
/// of [`Iterator::size_hint()`] **must return a value as part of its upper bound**.
/// This is important for optimizing allocations, the function panics otherwise.
///
/// * `root` - Auxiliary storage for the algorithm.
/// * `subjects` - The rectangles to pack. Their `x` and `y` components are filled in.
/// * `input` - Settings for the algorithm.
/// * `orders` - Ordering functions to use.
pub fn find_best_packing_ordered<
    'a,
    EmptySpacesType: EmptySpacesProviderTrait,
    T: Iterator<Item = &'a mut RectXYWH>,
    F: Fn(RectXYWH) -> CallbackResult,
    G: Fn(RectXYWH) -> CallbackResult,
    const N: usize,
>(
    root: &mut EmptySpaces<EmptySpacesType>,
    subjects: T,
    input: &Input<F, G>,
    orders: [fn(RectXYWH, RectXYWH) -> Ordering; N],
) -> RectWH {
    let size_hint = subjects.size_hint().1.expect("No upper bound on size_hint");
    let mut buffer = Box::<[*mut RectXYWH]>::new_uninit_slice(size_hint * N);
    let (orders, chunk_size) = process_rects(subjects, &mut buffer, orders);

    find_best_packing_impl(root, orders, chunk_size, input)
}

/// Takes a slice of uninitialized rects, fills + sorts all chunks,
/// and returns the actual usable initialized part of the slice,
/// as well as the chunk size (a.k.a. the number of non-zero-area rects).
fn process_rects<'a, 'b, T: Iterator<Item = &'a mut RectXYWH>, const N: usize>(
    subjects: T,
    orders: &'b mut [MaybeUninit<*mut RectXYWH>],
    orderers: [fn(RectXYWH, RectXYWH) -> Ordering; N],
) -> (&'b [*mut RectXYWH], usize) {
    let mut n_valid = 0;
    for s in subjects {
        let r: &mut RectXYWH = s.into();

        if r.area() > 0 {
            orders[n_valid].write(r as *mut RectXYWH);
            n_valid += 1;
        }
    }

    let (src, tgt) = orders.split_at_mut(n_valid);
    src.sort_by(unsafe { s(orderers[0]) });

    for (chunk, o) in tgt
        .chunks_exact_mut(n_valid)
        .zip(orderers.iter().skip(1).copied())
    {
        chunk.copy_from_slice(src);
        chunk.sort_by(unsafe { s(o) });
    }

    (unsafe { orders[..n_valid * N].assume_init_ref() }, n_valid)
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
