# Vulkan Engine

![Continuous integration](https://github.com/michidk/vulkan-renderer/workflows/Continuous%20Integration/badge.svg)

This repository contains a playground project by [Jonas](https://github.com/Shemnei), [Robin](https://github.com/Rob2309), and [Michael](https://github.com/michidk) to learn the Vulkan graphics API. It uses the [Rust language](https://www.rust-lang.org/) and the [Ash](https://github.com/MaikKlein/ash) Vulkan wrapper.

## Goal

The goal is to build a somewhat useable game engine and a demo using it. While we are probably never going to implement a fully-fledged engine, we try to make shortcuts to implement specific parts of it that we find interesting. It is more a learning project than anything else.

## Features

Currently implemented features are:
- BRDF shading
- Deferred rendering
- `.obj` parser
- Runs on both Linux and Windows

## Screenshots

BRDF testing:

![brdf testing](./.github/images/examples/brdf.png)

## Workspace

| Folder | Description | Readme |
| ---- | ----------- | - |
| `engine` | Main engine library | This one |
| `ve_asset` | Utility that converts files into our custom format | [here](./ve_asset/README.md) |
| `ve_format` | Stores some shared structs | [here](./ve_format/README.md) |
| `ve_shader_reflect` | Contains custom derive macros | [here](./ve_shader_reflect/README.md) |



## Examples

Examples are in the `/examples` folder. They can be run with `cargo +nightly run --example <name>`.
| Name | Description |
| ---- | ----------- |
| minimal | Displays a triangle using vertex colors |
| brdf | Renders using physically-based rendering |
| mesh | Loads and renders a custom mesh |

## Building

### Prerequisites

- [Rust](https://www.rust-lang.org/) (2021 Edition)
- [Vulkan SDK](https://www.lunarg.com/vulkan-sdk/) (at least v1.2.189.2)

Build with `make build` or run an exmaple with `make run`.

## Resources

- [Vulkan Specs](https://www.khronos.org/registry/vulkan/specs/1.0/html/)
- [Vulkan Tutorial](https://vulkan-tutorial.com/Introduction)
- [Ask Documentation](https://docs.rs/ash/0.31.0/ash/)
