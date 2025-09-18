use crate::{
    empty_spaces::{EmptySpaces, EmptySpacesProviderTrait},
    finders_interface::Input,
    rect_structs::{RectWH, RectXYWH, TotalAreaType},
};

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
    TotalArea(TotalAreaType),
    Rect(RectWH),
}

fn best_packing_for_ordering_impl(
    root: &mut EmptySpaces<impl EmptySpacesProviderTrait>,
    ordering: &[*mut RectXYWH],
    starting_bin: RectWH,
    mut discard_step: i32,
    tried_dimension: BinDimension,
) -> BestPackingReturn {
    let mut candidate_bin = starting_bin;
    let mut tries_before_discarding = 0;

    if discard_step <= 0 {
        tries_before_discarding = -discard_step;
        discard_step = 1
    }

    let starting_step = match tried_dimension {
        BinDimension::Both => {
            candidate_bin.w /= 2;
            candidate_bin.h /= 2;

            candidate_bin.w / 2
        }
        BinDimension::Width => {
            candidate_bin.w /= 2;

            candidate_bin.w / 2
        }
        BinDimension::Height => {
            candidate_bin.h /= 2;

            candidate_bin.h / 2
        }
    };

    let mut step = starting_step;
    loop {
        root.reset(candidate_bin);
        let mut total_inserted_area = 0;

        if all_inserted(ordering, root, &mut total_inserted_area) {
            if step <= discard_step {
                if tries_before_discarding > 0 {
                    tries_before_discarding -= 1;
                } else {
                    return BestPackingReturn::Rect(candidate_bin);
                }
            }

            match tried_dimension {
                BinDimension::Both => {
                    candidate_bin.w -= step;
                    candidate_bin.h -= step;
                }
                BinDimension::Width => {
                    candidate_bin.w -= step;
                }
                BinDimension::Height => {
                    candidate_bin.h -= step;
                }
            }

            root.reset(candidate_bin);
        } else {
            match tried_dimension {
                BinDimension::Both => {
                    candidate_bin.w += step;
                    candidate_bin.h += step;

                    if candidate_bin.area() > starting_bin.area() {
                        return BestPackingReturn::TotalArea(total_inserted_area);
                    }
                }
                BinDimension::Width => {
                    candidate_bin.w += step;

                    if candidate_bin.w > starting_bin.w {
                        return BestPackingReturn::TotalArea(total_inserted_area);
                    }
                }
                BinDimension::Height => {
                    candidate_bin.h += step;

                    if candidate_bin.h > starting_bin.h {
                        return BestPackingReturn::TotalArea(total_inserted_area);
                    }
                }
            }
        }

        step = 1.max(step / 2);
    }
}

fn best_packing_for_ordering(
    root: &mut EmptySpaces<impl EmptySpacesProviderTrait>,
    ordering: &[*mut RectXYWH],
    starting_bin: RectWH,
    discard_step: i32,
) -> BestPackingReturn {
    let best_result = try_pack(
        root,
        ordering,
        starting_bin,
        discard_step,
        BinDimension::Both,
    );

    if let BestPackingReturn::Rect(mut r) = best_result {
        trial(root, ordering, &mut r, discard_step, BinDimension::Width);
        trial(root, ordering, &mut r, discard_step, BinDimension::Height);
    }

    best_result
}

pub(crate) fn find_best_packing_impl<
    EST: EmptySpacesProviderTrait,
    F: Fn(RectXYWH) -> CallbackResult,
    G: Fn(RectXYWH) -> CallbackResult,
>(
    root: &mut EmptySpaces<EST>,
    orders: &mut [Vec<*mut RectXYWH>; 5],
    input: &Input<F, G>,
) -> RectWH {
    let max_bin = RectWH::new(input.max_bin_side, input.max_bin_side);

    let mut best_order: Option<&mut Vec<*mut RectXYWH>> = None;

    let mut best_total_inserted = -1;
    let mut best_bin = max_bin;

    for order in orders {
        for_each_order_lambda(
            root,
            order,
            max_bin,
            input.discard_step,
            &mut best_order,
            &mut best_total_inserted,
            &mut best_bin,
        );
    }

    assert!(best_order.is_some());
    root.reset(best_bin);

    for rr in best_order.as_mut().unwrap().iter_mut() {
        let rr = unsafe { &mut **rr };
        match root.insert(rr.into()) {
            Some(ret) => {
                // println!("Writing {ret:?}");
                *rr = ret;
                if let CallbackResult::AbortPacking = (input.handle_successful_insertion)(*rr) {
                    break;
                }
            }
            None => {
                if let CallbackResult::AbortPacking = (input.handle_unsuccessful_insertion)(*rr) {
                    break;
                }
            }
        }
    }

    root.get_rects_aabb()
}

fn all_inserted(
    ordering: &[*mut RectXYWH],
    root: &mut EmptySpaces<impl EmptySpacesProviderTrait>,
    total_inserted_area: &mut i32,
) -> bool {
    for rect in ordering {
        let rect = unsafe { rect.read() };
        if root.insert(RectWH::from_xywh(rect)).is_some() {
            *total_inserted_area += rect.area();
        } else {
            return false;
        }
    }

    true
}

fn try_pack(
    root: &mut EmptySpaces<impl EmptySpacesProviderTrait>,
    ordering: &[*mut RectXYWH],
    starting_bin: RectWH,
    discard_step: i32,
    tried_dimension: BinDimension,
) -> BestPackingReturn {
    best_packing_for_ordering_impl(root, ordering, starting_bin, discard_step, tried_dimension)
}

fn trial(
    root: &mut EmptySpaces<impl EmptySpacesProviderTrait>,
    ordering: &[*mut RectXYWH],
    best_bin: &mut RectWH,
    discard_step: i32,
    tried_dimension: BinDimension,
) {
    if let BestPackingReturn::Rect(better) =
        try_pack(root, ordering, *best_bin, discard_step, tried_dimension)
    {
        *best_bin = better;
    }
}

fn for_each_order_lambda<'a>(
    root: &mut EmptySpaces<impl EmptySpacesProviderTrait>,
    current_order: &'a mut Vec<*mut RectXYWH>,
    max_bin: RectWH,
    discard_step: i32,
    best_order: &mut Option<&'a mut Vec<*mut RectXYWH>>,
    best_total_inserted: &mut i32,
    best_bin: &mut RectWH,
) {
    match best_packing_for_ordering(root, current_order, max_bin, discard_step) {
        BestPackingReturn::TotalArea(total_inserted) => {
            if best_order.is_none() && total_inserted > *best_total_inserted {
                *best_order = Some(current_order);
                *best_total_inserted = total_inserted;
            }
        }
        BestPackingReturn::Rect(result_bin) => {
            if result_bin.area() <= best_bin.area() {
                *best_order = Some(current_order);
                *best_bin = result_bin;
            }
        }
    }
}
