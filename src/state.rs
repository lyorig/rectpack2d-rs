use std::{assert_matches, cmp::Ordering, mem::MaybeUninit};

use crate::{
    best_bin_finder::{BestPackingReturn, BinDimension, CallbackResult},
    empty_spaces::{EmptySpaces, EmptySpacesProvider},
    finders_interface::Input,
    rect_structs::{RectWH, RectXYWH},
};

pub struct State {
    orders: Box<[MaybeUninit<*mut RectXYWH>]>,
    max_bin: RectWH,
    best_bin: RectWH,

    count: u32,
    total_inserted_area: i32,
    best_total_inserted: i32,

    best_order: Option<()>,
}

impl State {
    pub fn new<'a, T: Iterator<Item = &'a mut RectXYWH>>(subjects: T) -> Self {
        let len = subjects
            .size_hint()
            .1
            .expect("Upper bound required on iterator");

        let mut bx = Box::new_uninit_slice(len * 2);
        let actual_len = process_rects(subjects, &mut bx);

        Self {
            orders: bx,
            count: actual_len as _,
            total_inserted_area: 0,
            best_order: Default::default(),
            max_bin: Default::default(),
            best_bin: Default::default(),
            best_total_inserted: -1,
        }
    }

    fn for_each_order_lambda<ESP: EmptySpacesProvider>(
        &mut self,
        root: &mut EmptySpaces<ESP>,
        discard_step: i32,
    ) -> bool {
        match self.best_packing_for_ordering(root, discard_step) {
            BestPackingReturn::TotalArea => {
                let total_inserted = self.total_inserted_area;
                if self.best_order.is_none() && total_inserted > self.best_total_inserted {
                    self.best_total_inserted = total_inserted;
                    return true;
                }
            }
            BestPackingReturn::Rect(result_bin) => {
                if result_bin.area() <= self.best_bin.area() {
                    self.best_bin = result_bin;
                    return true;
                }
            }
        }

        false
    }

    fn best_packing_for_ordering<ESP: EmptySpacesProvider>(
        &mut self,
        root: &mut EmptySpaces<ESP>,
        discard_step: i32,
    ) -> BestPackingReturn {
        let best_result = self.try_pack(root, self.max_bin, BinDimension::Both, discard_step);

        if let BestPackingReturn::Rect(ref _better) = best_result {
            self.trial(root, BinDimension::Width, discard_step);
            self.trial(root, BinDimension::Height, discard_step);
        }

        best_result
    }

    fn try_pack<ESP: EmptySpacesProvider>(
        &mut self,
        root: &mut EmptySpaces<ESP>,
        starting_bin: RectWH,
        tried_dimension: BinDimension,
        discard_step: i32,
    ) -> BestPackingReturn {
        self.best_packing_for_ordering_impl(root, starting_bin, tried_dimension, discard_step)
    }

    fn best_packing_for_ordering_impl<ESP: EmptySpacesProvider>(
        &mut self,
        root: &mut EmptySpaces<ESP>,
        starting_bin: RectWH,
        tried_dimension: BinDimension,
        mut discard_step: i32,
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

            if self.all_inserted(root) {
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
                            return BestPackingReturn::TotalArea;
                        }
                    }
                    BinDimension::Width => {
                        candidate_bin.w += step;

                        if candidate_bin.w > starting_bin.w {
                            return BestPackingReturn::TotalArea;
                        }
                    }
                    BinDimension::Height => {
                        candidate_bin.h += step;

                        if candidate_bin.h > starting_bin.h {
                            return BestPackingReturn::TotalArea;
                        }
                    }
                }
            }

            step = 1.max(step / 2);
        }
    }

    fn all_inserted<ESP: EmptySpacesProvider>(&mut self, root: &mut EmptySpaces<ESP>) -> bool {
        self.total_inserted_area = 0;

        // TODO(perf): eliminate this allocation while satisfying borrowck.
        for r in self.order_staging().to_owned() {
            let rect = unsafe { r.assume_init().read() };
            if root.insert((&rect).into()).is_some() {
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

    fn trial<ESP: EmptySpacesProvider>(
        &mut self,
        root: &mut EmptySpaces<ESP>,

        tried_dimension: BinDimension,
        discard_step: i32,
    ) {
        if let BestPackingReturn::Rect(better) =
            self.try_pack(root, self.best_bin, tried_dimension, discard_step)
        {
            self.best_bin = better;
        }
    }

    pub fn find_best_packing_ordered<
        ESP: EmptySpacesProvider,
        F: Fn(RectXYWH) -> CallbackResult,
        G: Fn(RectXYWH) -> CallbackResult,
    >(
        &mut self,
        root: &mut EmptySpaces<ESP>,
        input: &Input<F, G>,
        orders: &[fn(RectXYWH, RectXYWH) -> Ordering],
    ) -> RectWH {
        for func in orders.iter().copied() {
            self.order_staging().sort_by(unsafe { s(func) });
            if self.for_each_order_lambda(root, input.discard_step) {
                self.copy_best();
                self.best_order = Some(());
            }
        }

        assert_matches!(self.best_order, Some(()));

        root.reset(self.best_bin);

        // TODO(perf): eliminate this allocation while satisfying borrowck.
        for rr in self.order_best().to_owned() {
            let rect = unsafe { rr.assume_init().as_mut_unchecked() };

            match root.insert(rect.into()) {
                Some(ret) => {
                    *rect = ret;
                    if let CallbackResult::AbortPacking = (input.handle_successful_insertion)(*rect)
                    {
                        break;
                    }
                }
                None => {
                    if let CallbackResult::AbortPacking =
                        (input.handle_unsuccessful_insertion)(*rect)
                    {
                        break;
                    }
                }
            }
        }

        root.get_rects_aabb()
    }

    pub fn find_best_packing_dont_sort<
        ESP: EmptySpacesProvider,
        F: Fn(RectXYWH) -> CallbackResult,
        G: Fn(RectXYWH) -> CallbackResult,
    >(
        &mut self,
        root: &mut EmptySpaces<ESP>,
        i: &Input<F, G>,
    ) -> RectWH {
        _ = self.for_each_order_lambda(root, i.discard_step);

        root.reset(self.best_bin);

        // TODO(perf): eliminate this allocation while satisfying borrowck.
        for rr in self.order_best().to_owned() {
            let rect = unsafe { rr.assume_init().as_mut_unchecked() };

            match root.insert(rect.into()) {
                Some(ret) => {
                    *rect = ret;
                    if let CallbackResult::AbortPacking = (i.handle_successful_insertion)(*rect) {
                        break;
                    }
                }
                None => {
                    if let CallbackResult::AbortPacking = (i.handle_unsuccessful_insertion)(*rect) {
                        break;
                    }
                }
            }
        }

        root.get_rects_aabb()
    }

    fn copy_best(&mut self) {
        // TODO: use unchecked variant.
        let (staging, best) = self.orders.split_at_mut(self.count as _);
        best.copy_from_slice(staging)
    }
}

/// Takes a slice of uninitialized rects, fills it with non-zero rects,
/// and returns their count.
fn process_rects<'a, 'b, T: Iterator<Item = &'a mut RectXYWH>>(
    subjects: T,
    orders: &'b mut [MaybeUninit<*mut RectXYWH>],
) -> usize {
    let mut n_valid = 0;
    for s in subjects {
        if s.area() > 0 {
            orders[n_valid].write(s as *mut RectXYWH);
            n_valid += 1;
        }
    }

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
