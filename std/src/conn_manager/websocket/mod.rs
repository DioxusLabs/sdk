cfg_if::cfg_if! {
    if #[cfg(feature = "conn_mgr_ws")] {
        pub mod client;
        pub use client::*;
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature = "conn_mgr_ws_axum")] {
        pub mod axum;
        pub use axum::*;
    }
}
