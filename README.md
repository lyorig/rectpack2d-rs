# rectpack2d-rs

This is an attempt at a functionally 1:1 Rust port of the amazing [rectpack2D](https://github.com/TeamHypersomnia/rectpack2D) library. The motivation behind this is that it's used in a larger C++ project I'm rewriting in Rust, and as I failed to find a "proper" rewrite (only libraries "inspired" by it), I decided to undertake the effort myself.

## Implementation checklist
- [x] basic (`rect_xywh`) functionality
- [ ] flipping (`rect_xywhf`) functionality
- [x] ability to provide custom orders

I haven't yet figured out a "best practice" way to implement these, but if you're knowledgeable in the langauge, a PR would be much appreciated.
