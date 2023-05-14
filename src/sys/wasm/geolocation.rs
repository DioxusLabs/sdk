use std::sync::{Arc, Condvar, Mutex};

use wasm_bindgen::{prelude::Closure, JsCast, JsValue};
use web_sys::PositionOptions;

use crate::library::geolocation::{DeviceGeolocator, Error, Event, Geocoordinates, PowerMode};

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

impl DeviceGeolocator for Geolocator {
    /// Get the latest available coordinates.
    fn get_coordinates(&self) -> Result<Geocoordinates, Error> {
        todo!("this function is not currently available on web");

        // Create condvar
        // Option<Result<Geocoordinates, Error>>
        let pair = Arc::new((
            Mutex::new(None::<Result<Geocoordinates, Error>>),
            Condvar::new(),
        ));

        let pair2 = Arc::clone(&pair);
        let pair3 = Arc::clone(&pair);

        let success = Closure::wrap(Box::new(move |pos| {
            // Get condvar
            let (lock, cvar) = &*pair;
            let mut result = lock.lock().unwrap();

            // Lots of casting with soft error handling
            let coords = match js_sys::Reflect::get(&pos, &JsValue::from_str("coords")) {
                Ok(v) => v,
                Err(_) => {
                    *result = Some(Err(Error::DeviceError("failed cast".to_string())));
                    cvar.notify_one();
                    return;
                }
            };

            let latitude = match js_sys::Reflect::get(&coords, &JsValue::from_str("latitude")) {
                Ok(v) => v,
                Err(_) => {
                    *result = Some(Err(Error::DeviceError("failed cast".to_string())));
                    cvar.notify_one();
                    return;
                }
            };

            let latitude = match latitude.as_f64() {
                Some(v) => v,
                None => {
                    *result = Some(Err(Error::DeviceError("failed cast".to_string())));
                    cvar.notify_one();
                    return;
                }
            };

            let longitude = match js_sys::Reflect::get(&coords, &JsValue::from_str("longitude")) {
                Ok(v) => v,
                Err(_) => {
                    *result = Some(Err(Error::DeviceError("failed cast".to_string())));
                    cvar.notify_one();
                    return;
                }
            };

            let longitude = match longitude.as_f64() {
                Some(v) => v,
                None => {
                    *result = Some(Err(Error::DeviceError("failed cast".to_string())));
                    cvar.notify_one();
                    return;
                }
            };
            // End casting

            let geocoords = Geocoordinates {
                latitude,
                longitude,
            };

            *result = Some(Ok(geocoords));
            cvar.notify_one();
        }) as Box<dyn Fn(JsValue)>);
        let error = Closure::wrap(Box::new(move |e| {
            let (lock, cvar) = &*pair2;
            let mut result = lock.lock().unwrap();

            let message = match js_sys::Reflect::get(&e, &JsValue::from_str("message")) {
                Ok(v) => v,
                Err(_) => {
                    *result = Some(Err(Error::DeviceError("failed cast".to_string())));
                    cvar.notify_one();
                    return;
                }
            };

            let message = match message.as_string() {
                Some(v) => v,
                None => {
                    *result = Some(Err(Error::DeviceError("failed cast".to_string())));
                    cvar.notify_one();
                    return;
                }
            };

            *result = Some(Err(Error::DeviceError(message)));
            cvar.notify_one();
        }) as Box<dyn Fn(JsValue)>);

        // Get position
        self.device_geolocator
            .get_current_position_with_error_callback_and_options(
                success.as_ref().unchecked_ref(),
                Some(error.as_ref().unchecked_ref()),
                &self.options,
            )
            .map_err(|e| Error::DeviceError(format!("{:?}", e)))?;

        // Wait for data to be available
        let (lock, cvar) = &*pair3;
        let mut result = lock.lock().map_err(|e| Error::DeviceError(e.to_string()))?;

        // Spurious wakeup prevention
        while result.is_none() {
            result = cvar
                .wait(result)
                .map_err(|e| Error::DeviceError(e.to_string()))?;
        }

        // At this point, result should be Some(_)
        result
            .take()
            .unwrap()
            .map_err(|_| Error::DeviceError("an option was none when it shouldn't be".to_string()))
    }

    /// Listen to new events with a callback.
    fn listen(&self, callback: Arc<dyn Fn(Event) + Send + Sync>) -> Result<(), Error> {
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
        self.device_geolocator
            .watch_position_with_error_callback_and_options(
                success.as_ref().unchecked_ref(),
                None,
                &self.options,
            )
            .map_err(|e| Error::DeviceError(format!("{:?}", e)))?;
        
        // Prevent from being dropped.
        success.forget();
        Ok(())
    }

    /// Set the device's power mode.
    fn set_power_mode(&mut self, power_mode: PowerMode) -> Result<(), Error> {
        match power_mode {
            PowerMode::High => self.options.enable_high_accuracy(true),
            PowerMode::Low => self.options.enable_high_accuracy(false),
        };

        Ok(())
    }
}
