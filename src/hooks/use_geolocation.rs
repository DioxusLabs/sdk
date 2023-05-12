use std::{rc::Rc, sync::Once};

use crate::library::geolocation::{DeviceStatus, Geocoordinates, GeolocationError, Geolocator};
use dioxus::prelude::ScopeState;

use super::{use_rw, UseRw};

static INIT: Once = Once::new();

pub fn use_geolocation(cx: &ScopeState) -> Result<Geocoordinates, GeolocationError> {
    let current_result: &mut UseRw<Result<Geocoordinates, GeolocationError>> =
        use_rw(cx, || Ok(Geocoordinates::empty()));

    let geolocator = match cx.consume_context::<Rc<Geolocator>>() {
        Some(v) => v,
        None => {
            return Err(GeolocationError::FailedToFetchCoordinates(
                "geolocator not initialized".to_string(),
            ))
        }
    };

    let result1 = current_result.clone();
    let result2 = current_result.clone();

    INIT.call_once(|| {
        let _ = geolocator.on_position_changed(move |coords: Geocoordinates| {
            let _ = result1.write(Ok(coords));
        });

        let _ = geolocator.on_status_changed(move |status: DeviceStatus| {
            if status == DeviceStatus::Disabled {
                let _ = result2.write(Err(GeolocationError::AccessDenied));
            }
        });
    });

    let result = current_result
        .read()
        .map_err(|_| GeolocationError::FailedToFetchCoordinates("rw is poisioned".to_string()))?
        .clone();
    result
}

pub fn init_geolocator(
    cx: &ScopeState,
    report_interval: Option<u32>,
    movement_threshold: Option<u32>,
) -> Result<Rc<Geolocator>, GeolocationError> {
    let geolocator = Geolocator::new(report_interval, movement_threshold)?;
    let shared_locator = Rc::new(geolocator);
    cx.provide_context(shared_locator.clone());
    Ok(shared_locator)
}
