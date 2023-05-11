use dioxus::prelude::*;
use dioxus_std::library::*;

fn main() {
    dioxus_desktop::launch(app);
}

fn app(cx: Scope) -> Element {
    //let geolocator = geolocation::Geolocator::new().unwrap();
    //let coords = geolocator.get_current_coordinates().unwrap();
    let geolocator = geolocation::Geolocator::new().unwrap();
    let coords = geolocator.get_coordinates().unwrap();

    cx.render(rsx! (
        div {
            style: "text-align: center;",
            h1 { "Dioxus Geolocation Example" }
            h3 { "Your current location is:"}
            p { format!("Latitude: {} | Longitude: {} | Altitude: {}", coords.latitude, coords.longitude, coords.altitude) }
        }
    ))
}
