# bevy_immediate: immediate mode to easily build complex UI

[![bevy_version](https://img.shields.io/badge/bevy-0.16-blue)](https://github.com/bevy/bevy)
[![taffy_version](https://img.shields.io/badge/taffy-0.7-blue)](https://github.com/DioxusLabs/taffy)
[![Latest version](https://img.shields.io/crates/v/bevy_immediate.svg)](https://crates.io/crates/bevy_immediate)
[![Documentation](https://docs.rs/bevy_immediate/badge.svg)](https://docs.rs/bevy_immediate)
[![unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg)](https://github.com/rust-secure-code/safety-dance/)
[![License](https://img.shields.io/crates/l/bevy_immediate.svg)](https://crates.io/crates/bevy_immediate)

Powerful and easy to use UI library for bevy. Construct complex UI in one single function. No need to think about signals, observers, triggers, events, callbacks and ... 

Focus on what matters!

Additionally this library provides immediate mode API to easily manage entity hierarchies.

This library is in active development and some breaking changes are expected, but they will be kept as small as possible.

### 👉 [Web Demo](https://ppakalns.github.io/bevy_immediate/) 👈 of the latest released version.

## Version compatibility

| bevy_immediate | bevy | MSRV           |
|------------|------| ----------------|
| 0.1        | 0.16 | 1.83 (nightly) |

To use add `bevy_immediate` to your project dependencies in `Cargo.toml` file.

See [CHANGELOG](./CHANGELOG.md) for changes between versions.

## Examples

Check out `./examples/demo.rs` (cargo run --example demo).

### Demo example:



## Inspiration

This crate is made for [Settletopia](https://settletopia.com/)

This crate was inspired by my previous work on
* [egui_taffy](https://github.com/ppakalns/bevy_immediate/)

## Contributing

Contributions are welcome. Please add your improvements to examples so that it is easy to see and validate.
