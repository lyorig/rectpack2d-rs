use rectpack2d_rs::{
    best_bin_finder::CallbackResult,
    empty_space_allocators::DefaultEmptySpaces,
    empty_spaces::EmptySpaces,
    finders_interface::{Input, find_best_packing},
    rect_structs::RectXYWH,
};

fn main() {
    // The rectangles you'd like to sort.
    let mut subjects = [
        RectXYWH::from_wh(30, 40),
        RectXYWH::from_wh(256, 256),
        RectXYWH::from_wh(128, 512),
        RectXYWH::from_wh(512, 128),
    ];

    // The algorithm's auxiliary storage.
    // This one uses `DefaultEmptySpaces`, which allocates the heap via `Vec`,
    // but there's also `StaticEmptySpaces`, which uses an array of a user-provided size.
    let mut root = EmptySpaces::<DefaultEmptySpaces>::default();

    // Run the algorithm.
    // This will fill in the `x` and `y` components of the rectangles
    // present in `subjects`, and return the resulting bin size.
    let bin_size = find_best_packing(
        &mut root,
        subjects.iter_mut(),
        &Input::new(
            4096,
            4,
            |_| CallbackResult::ContinuePacking,
            |_| CallbackResult::AbortPacking,
        ),
    );

    println!("Bin size: {:?}", bin_size);
    println!("Subjects: {:?}", subjects);
}
