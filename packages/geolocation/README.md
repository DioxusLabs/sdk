# Dioxus Geolocation
Geolocation utilities and hooks for Dioxus.

### Supports
- [x] Web
- [x] Windows
- [x] Android (draft)
- [ ] Mac
- [ ] Linux
- [ ] iOS

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

## Platform Notes

### Android

> **Note:** Android support requires `dioxus = "0.7.0-alpha0"` or later.

The Android implementation provides robust geolocation support via JNI:

**Features:**
- **Automatic permission handling** - Requests `ACCESS_FINE_LOCATION` (GPS) or `ACCESS_COARSE_LOCATION` (network) based on `PowerMode`
- **Power mode support** - `PowerMode::High` uses GPS provider, `PowerMode::Low` uses network provider
- **Continuous location updates** - Background polling with configurable intervals (1s for High, 5s for Low accuracy)
- **Change detection** - Only emits `NewGeocoordinates` events when position changes (0.00001 degree threshold)
- **Permission monitoring** - Detects permission revocation during active listening
- **Automatic cleanup** - Listener thread stops when `Geolocator` is dropped

**Required Permissions:**

Add to your `AndroidManifest.xml`:
```xml
<uses-permission android:name="android.permission.ACCESS_FINE_LOCATION" />
<uses-permission android:name="android.permission.ACCESS_COARSE_LOCATION" />
```
