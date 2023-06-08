<div align="center">
  <h1>🧰 Dioxus Standard Library 🚀</h1>
  <p><strong>A platform agnostic library for supercharging your productivity with Dioxus.</strong></p>
</div>

<div align="center">
  <!-- Crates version -->
  <a href="https://crates.io/crates/dioxus-std">
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

<p align="center"><i>This library is still under development. Expect breaking changes!</i></p>
<br/>

`dioxus-std` is a Dioxus standard library that provides abstractions for your Dioxus app. Abstractions included are notifications, clipboard, and more to come.

**Current Features**
- [x] Geolocation - (wasm, Windows)
- [x] Clipboard - (Desktop)
- [x] Notifications - (Desktop)
- [x] Utility Hooks 
  - use_channel - (any)
  - use_rw - (any) 
  - use_prefererred_color_scheme - (wasm)

**Planned Features**
- [ ] Camera
- [ ] WiFi
- [ ] Bluetooth

```rust
fn app(cx: Scope) -> Element {
    let geolocator = hooks::init_geolocator(cx, PowerMode::High).unwrap();
    let coords = use_geolocation(cx);

    match coords {
      Ok(coords) => {
        render! { p { format!("Latitude: {} | Longitude: {}", coords.latitude, coords.longitude) } }
      }
      Err(Error::NotInitialized) => {
        render! { p { "Initializing..." }}
      }
      Err(e) => {
        render! { p { "An error occured {e}" }}
      }
    }
}
```

## Platform Support
Currently `dioxus-std` primarily supports desktop targets. It is planned to support all of Dioxus' targets in the future.

- [x] Desktop (Windows, MacOS, Linux)
- [ ] Mobile  (Android, iOS)
- [ ] Web     (WASM)

On linux you need the x11 library to use the clipboard abstraction:
```
sudo apt-get install xorg-dev
```

## Installation
You can add `dioxus-std` to your application by adding it to your dependencies.
```toml
[dependencies]
dioxus-std =  { version = "0.2.0", features = [] }
```

## License
This project is licensed under the [MIT license].

[mit license]: ./LICENSE

Every contribution intentionally submitted for inclusion in `dioxus-std` by you, shall be licensed as MIT, without any additional
terms or conditions.
