# Dioxus Notification
Send notifications from your Dioxus apps.

### Supports
- [x] Windows
- [x] Mac
- [x] Linux
- [ ] Android
- [ ] iOs

## Usage
Add `dioxus-notification` to your `Cargo.toml`:
```toml
[dependencies]
dioxus-notification = "0.1"
```

Example:
```rs
use dioxus_notification::Notification;

Notification::new()
    .app_name("dioxus test".to_string())
    .summary("hi, this is dioxus test".to_string())
    .body("lorem ipsum".to_string())
    .show()
    .unwrap();
```



### Dioxus Compatibility
This table represents the compatibility between this crate and Dioxus versions.
The crate version supports a Dioxus version up until the next crate version in the table.

E.g. if crate version `0.1` supported Dioxus `0.6` and crate version `0.4` supported Dioxus `0.7`, crate versions `0.1`, `0.2`, and `0.3` would support Dioxus `0.6`.

| Crate Version | Dioxus Version |
| ------------- | -------------- |
| 0.1           | 0.6            |