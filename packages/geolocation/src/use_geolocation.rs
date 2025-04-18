//! Provides an initialization and use_geolocation hook.

use super::core::{Error, Event, Geocoordinates, Geolocator, PowerMode, Status};
use dioxus::{
    prelude::{
        ReadOnlySignal, Signal, UnboundedReceiver, provide_context, try_consume_context,
        use_coroutine, use_hook, use_signal,
    },
    signals::{Readable, Writable},
};
use futures_util::stream::StreamExt;
use std::sync::Once;

static INIT: Once = Once::new();

/// Provides the latest geocoordinates. Good for navigation-type apps.
pub fn use_geolocation() -> ReadOnlySignal<Result<Geocoordinates, Error>> {
    // Store the coords
    let mut coords: Signal<Result<Geocoordinates, Error>> =
        use_signal(|| Err(Error::NotInitialized));

    // Initialize the handler of events
    let listener = use_coroutine(move |mut rx: UnboundedReceiver<Event>| async move {
        while let Some(event) = rx.next().await {
            match event {
                Event::NewGeocoordinates(new_coords) => {
                    *coords.write() = Ok(new_coords);
                }
                Event::StatusChanged(Status::Disabled) => {
                    *coords.write() = Err(Error::AccessDenied);
                }
                _ => {}
            }
        }
    });

    // Try getting the geolocator and starting the listener.
    match try_consume_context::<Signal<Result<Geolocator, Error>>>() {
        Some(geo) => {
            let geo = geo.read();
            match geo.as_ref() {
                Ok(geolocator) => {
                    INIT.call_once(|| {
                        geolocator.listen(listener).ok();
                    });
                }
                Err(e) => coords.set(Err(e.clone())),
            }
        }
        None => {
            coords.set(Err(Error::NotInitialized));
        }
    }

    use_hook(|| ReadOnlySignal::new(coords))
}

/// Must be called before any use of the geolocation abstraction.
pub fn init_geolocator(power_mode: PowerMode) -> Signal<Result<Geolocator, Error>> {
    use_hook(|| {
        let geolocator = Signal::new(Geolocator::new(power_mode));
        provide_context(geolocator)
    })
}
