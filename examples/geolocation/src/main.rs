use dioxus::prelude::*;

fn main() {
    // init debug tool for WebAssembly
    wasm_logger::init(wasm_logger::Config::default());
    console_error_panic_hook::set_once();

    dioxus_desktop::launch(app);
}

fn app(cx: Scope) -> Element {
    let geolocator = dioxus_std::geolocation::Geolocator::new().unwrap();
    let coords = geolocator.get_current_coordinates().unwrap();

    cx.render(rsx! (
        div {
            style: "text-align: center;",
            h1 { "Dioxus Geolocation Example" }
            h3 { "Your current location is:"}
            p { format!("Latitude: {} | Longitude: {} | Altitude: {}", coords.latitude, coords.longitude, coords.altitude) }
        }
    ))
}
