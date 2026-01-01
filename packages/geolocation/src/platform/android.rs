use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use jni::JNIEnv;
use jni::objects::{JObject, JValue};

use crate::core::{Error, Event, Geocoordinates, PowerMode, Status};

const FINE_LOCATION_PERMISSION: &str = "android.permission.ACCESS_FINE_LOCATION";
const COARSE_LOCATION_PERMISSION: &str = "android.permission.ACCESS_COARSE_LOCATION";

fn permission_for_power_mode(power_mode: PowerMode) -> &'static str {
    match power_mode {
        PowerMode::High => FINE_LOCATION_PERMISSION,
        PowerMode::Low => COARSE_LOCATION_PERMISSION,
    }
}

/// Represents the geolocator for Android.
pub struct Geolocator {
    power_mode: PowerMode,
    stop_listening: Arc<AtomicBool>,
}

impl Geolocator {
    /// Create a new Geolocator for the device.
    /// This will request location permissions if not already granted and wait for user response.
    pub fn new() -> Result<Self, Error> {
        use std::sync::mpsc::channel;

        let permission = FINE_LOCATION_PERMISSION;
        let already_granted = check_location_permission(permission)?;

        if !already_granted {
            let (tx, rx) = channel();

            dioxus::mobile::wry::prelude::dispatch(
                move |env: &mut JNIEnv, activity: &JObject, _webview| {
                    let permission_str = env.new_string(permission).unwrap();
                    let permissions = env
                        .new_object_array(1, "java/lang/String", &permission_str)
                        .unwrap();

                    let result = env.call_method(
                        activity,
                        "requestPermissions",
                        "([Ljava/lang/String;I)V",
                        &[JValue::Object(&permissions.into()), JValue::Int(1)],
                    );

                    tx.send(result.is_ok()).unwrap();
                },
            );

            rx.recv().map_err(|e| Error::DeviceError(e.to_string()))?;

            // Poll for permission result with timeout
            // The permission dialog is shown asynchronously, so we poll until
            // the user responds or we timeout (30 seconds)
            let poll_interval = Duration::from_millis(250);
            let timeout = Duration::from_secs(30);
            let start = std::time::Instant::now();

            loop {
                thread::sleep(poll_interval);

                match check_location_permission(permission) {
                    Ok(true) => break,
                    Ok(false) if start.elapsed() >= timeout => {
                        return Err(Error::AccessDenied);
                    }
                    Ok(false) => continue,
                    Err(e) => return Err(e),
                }
            }
        }

        Ok(Self {
            power_mode: PowerMode::High,
            stop_listening: Arc::new(AtomicBool::new(false)),
        })
    }
}

fn check_location_permission(permission: &'static str) -> Result<bool, Error> {
    use std::sync::mpsc::channel;

    let (tx, rx) = channel();

    dioxus::mobile::wry::prelude::dispatch(
        move |env: &mut JNIEnv, activity: &JObject, _webview| {
            let permission_str = env.new_string(permission).unwrap();

            let check_result = env
                .call_method(
                    activity,
                    "checkSelfPermission",
                    "(Ljava/lang/String;)I",
                    &[JValue::Object(&permission_str.into())],
                )
                .and_then(|v| v.i());

            match check_result {
                Ok(0) => tx.send(Ok(true)).unwrap(),
                Ok(_) => tx.send(Ok(false)).unwrap(),
                Err(e) => tx.send(Err(Error::DeviceError(e.to_string()))).unwrap(),
            }
        },
    );

    rx.recv().map_err(|e| Error::DeviceError(e.to_string()))?
}

pub async fn get_coordinates(geolocator: &Geolocator) -> Result<Geocoordinates, Error> {
    use std::sync::mpsc::channel;

    let power_mode = geolocator.power_mode;
    let permission = permission_for_power_mode(power_mode);

    if !check_location_permission(permission)? {
        return Err(Error::AccessDenied);
    }
    let (tx, rx) = channel();

    dioxus::mobile::wry::prelude::dispatch(
        move |env: &mut JNIEnv, activity: &JObject, _webview| {
            // Get LocationManager
            let location_service = env.new_string("location").unwrap();
            let location_manager = env
                .call_method(
                    activity,
                    "getSystemService",
                    "(Ljava/lang/String;)Ljava/lang/Object;",
                    &[JValue::Object(&location_service.into())],
                )
                .and_then(|v| v.l());

            let location_manager = match location_manager {
                Ok(lm) => lm,
                Err(e) => {
                    tx.send(Err(Error::DeviceError(e.to_string()))).unwrap();
                    return;
                }
            };

            // Determine provider based on power mode
            let provider = match power_mode {
                PowerMode::High => "gps",
                PowerMode::Low => "network",
            };
            let provider_str = env.new_string(provider).unwrap();

            // Get last known location
            let location = env
                .call_method(
                    &location_manager,
                    "getLastKnownLocation",
                    "(Ljava/lang/String;)Landroid/location/Location;",
                    &[JValue::Object(&provider_str.into())],
                )
                .and_then(|v| v.l());

            match location {
                Ok(loc) if !loc.is_null() => {
                    let latitude = env
                        .call_method(&loc, "getLatitude", "()D", &[])
                        .and_then(|v| v.d())
                        .unwrap_or(0.0);

                    let longitude = env
                        .call_method(&loc, "getLongitude", "()D", &[])
                        .and_then(|v| v.d())
                        .unwrap_or(0.0);

                    tx.send(Ok(Geocoordinates {
                        latitude,
                        longitude,
                    }))
                    .unwrap();
                }
                Ok(_) => {
                    tx.send(Err(Error::DeviceError(
                        "No last known location available".to_string(),
                    )))
                    .unwrap();
                }
                Err(e) => {
                    tx.send(Err(Error::DeviceError(e.to_string()))).unwrap();
                }
            }
        },
    );

    rx.recv().map_err(|e| Error::DeviceError(e.to_string()))?
}

/// This spawns a background thread that continuously polls for location updates.
pub fn listen(
    geolocator: &Geolocator,
    callback: Arc<dyn Fn(Event) + Send + Sync>,
) -> Result<(), Error> {
    use std::sync::Mutex;
    use std::sync::mpsc::channel;

    let power_mode = geolocator.power_mode;
    let permission = permission_for_power_mode(power_mode);

    if !check_location_permission(permission)? {
        return Err(Error::AccessDenied);
    }
    let stop_flag = geolocator.stop_listening.clone();

    stop_flag.store(false, Ordering::SeqCst);

    let (tx, rx) = channel();
    let callback_init = callback.clone();

    dioxus::mobile::wry::prelude::dispatch(
        move |env: &mut JNIEnv, activity: &JObject, _webview| {
            // Get LocationManager
            let location_service = env.new_string("location").unwrap();
            let location_manager = env
                .call_method(
                    activity,
                    "getSystemService",
                    "(Ljava/lang/String;)Ljava/lang/Object;",
                    &[JValue::Object(&location_service.into())],
                )
                .and_then(|v| v.l());

            let location_manager = match location_manager {
                Ok(lm) => lm,
                Err(e) => {
                    tx.send(Err(Error::DeviceError(e.to_string()))).unwrap();
                    return;
                }
            };

            let provider = match power_mode {
                PowerMode::High => "gps",
                PowerMode::Low => "network",
            };
            let provider_str = env.new_string(provider).unwrap();

            let is_enabled = env
                .call_method(
                    &location_manager,
                    "isProviderEnabled",
                    "(Ljava/lang/String;)Z",
                    &[JValue::Object(&provider_str.into())],
                )
                .and_then(|v| v.z())
                .unwrap_or(false);

            if is_enabled {
                callback_init(Event::StatusChanged(Status::Ready));
            } else {
                callback_init(Event::StatusChanged(Status::Disabled));
            }

            // Get initial location
            let provider_str = env.new_string(provider).unwrap();
            let location = env
                .call_method(
                    &location_manager,
                    "getLastKnownLocation",
                    "(Ljava/lang/String;)Landroid/location/Location;",
                    &[JValue::Object(&provider_str.into())],
                )
                .and_then(|v| v.l());

            let initial_coords = if let Ok(loc) = location {
                if !loc.is_null() {
                    let latitude = env
                        .call_method(&loc, "getLatitude", "()D", &[])
                        .and_then(|v| v.d())
                        .unwrap_or(0.0);

                    let longitude = env
                        .call_method(&loc, "getLongitude", "()D", &[])
                        .and_then(|v| v.d())
                        .unwrap_or(0.0);

                    Some(Geocoordinates {
                        latitude,
                        longitude,
                    })
                } else {
                    None
                }
            } else {
                None
            };

            if let Some(coords) = initial_coords.clone() {
                callback_init(Event::NewGeocoordinates(coords));
            }

            tx.send(Ok(initial_coords)).unwrap();
        },
    );

    let initial_coords = rx.recv().map_err(|e| Error::DeviceError(e.to_string()))??;

    // Spawn a background thread for continuous updates
    let last_coords: Arc<Mutex<Option<Geocoordinates>>> = Arc::new(Mutex::new(initial_coords));

    thread::spawn(move || {
        // Polling interval: 1 second for High accuracy, 5 seconds for Low
        let poll_interval = match power_mode {
            PowerMode::High => Duration::from_secs(1),
            PowerMode::Low => Duration::from_secs(5),
        };

        loop {
            if stop_flag.load(Ordering::SeqCst) {
                break;
            }

            thread::sleep(poll_interval);

            if stop_flag.load(Ordering::SeqCst) {
                break;
            }

            let (tx, rx) = channel();
            let callback_poll = callback.clone();
            let last_coords_poll = last_coords.clone();

            let permission_poll = permission_for_power_mode(power_mode);
            dioxus::mobile::wry::prelude::dispatch(
                move |env: &mut JNIEnv, activity: &JObject, _webview| {
                    // Check if permission is still granted
                    let permission_str = env.new_string(permission_poll).unwrap();

                    let has_permission = env
                        .call_method(
                            activity,
                            "checkSelfPermission",
                            "(Ljava/lang/String;)I",
                            &[JValue::Object(&permission_str.into())],
                        )
                        .and_then(|v| v.i())
                        .map(|v| v == 0)
                        .unwrap_or(false);

                    if !has_permission {
                        callback_poll(Event::StatusChanged(Status::NotAvailable));
                        tx.send(false).unwrap();
                        return;
                    }

                    let location_service = env.new_string("location").unwrap();
                    let location_manager = env
                        .call_method(
                            activity,
                            "getSystemService",
                            "(Ljava/lang/String;)Ljava/lang/Object;",
                            &[JValue::Object(&location_service.into())],
                        )
                        .and_then(|v| v.l());

                    let location_manager = match location_manager {
                        Ok(lm) => lm,
                        Err(_) => {
                            tx.send(true).unwrap();
                            return;
                        }
                    };

                    let provider = match power_mode {
                        PowerMode::High => "gps",
                        PowerMode::Low => "network",
                    };
                    let provider_str = env.new_string(provider).unwrap();

                    let location = env
                        .call_method(
                            &location_manager,
                            "getLastKnownLocation",
                            "(Ljava/lang/String;)Landroid/location/Location;",
                            &[JValue::Object(&provider_str.into())],
                        )
                        .and_then(|v| v.l());

                    if let Ok(loc) = location {
                        if !loc.is_null() {
                            let latitude = env
                                .call_method(&loc, "getLatitude", "()D", &[])
                                .and_then(|v| v.d())
                                .unwrap_or(0.0);

                            let longitude = env
                                .call_method(&loc, "getLongitude", "()D", &[])
                                .and_then(|v| v.d())
                                .unwrap_or(0.0);

                            let new_coords = Geocoordinates {
                                latitude,
                                longitude,
                            };

                            let mut last = last_coords_poll.lock().unwrap();
                            let should_notify = match &*last {
                                Some(prev) => {
                                    (prev.latitude - latitude).abs() > 0.00001
                                        || (prev.longitude - longitude).abs() > 0.00001
                                }
                                None => true,
                            };

                            if should_notify {
                                *last = Some(new_coords.clone());
                                callback_poll(Event::NewGeocoordinates(new_coords));
                            }
                        }
                    }

                    tx.send(true).unwrap();
                },
            );

            // Wait for the dispatch to complete before next iteration
            // If permission was revoked, stop the loop
            if let Ok(false) = rx.recv() {
                break;
            }
        }
    });

    Ok(())
}

pub fn set_power_mode(geolocator: &mut Geolocator, power_mode: PowerMode) -> Result<(), Error> {
    geolocator.power_mode = power_mode;
    Ok(())
}
