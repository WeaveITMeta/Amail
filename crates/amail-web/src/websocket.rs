// =============================================================================
// WebSocket Connection Management
// =============================================================================
// Table of Contents:
// 1. WebSocket hook
// 2. Connection management
// 3. Message handling
// =============================================================================

use gloo_net::websocket::{futures::WebSocket, Message};
use leptos::*;
use wasm_bindgen_futures::spawn_local;
use futures::StreamExt;

use crate::state::{DashboardState, WsMessage};

/// WebSocket connection hook
/// Returns (connected signal, messages signal)
pub fn use_websocket(
    url: &'static str,
    state: DashboardState,
) -> (ReadSignal<bool>, ReadSignal<Vec<WsMessage>>) {
    let (connected, set_connected) = create_signal(false);
    let (messages, _set_messages) = create_signal(Vec::<WsMessage>::new());
    
    // Clone state for the async block
    let state_clone = state.clone();
    
    // Spawn WebSocket connection
    spawn_local(async move {
        connect_websocket(url, state_clone, set_connected).await;
    });
    
    (connected, messages)
}

/// Connect to WebSocket with automatic reconnection
async fn connect_websocket(
    url: &'static str,
    state: DashboardState,
    set_connected: WriteSignal<bool>,
) {
    loop {
        // 1. Show "Connecting" while attempting
        log::info!("Connecting to WebSocket: {}", url);
        state.set_connecting();
        
        match WebSocket::open(url) {
            Ok(ws) => {
                // 2. Show "Online" when connected
                log::info!("WebSocket connected");
                set_connected.set(true);
                state.set_connected(true);
                
                let mut ws = ws;
                
                // Read messages while connected
                while let Some(msg) = ws.next().await {
                    match msg {
                        Ok(Message::Text(text)) => {
                            match serde_json::from_str::<WsMessage>(&text) {
                                Ok(ws_msg) => {
                                    log::debug!("Received message: {:?}", ws_msg);
                                    state.handle_message(ws_msg);
                                }
                                Err(e) => {
                                    log::warn!("Failed to parse message: {} - {}", e, text);
                                }
                            }
                        }
                        Ok(Message::Bytes(_)) => {
                            log::debug!("Received binary message (ignored)");
                        }
                        Err(e) => {
                            log::error!("WebSocket error: {:?}", e);
                            break;
                        }
                    }
                }
                
                // 3. Show "Offline" when disconnected
                log::warn!("WebSocket disconnected");
                set_connected.set(false);
                state.set_connected(false);
            }
            Err(e) => {
                // 3. Show "Offline" when connection fails
                log::error!("Failed to connect to WebSocket: {:?}", e);
                set_connected.set(false);
                state.set_connected(false);
            }
        }
        
        // 4. Stay "Offline" during wait, then loop back to "Connecting"
        log::info!("Reconnecting in 3 seconds...");
        gloo_timers::future::sleep(std::time::Duration::from_millis(3000)).await;
    }
}
