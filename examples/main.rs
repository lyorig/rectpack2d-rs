use rectpack2d_rs::{
    best_bin_finder::CallbackResult,
    empty_space_allocators::DefaultEmptySpaces,
    empty_spaces::EmptySpaces,
    finders_interface::{Input, find_best_packing},
    rect_structs::RectXYWH,
};

fn main() {
    let mut rects = [
        RectXYWH::from_wh(30, 40),
        RectXYWH::from_wh(256, 256),
        RectXYWH::from_wh(128, 512),
        RectXYWH::from_wh(512, 128),
    ];

    let mut root = EmptySpaces::<DefaultEmptySpaces>::default();

    let l = find_best_packing(
        &mut root,
        &mut rects,
        &Input::new(
            4096,
            4,
            |_| CallbackResult::ContinuePacking,
            |_| CallbackResult::AbortPacking,
        ),
    );

    println!("Atlas size: {:?}", l);
    println!("Members: {:?}", rects);
}
