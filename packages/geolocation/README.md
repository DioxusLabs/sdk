# Dioxus Geolocation
Geolocation utilities and hooks for Dioxus.

### Supports
- [x] Web
- [x] Windows
- [ ] Mac
- [ ] Linux
- [ ] Android
- [ ] iOs

## Usage
Add `dioxus-geolocation` to your `Cargo.toml`:
```toml
[dependencies]
dioxus-geolocation = "0.1"
```

Example:
```rs
use dioxus::prelude::*;
use dioxus_geolocation::{
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

### Dioxus Compatibility
This table represents the compatibility between this crate and Dioxus versions.
The crate version supports a Dioxus version up until the next crate version in the table.

E.g. if crate version `0.1` supported Dioxus `0.6` and crate version `0.4` supported Dioxus `0.7`, crate versions `0.1`, `0.2`, and `0.3` would support Dioxus `0.6`.

| Crate Version | Dioxus Version |
| ------------- | -------------- |
| 0.1           | 0.6            |