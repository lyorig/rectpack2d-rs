use rectpack2d_rs::{
    best_bin_finder::CallbackResult,
    empty_space_allocators::DefaultEmptySpaces,
    empty_spaces::EmptySpaces,
    finders_interface::{Input, find_best_packing},
    rect_structs::RectXYWH,
};

fn rect(w: i32, h: i32) -> RectXYWH {
    RectXYWH::new(0, 0, w, h)
}

fn main() {
    let mut rects = [rect(30, 40), rect(256, 256), rect(128, 512), rect(512, 128)];
    let mut root = EmptySpaces::<DefaultEmptySpaces>::default();
    let l = find_best_packing(
        &mut root,
        &mut rects,
        &Input {
            max_bin_side: 4096,
            discard_step: 4,
            handle_successful_insertion: |_| CallbackResult::ContinuePacking,
            handle_unsuccessful_insertion: |_| CallbackResult::AbortPacking,
        },
    );

    println!("Atlas size: {:?}", l);
    println!("Members: {:?}", rects);
}
