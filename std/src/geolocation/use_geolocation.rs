//! Provides an initialization and use_geolocation hook.

use super::core::{Error, Event, Geocoordinates, Geolocator, PowerMode, Status};
use dioxus::prelude::{use_coroutine, ScopeState, UnboundedReceiver};
use futures_util::stream::StreamExt;
use std::{rc::Rc, sync::Once};

use crate::utils::rw::{use_rw, UseRw};

static INIT: Once = Once::new();

/// Provides the latest geocoordinates. Good for navigation-type apps.
pub fn use_geolocation(cx: &ScopeState) -> Result<Geocoordinates, Error> {
    // Store the coords
    let coords: &mut UseRw<Result<Geocoordinates, Error>> =
        use_rw(cx, || Err(Error::NotInitialized));

    // Get geolocator
    let geolocator = match cx.consume_context::<Rc<Geolocator>>() {
        Some(v) => v,
        None => return Err(Error::NotInitialized),
    };

    let coords_cloned = coords.clone();

    // Initialize the handler of events
    let listener = use_coroutine(cx, |mut rx: UnboundedReceiver<Event>| async move {
        while let Some(event) = rx.next().await {
            match event {
                Event::NewGeocoordinates(new_coords) => {
                    let _ = coords_cloned.write(Ok(new_coords));
                }
                Event::StatusChanged(Status::Disabled) => {
                    let _ = coords_cloned.write(Err(Error::AccessDenied));
                }
                _ => {}
            }
        }
    });

    // Start listening
    INIT.call_once(|| {
        let _ = geolocator.listen(listener.clone());
    });

    // Get the result and return a clone
    coords.read().map_err(|_| Error::Poisoned)?.clone()
}

/// Must be called before any use of the geolocation abstraction.
pub fn init_geolocator(cx: &ScopeState, power_mode: PowerMode) -> Result<Rc<Geolocator>, Error> {
    let geolocator = Geolocator::new(power_mode)?;
    let shared_locator = Rc::new(geolocator);
    cx.provide_context(shared_locator.clone());
    Ok(shared_locator)
}
