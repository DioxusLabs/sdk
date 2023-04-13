<div align="center">
  <h1>🧰 Dioxus Standard Library 🚀</h1>
  <p><strong>A platform agnostic library for supercharging your productivity with Dioxus.</strong></p>
</div>

-----
<p align="center"><i>This library is still under development and not ready for production use.</i></p>
<br/>

`dioxus-std` is a Dioxus standard library that provides abstractions for your Dioxus app. Abstractions included are notifications, clipboard, and more to come.

**Current & Planned Features**
- [x] Clipboard
- [x] Notifications
- [x] Utility Hooks - (use_prefererred_color_scheme: web only)
- [ ] Camera - In Progress
- [x] Geolocation (GPS)
- [ ] Mobile File System
- [ ] WiFi
- [ ] Bluetooth

**This library is current under development. Expect breaking changes.**

```rust, ignore
fn app() {
    // TODO: Add example
}
```

## Platform Support
Currently `dioxus-std` primarily supports desktop targets. It is planned to support all of Dioxus' targets in the future.

- [x] Desktop (Windows, MacOS, Linux)
- [ ] Mobile  (Android, iOS)
- [ ] Web     (WASM)

On linux you need the x11 library to use the clipboard abstraction:
```
sudo apt-get install xorg-dev
```

## Installation
You can add `dioxus-std` to your application by adding it to your dependencies.
```toml
[dependencies]
dioxus-std =  { git="https://github.com/DioxusLabs/dioxus-std" }
```

## License
This project is licensed under the [MIT license].

[mit license]: ./LICENSE

Every contribution intentionally submitted for inclusion in `dioxus-std` by you, shall be licensed as MIT, without any additional
terms or conditions.
