# masslynx-rs

Rust bindings for (a subset of) Waters Corporation MassLynxRaw 4.11.

These bindings are used to link a Rust crate with the Waters `MassLynxRaw.lib` file provided by the
Waters Corporation.

This library is only available for Windows. Waters does not provide libraries for other platforms.

To build this library, you must obtain a copy of the relevant C library from Waters. The `MassLynxRaw.lib`
file should be on your linker's include path. For convenience, the `build.rs` script will put `./lib` on
the path automatically.

## Usage

See `main` for a brief usage example.

## Modules

- `constants` - The enums that map entities in the C API.
- `ffi` - The raw bindings to the C API are defined here.
- `base` - The low-level Rust wrappers of the C API that perform a modicum of error handling.
- `reader` - A modestly higher level wrapper around `base` to exercise all the functions.

