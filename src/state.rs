use std::{cmp::Ordering, mem::MaybeUninit};

use crate::{
    best_bin_finder::{BestPackingReturn, BinDimension, CallbackResult},
    empty_spaces::{EmptySpaces, EmptySpacesProviderTrait},
    finders_interface::Input,
    rect_structs::{RectWH, RectXYWH},
};

pub struct State<
    ESP: EmptySpacesProviderTrait,
    F: Fn(RectXYWH) -> CallbackResult,
    G: Fn(RectXYWH) -> CallbackResult,
    const N: usize,
> {
    root: EmptySpaces<ESP>,
    input: Input<F, G>,
    best_order: Option<RectXYWH>,
    max_bin: RectWH,
    best_bin: RectWH,

    orders: Box<[MaybeUninit<*mut RectXYWH>]>,
    funcs: [fn(RectXYWH, RectXYWH) -> Ordering; N],
    count: u32,
    total_inserted_area: i32,
    best_total_inserted: i32,
}

impl<
    ESP: EmptySpacesProviderTrait,
    F: Fn(RectXYWH) -> CallbackResult,
    G: Fn(RectXYWH) -> CallbackResult,
    const N: usize,
> State<ESP, F, G, N>
{
    fn new<'a, T: Iterator<Item = &'a mut RectXYWH>>(
        input: Input<F, G>,
        subjects: T,
        orders: [fn(RectXYWH, RectXYWH) -> Ordering; N],
    ) -> Self {
        let len = subjects
            .size_hint()
            .1
            .expect("Upper bound required on iterator");
        let mut bx = Box::new_uninit_slice(len);
        let actual_len = process_rects(subjects, &mut bx, &orders);

        Self {
            input,
            orders: bx,
            count: actual_len as _,
            funcs: orders,
            total_inserted_area: 0,
            root: Default::default(),
            best_order: Default::default(),
            max_bin: Default::default(),
            best_bin: Default::default(),
            best_total_inserted: 0,
        }
    }

    pub fn find_best_packing_impl(&mut self) -> RectWH {
        for func in self.funcs {
            self.for_each_order_lambda();
            self.order_staging().sort_by(unsafe { s(func) });
        }

        self.root.reset(self.best_bin);

        // TODO(perf): eliminate this allocation while satisfying borrowck.
        for rr in self.order_best().to_owned() {
            let rect = unsafe { rr.assume_init().as_mut_unchecked() };
            match self.root.insert(rect.into()) {
                Some(ret) => {
                    *rect = ret;
                    if let CallbackResult::AbortPacking =
                        (self.input.handle_successful_insertion)(*rect)
                    {
                        break;
                    }
                }
                None => {
                    if let CallbackResult::AbortPacking =
                        (self.input.handle_unsuccessful_insertion)(*rect)
                    {
                        break;
                    }
                }
            }
        }

        self.root.get_rects_aabb()
    }

    fn for_each_order_lambda(&mut self) {
        match self.best_packing_for_ordering() {
            BestPackingReturn::TotalArea(total_inserted) => {
                if self.best_order.is_none() && total_inserted > self.best_total_inserted {
                    self.order_best().copy_from_slice(self.order_staging());
                    self.best_total_inserted = total_inserted;
                }
            }
            BestPackingReturn::Rect(result_bin) => {
                if result_bin.area() <= best_bin.area() {
                    self.best_order = Some(current_order);
                    self.best_bin = result_bin;
                }
            }
        }
    }

    fn best_packing_for_ordering(&mut self) -> BestPackingReturn {
        let best_result = self.try_pack(self.max_bin, BinDimension::Both);

        if let BestPackingReturn::Rect(_) = best_result {
            self.trial(BinDimension::Width);
            self.trial(BinDimension::Height);
        }

        best_result
    }

    fn try_pack(
        &mut self,
        starting_bin: RectWH,
        tried_dimension: BinDimension,
    ) -> BestPackingReturn {
        self.best_packing_for_ordering_impl(starting_bin, tried_dimension)
    }

    fn best_packing_for_ordering_impl(
        &mut self,
        starting_bin: RectWH,
        tried_dimension: BinDimension,
    ) -> BestPackingReturn {
        let mut candidate_bin = starting_bin;
        let mut tries_before_discarding = 0;

        let mut discard_step = self.input.discard_step;
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
            self.root.reset(candidate_bin);

            if self.all_inserted() {
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

                self.root.reset(candidate_bin);
            } else {
                match tried_dimension {
                    BinDimension::Both => {
                        candidate_bin.w += step;
                        candidate_bin.h += step;

                        if candidate_bin.area() > starting_bin.area() {
                            return BestPackingReturn::TotalArea(self.total_inserted_area);
                        }
                    }
                    BinDimension::Width => {
                        candidate_bin.w += step;

                        if candidate_bin.w > starting_bin.w {
                            return BestPackingReturn::TotalArea(self.total_inserted_area);
                        }
                    }
                    BinDimension::Height => {
                        candidate_bin.h += step;

                        if candidate_bin.h > starting_bin.h {
                            return BestPackingReturn::TotalArea(self.total_inserted_area);
                        }
                    }
                }
            }

            step = 1.max(step / 2);
        }
    }

    fn all_inserted(&mut self) -> bool {
        // TODO(perf): eliminate this allocation while satisfying borrowck.
        for r in self.order_best().to_owned() {
            let rect = unsafe { r.assume_init().read() };
            if self.root.insert((&rect).into()).is_some() {
                self.total_inserted_area += rect.area();
            } else {
                return false;
            }
        }

        true
    }

    fn order_best(&self) -> &[MaybeUninit<*mut RectXYWH>] {
        &self.orders[self.count as usize..]
    }

    fn order_staging(&mut self) -> &mut [MaybeUninit<*mut RectXYWH>] {
        &mut self.orders[..self.count as usize]
    }

    fn trial(&mut self, tried_dimension: BinDimension) {
        if let BestPackingReturn::Rect(better) = self.try_pack(self.best_bin, tried_dimension) {
            self.best_bin = better;
        }
    }
}

/// Takes a slice of uninitialized rects, fills + sorts all chunks,
/// and returns the actual usable initialized part of the slice,
/// as well as the chunk size (a.k.a. the number of non-zero-area rects).
fn process_rects<'a, 'b, T: Iterator<Item = &'a mut RectXYWH>>(
    subjects: T,
    orders: &'b mut [MaybeUninit<*mut RectXYWH>],
    orderers: &[fn(RectXYWH, RectXYWH) -> Ordering],
) -> usize {
    let mut n_valid = 0;
    for s in subjects {
        if s.area() > 0 {
            orders[n_valid].write(s as *mut RectXYWH);
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

    let ord = unsafe { orders[..n_valid * 2].assume_init_mut() };
    let split = ord.split_at_mut(n_valid);
    split.1.copy_from_slice(split.0);

    n_valid
}

/// Convenience function that converts a "user-facing" sort function
/// to one that can be used with a slice of [`MaybeUninit`]. In other
/// words, converts from `Fn(RectXYWH, RectXYWH) -> Ordering` to
/// `Fn(&MaybeUninit<RectXYWH>, &MaybeUninit<RectXYWH>) -> Ordering`.
///
/// # Safety
/// This function assumes that the [`MaybeUninit`] slice you're
/// sorting with the returned function is fully initialized.
unsafe fn s(
    func: fn(RectXYWH, RectXYWH) -> Ordering,
) -> impl FnMut(&MaybeUninit<*mut RectXYWH>, &MaybeUninit<*mut RectXYWH>) -> Ordering {
    move |lhs, rhs| func(unsafe { *lhs.assume_init() }, unsafe { *rhs.assume_init() }).reverse()
}
