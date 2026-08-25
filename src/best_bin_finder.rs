use std::{cmp::Ordering, mem::MaybeUninit};

use crate::{
    empty_spaces::{EmptySpaces, EmptySpacesProvider},
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

/// Shared algorithm state used during bin-packing search.
pub struct Finder {
    pub best_bin: RectWH,
    best_total_inserted: i32,
    pub best_order: Option<()>,
}

impl Finder {
    pub fn new(max_bin: RectWH) -> Self {
        Self {
            best_bin: max_bin,
            best_total_inserted: -1,
            best_order: None,
        }
    }

    /// Evaluate a single ordering: run the packing algorithm for this ordering
    /// and update the best-known result if this one is better.
    #[must_use = "The returned value indicates whether the best result should be updated"]
    pub fn evaluate_order(
        &mut self,
        root: &mut EmptySpaces<impl EmptySpacesProvider>,
        current_order: &[&RectXYWH],
        max_bin: RectWH,
        discard_step: i32,
    ) -> bool {
        match Self::best_packing_for_ordering(root, current_order, max_bin, discard_step) {
            BestPackingReturn::TotalArea(total_inserted) => {
                if self.best_order.is_none() && total_inserted > self.best_total_inserted {
                    self.best_order = Some(());
                    self.best_total_inserted = total_inserted;
                    return true;
                }
            }
            BestPackingReturn::Rect(result_bin) => {
                if result_bin.area() <= self.best_bin.area() {
                    self.best_order = Some(());
                    self.best_bin = result_bin;
                    return true;
                }
            }
        }

        false
    }

    fn best_packing_for_ordering_impl(
        root: &mut EmptySpaces<impl EmptySpacesProvider>,
        current_order: &[&RectXYWH],
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

            if Self::all_inserted(current_order, root, &mut total_inserted_area) {
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
        root: &mut EmptySpaces<impl EmptySpacesProvider>,
        ordering: &[&RectXYWH],
        starting_bin: RectWH,
        discard_step: i32,
    ) -> BestPackingReturn {
        let mut best_result = Self::try_pack(
            root,
            ordering,
            starting_bin,
            discard_step,
            BinDimension::Both,
        );

        if let BestPackingReturn::Rect(r) = &mut best_result {
            Self::trial(root, ordering, r, discard_step, BinDimension::Width);
            Self::trial(root, ordering, r, discard_step, BinDimension::Height);
        }

        best_result
    }

    fn all_inserted(
        ordering: &[&RectXYWH],
        root: &mut EmptySpaces<impl EmptySpacesProvider>,
        total_inserted_area: &mut i32,
    ) -> bool {
        for r in ordering {
            let rr = **r;
            if root.insert((&rr).into()).is_some() {
                *total_inserted_area += r.area();
            } else {
                return false;
            }
        }

        true
    }

    fn try_pack(
        root: &mut EmptySpaces<impl EmptySpacesProvider>,
        ordering: &[&RectXYWH],
        starting_bin: RectWH,
        discard_step: i32,
        tried_dimension: BinDimension,
    ) -> BestPackingReturn {
        Self::best_packing_for_ordering_impl(
            root,
            ordering,
            starting_bin,
            discard_step,
            tried_dimension,
        )
    }

    fn trial(
        root: &mut EmptySpaces<impl EmptySpacesProvider>,
        ordering: &[&RectXYWH],
        best_bin: &mut RectWH,
        discard_step: i32,
        tried_dimension: BinDimension,
    ) {
        if let BestPackingReturn::Rect(better) =
            Self::try_pack(root, ordering, *best_bin, discard_step, tried_dimension)
        {
            *best_bin = better;
        }
    }
}

pub struct Solver {
    orders: Box<[MaybeUninit<*mut RectXYWH>]>,
    count: usize,
}

impl Solver {
    pub fn new<'r, T: Iterator<Item = &'r mut RectXYWH>>(iter: T) -> Solver {
        let (orders, count) = Self::process_rects(iter);
        Solver { orders, count }
    }

    fn process_rects<'a, T: Iterator<Item = &'a mut RectXYWH>>(
        subjects: T,
    ) -> (Box<[MaybeUninit<*mut RectXYWH>]>, usize) {
        let upper = subjects
            .size_hint()
            .1
            .expect("Rect iterator should provide upper bound");

        let mut orders = Box::new_uninit_slice(upper * 2);

        let mut n_valid = 0;
        for s in subjects {
            if s.area() > 0 {
                orders[n_valid].write(s as *mut RectXYWH);
                n_valid += 1;
            }
        }

        (orders, n_valid)
    }

    fn copy_best(&mut self) {
        let orders = &mut self.orders[..self.count * 2];
        let (current, best) = orders.split_at_mut(self.count);

        best.copy_from_slice(current);
    }

    fn order_current(&self) -> &[&RectXYWH] {
        let slice = &self.orders[..self.count];
        unsafe { std::mem::transmute(slice) }
    }

    fn order_current_mut(&mut self) -> &mut [&mut RectXYWH] {
        let slice = &mut self.orders[..self.count];
        unsafe { std::mem::transmute(slice) }
    }

    fn order_best(&self) -> &[*mut RectXYWH] {
        let slice = &self.orders[self.count..self.count * 2];
        unsafe { std::mem::transmute(slice) }
    }

    pub fn find_best_packing_ordered<
        ESP: EmptySpacesProvider,
        F: Fn(RectXYWH) -> CallbackResult,
        G: Fn(RectXYWH) -> CallbackResult,
    >(
        &mut self,
        root: &mut EmptySpaces<ESP>,
        input: &Input<F, G>,
        orders: &[fn(&RectXYWH, &RectXYWH) -> Ordering],
    ) -> RectWH {
        let max_bin = RectWH::new(input.max_bin_side, input.max_bin_side);
        let mut finder = Finder::new(max_bin);

        for order in orders {
            self.order_current_mut().sort_by(|a, b| order(a, b));
            if finder.evaluate_order(root, self.order_current(), max_bin, input.discard_step) {
                self.copy_best();
            }
        }

        assert!(finder.best_order.is_some());

        root.reset(finder.best_bin);

        for rr in self.order_best().iter().copied() {
            let rect = unsafe { rr.as_mut_unchecked() };
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

        root.rects_aabb()
    }
}
