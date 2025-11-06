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
Add `dioxus-sdk-geolocation` to your `Cargo.toml`:
```toml
[dependencies]
dioxus-sdk-geolocation = "0.1"
```

Example:
```rs
use dioxus::prelude::*;
use dioxus_sdk_geolocation::{
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
