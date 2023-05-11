use dioxus::prelude::*;
use dioxus_std::library::*;

fn main() {
    dioxus_desktop::launch(app);
}

fn app(cx: Scope) -> Element {
    let geolocator = geolocation::Geolocator::new(None, None).unwrap();
    let coords = geolocator.get_coordinates().unwrap();

    // Google maps embed api key
    let key = std::env::var("DIOXUS_GEOLOCATION_MAP_KEY").unwrap();

    cx.render(rsx! (
        div {
            style: "text-align: center;",
            h1 { "🗺️ Dioxus Geolocation Example 🛰️" }
            h3 { "Your current location is:"}
            p { format!("Latitude: {} | Longitude: {} | Altitude: {}", coords.latitude, coords.longitude, coords.altitude) }
        
            iframe {
                width: "400",
                height: "400",
                style: "border: 1px solid black",
                src: "https://www.google.com/maps/embed/v1/view?key={key}&center={coords.latitude},{coords.longitude}&zoom=16",
            }
        }
    ))
}
