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

**Features**
- [x] Geolocation - (wasm, Windows)
- [x] Clipboard - (Desktop)
- [x] Notifications - (Desktop)
- [x] Color Scheme - (any)
- [x] Utility Hooks 
  - [x] use_rw - (any)
  - [x] use_channel - (any)
  - [ ] use_interval (any)
- [x] i18n - (any)
- [ ] Camera (Desktop)
- [ ] WiFi (Desktop)
- [ ] Bluetooth (Desktop)

Geolocation example:

```rust
use dioxus_std::geolocation::{
    init_geolocator, use_geolocation, PowerMode
};

fn app(cx: Scope) -> Element {
    let geolocator = init_geolocator(cx, PowerMode::High).unwrap();
    let coords = use_geolocation(cx);

    match coords {
      Ok(coords) => {
        rsx! { p { format!("Latitude: {} | Longitude: {}", coords.latitude, coords.longitude) } }
      }
      Err(Error::NotInitialized) => {
        rsx! { p { "Initializing..." }}
      }
      Err(e) => {
        rsx! { p { "An error occurred {e}" }}
      }
    }
}
```

## Platform Support
### Clipboard

On linux you need the x11 library to use the clipboard abstraction:
```
sudo apt-get install xorg-dev
```

## Usage
You can add `dioxus-std` to your application by adding it to your dependencies.
```toml
[dependencies]
dioxus-std =  { version = "0.5", features = [] }
```

## License
This project is licensed under the [MIT license].

[mit license]: ./LICENSE

Every contribution intentionally submitted for inclusion in `dioxus-std` by you, shall be licensed as MIT, without any additional terms or conditions.
