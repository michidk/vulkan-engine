# ve_asset

This utility converts different game development assets into a more hardware-oriented format.

## Building

### Prerequisites

- [Rust](https://www.rust-lang.org/)

Then build with `cargo build`.

## Execute

Get an overview of the parameters with `ve_asset -h`.

For example, `ve_asset "./import/*" -o ./output` compiles all assets in the `/import` folder and outputs the artifacts to the `/output` folder.

## Files

Currently, this utility supports the following file formats:

|Format|Extension|Functionality|
|-|-|
|Wavefront|`.obj`|Converts to an indexed format|
