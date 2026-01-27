//! AMAIL-EMBED - Embeddable Leptos Components
//!
//! Table of Contents:
//! 1. Inbox Widget - Mini inbox for external sites
//! 2. Contact Form - AI-powered contact form
//! 3. Status Badge - Server status indicator
//! 4. Configuration - Widget setup and theming
//!
//! Usage in external Leptos apps:
//! ```rust
//! use amail_embed::{AmailInbox, AmailContactForm, AmailStatusBadge};
//! 
//! view! {
//!     <AmailInbox api_url="https://mail.example.com" mailbox="support" />
//!     <AmailContactForm api_url="https://mail.example.com" to="contact@example.com" />
//!     <AmailStatusBadge api_url="https://mail.example.com" />
//! }
//! ```

pub mod config;
pub mod widgets;
pub mod styles;

pub use config::{AmailConfig, ThemeConfig};
pub use widgets::inbox::AmailInbox;
pub use widgets::contact_form::AmailContactForm;
pub use widgets::status_badge::AmailStatusBadge;

use wasm_bindgen::prelude::*;

/// Initialize the Amail embed library
#[wasm_bindgen(start)]
pub fn init() {
    // Set panic hook for better error messages in WASM
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

/// Mount an inbox widget to a DOM element by ID
#[wasm_bindgen]
pub fn mount_inbox(element_id: &str, api_url: &str, mailbox: &str) {
    use leptos::*;
    
    let element = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.get_element_by_id(element_id))
        .expect("Could not find element");
    
    let api = api_url.to_string();
    let mb = mailbox.to_string();
    
    mount_to(element.unchecked_into(), move || {
        view! { <AmailInbox api_url=api.clone() mailbox=mb.clone() /> }
    });
}

/// Mount a contact form to a DOM element by ID
#[wasm_bindgen]
pub fn mount_contact_form(element_id: &str, api_url: &str, to_email: &str) {
    use leptos::*;
    
    let element = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.get_element_by_id(element_id))
        .expect("Could not find element");
    
    let api = api_url.to_string();
    let to = to_email.to_string();
    
    mount_to(element.unchecked_into(), move || {
        view! { <AmailContactForm api_url=api.clone() to=to.clone() /> }
    });
}

/// Mount a status badge to a DOM element by ID
#[wasm_bindgen]
pub fn mount_status_badge(element_id: &str, api_url: &str) {
    use leptos::*;
    
    let element = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.get_element_by_id(element_id))
        .expect("Could not find element");
    
    let api = api_url.to_string();
    
    mount_to(element.unchecked_into(), move || {
        view! { <AmailStatusBadge api_url=api.clone() /> }
    });
}
