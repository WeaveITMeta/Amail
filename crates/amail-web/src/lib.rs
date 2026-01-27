// =============================================================================
// AMAIL-WEB - Leptos Admin UI
// =============================================================================
// Table of Contents:
// 1. Module declarations
// 2. App component with leptos_router
// 3. WebSocket connection
// 4. Layout and pages
// =============================================================================

pub mod components;
pub mod state;
pub mod websocket;
pub mod layout;
pub mod pages;
pub mod api;

use leptos::*;
use leptos_router::*;
use wasm_bindgen::JsCast;
use state::DashboardState;
use websocket::use_websocket;
use layout::AppLayout;
use pages::{DashboardPage, DomainsPage, DnsWizardPage, AgentsPage, SettingsPage, InboxPage, TenantsPage, ReviewsPage, AnalyticsPage, DocsPage, WorkspaceRulesPage};

/// Main application entry point
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn main() {
    // Initialize panic hook for better error messages
    console_error_panic_hook::set_once();
    
    // Initialize logging
    console_log::init_with_level(log::Level::Debug).expect("Failed to initialize logger");
    
    log::info!("Amail Admin UI starting...");
    
    // Mount the app to the #app div
    let app_element = document()
        .get_element_by_id("app")
        .expect("Could not find #app element")
        .unchecked_into::<web_sys::HtmlElement>();
    mount_to(app_element, || view! { <App/> });
}

/// Root application component with proper URL-based routing
#[component]
pub fn App() -> impl IntoView {
    // Create dashboard state
    let state = DashboardState::new();
    provide_context(state.clone());
    
    // Setup WebSocket connection
    let ws_url = "ws://localhost:4000/ws/dashboard";
    let (_connected, _messages) = use_websocket(ws_url, state.clone());
    
    view! {
        <Router>
            <AppLayout>
                <Routes>
                    <Route path="/" view=DashboardPage/>
                    <Route path="/inbox" view=InboxPage/>
                    <Route path="/reviews" view=ReviewsPage/>
                    <Route path="/analytics" view=AnalyticsPage/>
                    <Route path="/tenants" view=TenantsPage/>
                    <Route path="/domains" view=DomainsPage/>
                    <Route path="/dns-wizard" view=DnsWizardPage/>
                    <Route path="/dns-wizard/:domain_id" view=DnsWizardPage/>
                    <Route path="/agents" view=AgentsPage/>
                    <Route path="/settings" view=SettingsPage/>
                    <Route path="/docs" view=DocsPage/>
                    <Route path="/workspace-rules" view=WorkspaceRulesPage/>
                </Routes>
            </AppLayout>
        </Router>
    }
}
