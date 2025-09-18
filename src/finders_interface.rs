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

    orders[0].sort_by(|l, r| {
        let l = unsafe { **l };
        let r = unsafe { **r };
        l.area().cmp(&r.area()).reverse()
    });
    orders[1].sort_by(|l, r| {
        let l = unsafe { **l };
        let r = unsafe { **r };
        l.perimeter().cmp(&r.perimeter()).reverse()
    });
    orders[2].sort_by(|l, r| {
        let l = unsafe { **l };
        let r = unsafe { **r };
        l.w.max(l.h).cmp(&r.w.max(r.h)).reverse()
    });
    orders[3].sort_by(|l, r| {
        let l = unsafe { **l };
        let r = unsafe { **r };
        l.w.cmp(&r.w).reverse()
    });
    orders[4].sort_by(|l, r| {
        let l = unsafe { **l };
        let r = unsafe { **r };
        l.h.cmp(&r.h).reverse()
    });

    find_best_packing_impl(root, &mut orders, input)
}
