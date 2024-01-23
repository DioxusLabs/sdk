use dioxus::prelude::*;
use dioxus_std::geolocation::{init_geolocator, use_geolocation, PowerMode};

fn main() {
    dioxus_desktop::launch(app);
    //dioxus_web::launch(app);
}

fn app() -> Element {
    let geolocator = init_geolocator(PowerMode::High).unwrap();
    let initial_coords = use_future(|_| async move { geolocator.get_coordinates().await.unwrap() });
    let latest_coords = use_geolocation();

    let latest_coords = match latest_coords {
        Ok(v) => v,
        Err(e) => {
            let e = format!("Initializing: {:?}", e);
            return cx.render(rsx!(p { "{e}" }));
        }
    };

    // Google maps embed api key
    //let key = std::env::var("DIOXUS_GEOLOCATION_MAP_KEY").unwrap();

    let initial_coords = initial_coords.value();

    cx.render(rsx! (
        div {
            style: "text-align: center;",
            h1 { "🗺️ Dioxus Geolocation Example 🛰️" }
            h3 { "Your initial location is:"}

            p {
                if let Some(coords) = initial_coords {
                    format!("Latitude: {} | Longitude: {}", coords.latitude, coords.longitude) 
                } else {
                    "Loading...".to_string()
                }
            }

            h3 { "Your latest location is:" }
            p { format!("Latitude: {} | Longitude: {}", latest_coords.latitude, latest_coords.longitude) }

            // Google maps embed
            //iframe {
            //    width: "400",
            //    height: "400",
            //    style: "border: 1px solid black",
            //    src: "https://www.google.com/maps/embed/v1/view?key={key}&center={latest_coords.latitude},{latest_coords.longitude}&zoom=16",
            //}
        }
    ))
}
