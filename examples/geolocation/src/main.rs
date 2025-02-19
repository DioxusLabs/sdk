use dioxus::prelude::*;
use dioxus_geolocation::{init_geolocator, use_geolocation, PowerMode};

fn main() {
    launch(App);
}

#[component]
fn App() -> Element {
    let geolocator = init_geolocator(PowerMode::High);
    let initial_coords = use_resource(move || async move {
        geolocator
            .read()
            .as_ref()
            .unwrap()
            .get_coordinates()
            .await
            .unwrap()
    });
    let latest_coords = use_geolocation();

    let latest_coords = match latest_coords() {
        Ok(v) => v,
        Err(e) => {
            let e = format!("Initializing: {:?}", e);
            return rsx!(p { "{e}" });
        }
    };

    // Google maps embed api key
    //let key = std::env::var("DIOXUS_GEOLOCATION_MAP_KEY").unwrap();

    rsx!(
        div {
            style: "text-align: center;",
            h1 { "🗺️ Dioxus Geolocation Example 🛰️" }
            h3 { "Your initial location is:"}

            p {
                if let Some(coords) = initial_coords.read().as_ref() {
                    "Latitude: {coords.latitude} | Longitude: {coords.longitude}"
                } else {
                    "Loading..."
                }
            }

            h3 { "Your latest location is:" }
            p { "Latitude: {latest_coords.latitude} | Longitude: {latest_coords.longitude}" }

            // Google maps embed
            //iframe {
            //    width: "400",
            //    height: "400",
            //    style: "border: 1px solid black",
            //    src: "https://www.google.com/maps/embed/v1/view?key={key}&center={latest_coords.latitude},{latest_coords.longitude}&zoom=16",
            //}
        }
    )
}
