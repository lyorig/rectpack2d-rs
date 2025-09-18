use std::cmp::Ordering;

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
    let mut orders = [const { Vec::new() }; 5];

    {
        orders[0].clear();

        for s in subjects.iter_mut() {
            let r: &mut RectXYWH = s.into();

            if r.area() > 0 {
                orders[0].push(r as *mut RectXYWH);
            }
        }

        for i in 1..5 {
            orders[i] = orders[0].clone();
        }
    }

    orders[0].sort_by(s(|l, r| l.area().cmp(&r.area())));
    orders[1].sort_by(s(|l, r| l.perimeter().cmp(&r.perimeter())));
    orders[2].sort_by(s(|l, r| l.w.max(l.h).cmp(&r.w.max(r.h))));
    orders[3].sort_by(s(|l, r| l.w.cmp(&r.w)));
    orders[4].sort_by(s(|l, r| l.h.cmp(&r.h)));

    find_best_packing_impl(root, &mut orders, input)
}

/// Sorter shorthand function.
fn s<F: Fn(RectXYWH, RectXYWH) -> Ordering>(
    func: F,
) -> impl Fn(&*mut RectXYWH, &*mut RectXYWH) -> Ordering {
    move |lhs, rhs| func(unsafe { **lhs }, unsafe { **rhs }).reverse()
}
