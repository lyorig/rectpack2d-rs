pub mod best_bin_finder;
pub mod empty_space_allocators;
pub mod empty_spaces;
pub mod finders_interface;
pub mod insert_and_split;
pub mod rect_structs;

#[cfg(test)]
mod tests {
    use crate::{
        best_bin_finder::CallbackResult,
        empty_space_allocators::DefaultEmptySpaces,
        empty_spaces::EmptySpaces,
        finders_interface::{Input, find_best_packing},
        rect_structs::{RectWH, RectXYWH},
    };

    #[test]
    fn basic_usage() {
        let mut subjects = [
            RectXYWH::from_wh(30, 40),
            RectXYWH::from_wh(256, 256),
            RectXYWH::from_wh(128, 512),
            RectXYWH::from_wh(512, 128),
        ];

        let mut root = EmptySpaces::<DefaultEmptySpaces>::default();

        let input = Input {
            max_bin_side: 4096,
            discard_step: 4,
            handle_successful_insertion: |_| CallbackResult::ContinuePacking,
            handle_unsuccessful_insertion: |_| CallbackResult::AbortPacking,
        };

        let result = find_best_packing(&mut root, &mut subjects, &input);

        assert_eq!(result, RectWH::new(640, 512));
        assert_eq!(
            subjects,
            [
                RectXYWH::new(128, 384, 30, 40),
                RectXYWH::new(128, 0, 256, 256),
                RectXYWH::new(0, 0, 128, 512),
                RectXYWH::new(128, 256, 512, 128),
            ]
        )
    }

    #[test]
    fn structured() {
        #[derive(Debug, PartialEq)]
        struct Foo(RectXYWH);

        impl<'a> From<&'a Foo> for &'a RectXYWH {
            fn from(value: &'a Foo) -> Self {
                &value.0
            }
        }

        impl<'a> From<&'a mut Foo> for &'a mut RectXYWH {
            fn from(value: &'a mut Foo) -> Self {
                &mut value.0
            }
        }

        let mut subjects = [
            Foo(RectXYWH::from_wh(100, 40)), // top
            Foo(RectXYWH::from_wh(40, 20)),  // center
            Foo(RectXYWH::from_wh(70, 20)),  // bottom
            Foo(RectXYWH::from_wh(30, 40)),  // left
            Foo(RectXYWH::from_wh(30, 20)),  // right
        ];

        let mut root = EmptySpaces::<DefaultEmptySpaces>::default();

        let input = Input {
            max_bin_side: 4096,
            discard_step: 4,
            handle_successful_insertion: |_| CallbackResult::ContinuePacking,
            handle_unsuccessful_insertion: |_| CallbackResult::AbortPacking,
        };

        let result = find_best_packing(&mut root, &mut subjects, &input);

        println!("{result:?}");
        println!("{subjects:?}",)
    }
}
