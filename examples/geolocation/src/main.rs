use dioxus::prelude::*;
use dioxus_std::hooks::{self, use_geolocation};

fn main() {
    dioxus_desktop::launch(app);
}

fn app(cx: Scope) -> Element {
    let geolocator = hooks::init_geolocator(cx, None, None).unwrap();
    let coords = geolocator.get_coordinates().unwrap();

    let geo_result = use_geolocation(cx).unwrap();

    // Google maps embed api key
    let key = std::env::var("DIOXUS_GEOLOCATION_MAP_KEY").unwrap();

    cx.render(rsx! (
        div {
            style: "text-align: center;",
            h1 { "🗺️ Dioxus Geolocation Example 🛰️" }
            h3 { "Your initial location is:"}
            p { format!("Latitude: {} | Longitude: {} | Altitude: {}", coords.latitude, coords.longitude, coords.altitude) }
            h3 { "Your latest location is:" }
            p { format!("Latitude: {} | Longitude: {} | Altitude: {}", geo_result.latitude, geo_result.longitude, geo_result.altitude) }

            iframe {
                width: "400",
                height: "400",
                style: "border: 1px solid black",
                src: "https://www.google.com/maps/embed/v1/view?key={key}&center={geo_result.latitude},{geo_result.longitude}&zoom=16",
            }
        }
    ))
}
