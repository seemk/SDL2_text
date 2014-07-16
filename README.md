SDL2_text
=========

A convenience wrapper over rust-sdl2_ttf for text drawing.

Requirements
------------

* [rust-sdl2](https://github.com/AngryLawyer/rust-sdl2)
* [rust-sdl2_ttf](https://github.com/andelf/rust-sdl2_ttf)

Building
--------

```
cargo build
make examples
```

Notes
--------
Characters are rendered to a texture atlas. SDL2_ttf doesn't provide kerning info for separate characters, so currently monospaced fonts are better suited for drawing.
