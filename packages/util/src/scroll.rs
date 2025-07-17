use dioxus::{core::use_drop, prelude::*};
use std::sync::atomic::{AtomicUsize, Ordering};

/// Scroll metrics.
#[derive(serde::Deserialize, Clone, Debug)]
pub struct ScrollMetrics {
    /// Current scroll position from top: https://developer.mozilla.org/en-US/docs/Web/API/Element/scrollTop
    pub scroll_top: f64,
    /// Current scroll position from left: https://developer.mozilla.org/en-US/docs/Web/API/Element/scrollLeft
    pub scroll_left: f64,

    /// Viewport height: https://developer.mozilla.org/en-US/docs/Web/API/Element/clientHeight
    pub client_height: f64,
    /// Viewport width: https://developer.mozilla.org/en-US/docs/Web/API/Element/clientWidth
    pub client_width: f64,

    /// Content height: https://developer.mozilla.org/en-US/docs/Web/API/Element/scrollHeight
    pub scroll_height: f64,
    /// Content width: https://developer.mozilla.org/en-US/docs/Web/API/Element/scrollWidth
    pub scroll_width: f64,
}

// Static counter to generate unique IDs for each scroll tracker instance
static SCROLL_TRACKER_COUNTER: AtomicUsize = AtomicUsize::new(0);

/// Creates a signal that tracks root scrolling.
/// ```rust
/// use dioxus::{logger::tracing::{info, Level}, prelude::*};
/// use dioxus_util::scroll::use_root_scroll;
///
/// #[component]
/// fn App() -> Element {
///     let random_text = "This is some random repeating text. ".repeat(1000);
///     
///     let scroll_metrics = use_root_scroll();
///     use_effect(move || {
///         let scroll_metrics = scroll_metrics();
///         let distance_from_bottom = scroll_metrics.scroll_height - (scroll_metrics.scroll_top + scroll_metrics.client_height);
///         info!("Distance from bottom: {}", distance_from_bottom);
///         let scroll_percentage = (scroll_metrics.scroll_top + scroll_metrics.client_height) / scroll_metrics.scroll_height;
///         info!("Scroll percentage: {}", scroll_percentage);
///     });
///  
///     rsx! {
///         p { "{random_text}" }  
///     }
/// }
/// ```
pub fn use_root_scroll() -> Signal<ScrollMetrics> {
    let callback_name = use_hook(|| {
        let instance_id = SCROLL_TRACKER_COUNTER.fetch_add(1, Ordering::SeqCst);
        format!("scrollCallback_{}", instance_id)
    });

    let mut scroll_metrics = use_signal(|| ScrollMetrics {
        scroll_top: 0.0,
        scroll_left: 0.0,
        client_height: 0.0,
        client_width: 0.0,
        scroll_height: 0.0,
        scroll_width: 0.0,
    });

    use_future({
        to_owned![callback_name];
        move || {
            to_owned![callback_name];
            async move {
                let js_code = format!(
                    r#"
                function {callback_name}() {{
                    const doc = document.documentElement;
                    const scrollTop = window.scrollY || doc.scrollTop;
                    const scrollLeft = window.scrollX || doc.scrollLeft;
                    const viewportHeight = window.innerHeight;
                    const viewportWidth = window.innerWidth;
                    const contentHeight = doc.scrollHeight;
                    const contentWidth = doc.scrollWidth;

                    dioxus.send({{
                        scroll_top: scrollTop,
                        scroll_left: scrollLeft,
                        client_height: viewportHeight,
                        client_width: viewportWidth,
                        scroll_height: contentHeight,
                        scroll_width: contentWidth,
                    }});
                }}

                {callback_name}();

                window['{callback_name}'] = {callback_name};
                window.addEventListener('scroll', window['{callback_name}']);
                window.addEventListener('resize', window['{callback_name}']);
                "#,
                );

                let mut eval = document::eval(&js_code);

                loop {
                    match eval.recv::<ScrollMetrics>().await {
                        Ok(metrics) => {
                            dioxus::logger::tracing::trace!("Got scroll metrics {:?}", metrics);
                            scroll_metrics.set(metrics);
                        }
                        Err(error) => dioxus::logger::tracing::error!(
                            "Error receiving scroll metrics: {:?}",
                            error
                        ),
                    }
                }
            }
        }
    });

    use_drop(move || {
        let cleanup_code = format!(
            r#"
            window.removeEventListener('scroll', window['{callback_name}']);
            window.removeEventListener('resize', window['{callback_name}']);
            delete window['{callback_name}'];
            "#,
        );
        let _ = document::eval(&cleanup_code);
    });

    scroll_metrics
}
