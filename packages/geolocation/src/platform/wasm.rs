use futures::channel::mpsc;
use futures_util::StreamExt;
use std::sync::Arc;
use wasm_bindgen::{JsCast, JsValue, prelude::Closure};
use web_sys::PositionOptions;

use crate::{Error, Event, Geocoordinates, PowerMode};

/// Represents the HAL's geolocator.
pub struct Geolocator {
    device_geolocator: web_sys::Geolocation,
    options: PositionOptions,
}

impl Geolocator {
    /// Create a new Geolocator for the device.
    pub fn new() -> Result<Self, Error> {
        let window = web_sys::window().expect("not a wasm context");
        let navigator = window.navigator();
        let locator = navigator
            .geolocation()
            .map_err(|e| Error::DeviceError(format!("{:?}", e)))?;

        let options = PositionOptions::new();

        Ok(Self {
            device_geolocator: locator,
            options,
        })
    }
}

pub async fn get_coordinates(geolocator: &Geolocator) -> Result<Geocoordinates, Error> {
    // Start channel
    let (mut sender, mut receiver) = mpsc::channel::<Result<Geocoordinates, Error>>(1);
    let mut sender1 = sender.clone();

    // Success
    let success = Closure::wrap(Box::new(move |pos| {
        // Lots of casting with soft error handling
        let coords = match js_sys::Reflect::get(&pos, &JsValue::from_str("coords")) {
            Ok(v) => v,
            Err(_) => {
                let _ = sender.try_send(Err(Error::DeviceError("failed cast".to_string())));
                return;
            }
        };

        let latitude = match js_sys::Reflect::get(&coords, &JsValue::from_str("latitude")) {
            Ok(v) => v,
            Err(_) => {
                let _ = sender.try_send(Err(Error::DeviceError("failed cast".to_string())));
                return;
            }
        };

        let latitude = match latitude.as_f64() {
            Some(v) => v,
            None => {
                let _ = sender.try_send(Err(Error::DeviceError("failed cast".to_string())));
                return;
            }
        };

        let longitude = match js_sys::Reflect::get(&coords, &JsValue::from_str("longitude")) {
            Ok(v) => v,
            Err(_) => {
                let _ = sender.try_send(Err(Error::DeviceError("failed cast".to_string())));
                return;
            }
        };

        let longitude = match longitude.as_f64() {
            Some(v) => v,
            None => {
                let _ = sender.try_send(Err(Error::DeviceError("failed cast".to_string())));
                return;
            }
        };
        // End casting

        let geocoords = Geocoordinates {
            latitude,
            longitude,
        };

        let _ = sender.try_send(Ok(geocoords));
    }) as Box<dyn FnMut(JsValue)>);

    // Error
    let error = Closure::wrap(Box::new(move |e| {
        let message = match js_sys::Reflect::get(&e, &JsValue::from_str("message")) {
            Ok(v) => v,
            Err(_) => {
                let _ = sender1.try_send(Err(Error::DeviceError("failed cast".to_string())));
                return;
            }
        };

        let message = match message.as_string() {
            Some(v) => v,
            None => {
                let _ = sender1.try_send(Err(Error::DeviceError("failed cast".to_string())));
                return;
            }
        };

        let _ = sender1.try_send(Err(Error::DeviceError(message)));
    }) as Box<dyn FnMut(JsValue)>);

    // Get position
    geolocator
        .device_geolocator
        .get_current_position_with_error_callback_and_options(
            success.as_ref().unchecked_ref(),
            Some(error.as_ref().unchecked_ref()),
            &geolocator.options,
        )
        .map_err(|e| Error::DeviceError(format!("{:?}", e)))?;

    if let Some(msg) = receiver.next().await {
        receiver.close();
        return msg;
    }

    Err(Error::DeviceError("async communication failed".to_string()))
}

/// Listen to new events with a callback.
pub fn listen(
    geolocator: &Geolocator,
    callback: Arc<dyn Fn(Event) + Send + Sync>,
) -> Result<(), Error> {
    let success = Closure::wrap(Box::new(move |pos| {
        // Lots of casting with soft error handling
        let coords = match js_sys::Reflect::get(&pos, &JsValue::from_str("coords")) {
            Ok(v) => v,
            Err(_) => return,
        };

        let latitude = match js_sys::Reflect::get(&coords, &JsValue::from_str("latitude")) {
            Ok(v) => v,
            Err(_) => return,
        };

        let latitude = match latitude.as_f64() {
            Some(v) => v,
            None => return,
        };

        let longitude = match js_sys::Reflect::get(&coords, &JsValue::from_str("longitude")) {
            Ok(v) => v,
            Err(_) => return,
        };

        let longitude = match longitude.as_f64() {
            Some(v) => v,
            None => return,
        };
        // End casting

        let geocoords = Geocoordinates {
            latitude,
            longitude,
        };

        (callback)(Event::NewGeocoordinates(geocoords))
    }) as Box<dyn Fn(JsValue)>);

    // Subscribe
    geolocator
        .device_geolocator
        .watch_position_with_error_callback_and_options(
            success.as_ref().unchecked_ref(),
            None,
            &geolocator.options,
        )
        .map_err(|e| Error::DeviceError(format!("{:?}", e)))?;

    // Prevent from being dropped.
    success.forget();
    Ok(())
}

/// Set the device's power mode.
pub fn set_power_mode(geolocator: &mut Geolocator, power_mode: PowerMode) -> Result<(), Error> {
    let value = match power_mode {
        PowerMode::High => true,
        PowerMode::Low => false,
    };

    geolocator.options.set_enable_high_accuracy(value);

    Ok(())
}
