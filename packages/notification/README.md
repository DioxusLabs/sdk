# Dioxus Notification
Send notifications from your Dioxus apps.

### Supports
- [x] Windows
- [x] Mac
- [x] Linux
- [ ] Android
- [ ] iOs

## Usage
Add `dioxus-sdk-notification` to your `Cargo.toml`:
```toml
[dependencies]
dioxus-sdk-notification = "0.1"
```

Example:
```rs
use dioxus_sdk_notification::Notification;

Notification::new()
    .app_name("dioxus test".to_string())
    .summary("hi, this is dioxus test".to_string())
    .body("lorem ipsum".to_string())
    .show()
    .unwrap();
```
