# Embedded ASCII debug font

An ASCII bitmap font embedded in a rust module for debugging purposes.

The font can be easily used with a GPU API such as `wgpu`.

# Generator

The embedded font atlas is generated via a small rust script in the generator folder using stb_TrueType.

In the `generator` folder:

Usage: `cargo run <font> [<destination>]`

```sh
# If the destination ends with ".rs", generates the embedded font data in a rust source file. 
$ cargo run ../assets/Hack-Regular.ttf generated_font_data.rs
# Equivalent to:
$ cargo run ../assets/Hack-Regular.ttf > generated_font_data.rs
# If the destination ends with ".png", generates an image containing the atlas. 
$ cargo run ../assets/Hack-Regular.ttf test.png
```
