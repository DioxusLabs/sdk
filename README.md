<div align="center">
  <h1>🧰 Dioxus Development Kit 🚀</h1>
  <p><strong>Cross-platform crates for supercharging your productivity with Dioxus.</strong></p>
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

<p align="center"><i>These crates are still under development. Expect breaking changes!</i></p>
<br/>

`dioxus-sdk` is a development kit for Dioxus that provides cross-platform APIs for your Dioxus app. SDK is organized into many different crates accessible through the `dioxus-sdk` crate with the corresponding feature flags.

## Features
- `dioxus-storage`
- `dioxus-geolocation` - Web & Windows
- `dioxus-notifications` - Desktop
- `dioxus-window`
  - [x] Theme - (Web, Windows, Mac)
  - [x] Window Size
- `dioxus-time`
  - [x] Sleep
  - [x] Intervals
  - [x] Debounce
  - [x] Timeouts
- `dioxus-sync`
 - [x] Channels
- [ ] Camera
- [ ] WiFi
- [ ] Bluetooth

Geolocation example:

```rust
// dioxus-sdk = { version = "*", features = ["geolocation"] }
use dioxus::prelude::*;
use dioxus_sdk::geolocation::{
    init_geolocator, use_geolocation, PowerMode
};

#[component]
fn App() -> Element {
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

## Usage
You can add `dioxus-sdk` to your application by adding it to your dependencies.
```toml
[dependencies]
dioxus-sdk = { version = "0.7", features = [] }
```

## License
This project is dual licensed under the [MIT](./LICENSE-MIT) and [Apache 2.0](./LICENSE-APACHE) licenses.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in `dioxus-sdk` or any of it's crates, by you, shall be licensed as MIT or Apache 2.0, without any additional terms or conditions.
