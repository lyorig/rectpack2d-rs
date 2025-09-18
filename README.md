# rectpack2d-rs

This is a direct 1:1 Rust port of the amazing [rectpack2D](https://github.com/TeamHypersomnia/rectpack2D) library. The motivation behind this is that it's used in a larger C++ project I'm rewriting in Rust, and as I failed to find a "proper" rewrite (only libraries "inspired" by it), I decided to undertake the effort myself.

Some functionality is missing, namely rect flipping functionality. I haven't figured out a "best practice" way to implement this via traits or otherwise, but if you know of one, a PR will be much appreciated.
