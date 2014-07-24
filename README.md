SDL2_text
=========

A simple Rust library to draw text in SDL2.

Requirements
------------

* [rust-sdl2](https://github.com/AngryLawyer/rust-sdl2)
* [freetype-rs](https://github.com/PistonDevelopers/freetype-rs)

Building
--------

```
cargo build
make examples
```

Notes
-----
Differs from SDL2_ttf by rendering the font to a texture atlas, handles newlines.

An example using Arial font:

![](http://i.imgur.com/UMdQmal.png)
