# ve_asset

This utility converts different game development assets into a more hardware-oriented format.

## Building

### Prerequisites

- [Rust](https://www.rust-lang.org/)

Then build with `cargo build`.

## Execute

Get an overview of the parameters with `ve_asset -h`.

For example, `ve_asset "./import/" -o ./output` compiles all assets in the `/import` folder and outputs the artifacts to the `/output` folder.

## Files

Currently, this utility supports the following file formats:

| Format | Extension | Functionality | More Info/Settings |
| --- | --- | --- | --- |
| [Wavefront OBJ](https://en.wikipedia.org/wiki/Wavefront_.obj_file) | `.obj` | Converts to an indexed format | [here](./src/mesh/README.md) |

## Meta

Formats like `.obj` have to be parsed with different settings, e.g., depending on the software it was made with. These settings can be set in `.toml` files that are named the same as the corresponding file. This metadata can also be set for a whole folder in a `<file_extension>.toml` file.

The meta-file for the `.obj` file type looks like this:

```toml
calculate_normals = false
flip_axis = [true, false, false]
```

If the corresponding file is named `test.obj`, set the metadata either in `test.toml` or `obj.toml` for all `.obj` files in that folder.
