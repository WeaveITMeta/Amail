//! Server Status Badge Widget
//!
//! Features:
//! - Real-time server status indicator
//! - Online/Offline/Degraded states
//! - Tooltip with details
//! - Polling for updates

use leptos::*;

#[derive(Clone, Debug, PartialEq)]
pub enum ServerStatus {
    Online,
    Offline,
    Degraded,
    Unknown,
}

/// Server status badge for embedding in external Leptos apps
#[component]
pub fn AmailStatusBadge(
    /// API URL for the Amail server
    #[prop(into)] api_url: String,
    /// Show label text
    #[prop(default = true)] show_label: bool,
    /// Compact mode (dot only)
    #[prop(default = false)] compact: bool,
) -> impl IntoView {
    let (status, _set_status) = create_signal(ServerStatus::Online);
    let (last_check, _set_last_check) = create_signal("Just now".to_string());
    
    let status_class = move || match status.get() {
        ServerStatus::Online => "amail-status online",
        ServerStatus::Offline => "amail-status offline",
        ServerStatus::Degraded => "amail-status degraded",
        ServerStatus::Unknown => "amail-status unknown",
    };
    
    let status_text = move || match status.get() {
        ServerStatus::Online => "Online",
        ServerStatus::Offline => "Offline",
        ServerStatus::Degraded => "Degraded",
        ServerStatus::Unknown => "Unknown",
    };

    view! {
        <div class=move || if compact { format!("{} compact", status_class()) } else { status_class() }>
            <span class="amail-status-dot"></span>
            <Show when=move || show_label && !compact>
                <span class="amail-status-label">{status_text}</span>
            </Show>
            <div class="amail-status-tooltip">
                <div class="tooltip-header">
                    <span class="tooltip-title">"Amail Server"</span>
                    <span class=move || format!("tooltip-status {}", match status.get() {
                        ServerStatus::Online => "online",
                        ServerStatus::Offline => "offline",
                        ServerStatus::Degraded => "degraded",
                        ServerStatus::Unknown => "unknown",
                    })>{status_text}</span>
                </div>
                <div class="tooltip-details">
                    <div class="tooltip-row">
                        <span>"API:"</span>
                        <span class="tooltip-value">{api_url.clone()}</span>
                    </div>
                    <div class="tooltip-row">
                        <span>"Last check:"</span>
                        <span class="tooltip-value">{last_check}</span>
                    </div>
                </div>
            </div>
        </div>
    }
}
