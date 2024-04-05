<div align="center">
  <h1>🧰 Dioxus Development Kit 🚀</h1>
  <p><strong>A platform agnostic library for supercharging your productivity with Dioxus.</strong></p>
</div>

<div align="center">
  <!-- Crates version -->
  <a href="https://crates.io/crates/dioxus-sdk">
    <img src="https://img.shields.io/crates/v/dioxus-sdk.svg?style=flat-square"
    alt="Crates.io version" />
  </a>
  <!-- Downloads -->
  <a href="https://crates.io/crates/dioxus-sdk">
    <img src="https://img.shields.io/crates/d/dioxus-sdk.svg?style=flat-square"
      alt="Download" />
  </a>
  <!-- docs -->
  <a href="https://docs.rs/dioxus-sdk">
    <img src="https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square"
      alt="docs.rs docs" />
  </a>
</div>

-----

<p align="center"><i>This library is still under development. Expect breaking changes!</i></p>
<br/>

`dioxus-sdk` is a development kit for Dioxus that provides abstractions for your Dioxus app. Abstractions included are notifications, clipboard, geolocation and storage with more to come!

**Features**
- [x] Geolocation - (Web, Windows)
- [x] Storage - (Web, Desktop)
- [x] Clipboard - (Desktop)
- [x] Notifications - (Desktop)
- [x] Color Scheme - (Web)
- [x] i18n
- [x] Utility Hooks 
  - [x] use_channel
  - [ ] use_interval
- [ ] Camera
- [ ] WiFi
- [ ] Bluetooth

Geolocation example:

```rust
use dioxus_sdk::geolocation::{
    init_geolocator, use_geolocation, PowerMode
};

fn app() -> Element {
    let geolocator = init_geolocator(PowerMode::High).unwrap();
    let coords = use_geolocation();

    match coords {
      Ok(coords) => {
        rsx!( p { "Latitude: {coords.latitude} | Longitude: {coords.longitude}" } )
      }
      Err(Error::NotInitialized) => {
        rsx!( p { "Initializing..." } )
      }
      Err(e) => {
        rsx!( p { "An error occurred {e}" } )
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
You can add `dioxus-sdk` to your application by adding it to your dependencies.
```toml
[dependencies]
dioxus-sdk=  { version = "0.5", features = [] }
```

## License
This project is licensed under the [MIT license].

[mit license]: ./LICENSE

Every contribution intentionally submitted for inclusion in `dioxus-sdk` by you, shall be licensed as MIT, without any additional terms or conditions.
