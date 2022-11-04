<div align="center">
  <h1>🧰 Dioxus Standard Library 🚀</h1>
  <p><strong>A platform agnostic library for supercharging your productivity with Dioxus</strong></p>
</div>

<div align="center">
  <!-- Crates version -->
  <a href="https://crates.io/crates/dioxus">
    <img src="https://img.shields.io/crates/v/dioxus-std.svg?style=flat-square"
    alt="Crates.io version" />
  </a>
  <!-- Downloads -->
  <a href="https://crates.io/crates/dioxus-std">
    <img src="https://img.shields.io/crates/d/dioxus-std.svg?style=flat-square"
      alt="Download" />
  </a>
  <!-- docs -->
  <a href="https://docs.rs/dioxus-std">
    <img src="https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square"
      alt="docs.rs docs" />
  </a>
</div>

-----

`dioxus-std` is a Dioxus standard library for utilizing common functionality in your Dioxus app such as notifications, clipboard, audio, and more.

```rust, ignore
fn app() {
    // TODO: Add example
}
```

## Platform Support
Currently `dioxus-std` only supports desktop targets. It is planned to support all of Dioxus' targets in the future.


- [x] Desktop (Windows, MacOS, Linux)
- [ ] Mobile  (Android, iOS)
- [ ] Web     (WASM)

On linux you need the x11 library:
```
sudo apt-get install xorg-dev
```

## Installation
You can add `dioxus-std` to your application by adding it to your dependencies.
```toml
[dependencies]
dioxus-std = "0.1.0"
```

