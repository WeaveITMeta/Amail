//! Dashboard Page - Main overview with domain selector
//!
//! The ultimate AI-powered mail server dashboard.
//! Self-hosted. Open source. Free forever.

use leptos::*;
use wasm_bindgen_futures::spawn_local;
use crate::state::DashboardState;
use crate::api;

#[derive(Clone, Debug, PartialEq, Default)]
pub struct DomainStats {
    pub id: String,
    pub name: String,
    pub mailbox_count: u32,
    pub emails_today: u32,
    pub ai_replies: u32,
    pub pending_review: u32,
    pub status: String,
}

#[component]
pub fn DashboardPage() -> impl IntoView {
    let state = use_context::<DashboardState>().expect("DashboardState not provided");
    let stats = state.stats;
    let agents = state.agents;
    
    // Domain selector state - fetched from API
    let (domains, set_domains) = create_signal::<Vec<DomainStats>>(vec![]);
    let (selected_domain, set_selected_domain) = create_signal::<Option<String>>(None);
    let (dropdown_open, set_dropdown_open) = create_signal(false);
    let (loading, set_loading) = create_signal(true);
    
    // Fetch domains and stats from API
    {
        let set_domains = set_domains.clone();
        let set_loading = set_loading.clone();
        
        spawn_local(async move {
            match api::fetch_domains().await {
                Ok(domain_list) => {
                    let domains: Vec<DomainStats> = domain_list.into_iter().map(|d| DomainStats {
                        id: d.id,
                        name: d.name,
                        mailbox_count: d.mailbox_count as u32,
                        emails_today: 0, // Will be populated from stats
                        ai_replies: 0,
                        pending_review: 0,
                        status: d.status,
                    }).collect();
                    set_domains.set(domains);
                }
                Err(e) => {
                    log::error!("Failed to fetch domains: {}", e);
                }
            }
            
            // Also fetch dashboard stats
            match api::fetch_stats().await {
                Ok(dashboard_stats) => {
                    // Update domains with real stats if available
                    log::info!("Dashboard stats loaded: {} emails today", dashboard_stats.emails_today);
                }
                Err(e) => {
                    log::warn!("Failed to fetch dashboard stats: {}", e);
                }
            }
            
            set_loading.set(false);
        });
    }
    
    // Filtered stats based on selected domain
    let current_domain = move || {
        let sel = selected_domain.get();
        domains.get().into_iter().find(|d| Some(d.id.clone()) == sel)
    };
    
    let total_stats = move || {
        let all = domains.get();
        (
            all.iter().map(|d| d.mailbox_count).sum::<u32>(),
            all.iter().map(|d| d.emails_today).sum::<u32>(),
            all.iter().map(|d| d.ai_replies).sum::<u32>(),
            all.iter().map(|d| d.pending_review).sum::<u32>(),
        )
    };

    view! {
        <div class="page dashboard-page">
            // Header with domain selector
            <div class="dashboard-header">
                <div class="dashboard-title-section">
                    <h1 class="page-title">"Dashboard"</h1>
                    <p class="page-subtitle">"Your AI mail server at a glance"</p>
                </div>
                
                // Domain Selector Dropdown
                <div class="domain-selector">
                    <button 
                        class="domain-selector-btn"
                        on:click=move |_| set_dropdown_open.update(|o| *o = !*o)
                    >
                        <svg viewBox="0 0 24 24" fill="currentColor" class="selector-icon">
                            <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-1 17.93c-3.95-.49-7-3.85-7-7.93 0-.62.08-1.21.21-1.79L9 15v1c0 1.1.9 2 2 2v1.93zm6.9-2.54c-.26-.81-1-1.39-1.9-1.39h-1v-3c0-.55-.45-1-1-1H8v-2h2c.55 0 1-.45 1-1V7h2c1.1 0 2-.9 2-2v-.41c2.93 1.19 5 4.06 5 7.41 0 2.08-.8 3.97-2.1 5.39z"/>
                        </svg>
                        <span class="selector-text">
                            {move || match selected_domain.get() {
                                None => "All Domains".to_string(),
                                Some(_) => current_domain().map(|d| d.name.clone()).unwrap_or("All Domains".to_string())
                            }}
                        </span>
                        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="selector-arrow">
                            <path d="M6 9l6 6 6-6"/>
                        </svg>
                    </button>
                    
                    <Show when=move || dropdown_open.get()>
                        <div class="domain-dropdown">
                            <button 
                                class=move || if selected_domain.get().is_none() { "dropdown-item active" } else { "dropdown-item" }
                                on:click=move |_| { set_selected_domain.set(None); set_dropdown_open.set(false); }
                            >
                                <svg viewBox="0 0 24 24" fill="currentColor"><path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2z"/></svg>
                                <span>"All Domains"</span>
                                <span class="dropdown-count">{move || domains.get().len()}" domains"</span>
                            </button>
                            <div class="dropdown-divider"></div>
                            {move || domains.get().into_iter().map(|domain| {
                                let domain_id = domain.id.clone();
                                let domain_id2 = domain.id.clone();
                                let is_selected = move || selected_domain.get() == Some(domain_id.clone());
                                view! {
                                    <button 
                                        class=move || if is_selected() { "dropdown-item active" } else { "dropdown-item" }
                                        on:click=move |_| { set_selected_domain.set(Some(domain_id2.clone())); set_dropdown_open.set(false); }
                                    >
                                        <div class=format!("domain-status-dot status-{}", domain.status)></div>
                                        <span>{&domain.name}</span>
                                        <span class="dropdown-count">{domain.mailbox_count}" mailboxes"</span>
                                    </button>
                                }
                            }).collect_view()}
                        </div>
                    </Show>
                </div>
                
                // Quick Actions
                <div class="quick-actions">
                    <button class="quick-action-btn primary">
                        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M3 8l7.89 5.26a2 2 0 002.22 0L21 8M5 19h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z"/></svg>
                        "Go to Inbox"
                    </button>
                </div>
            </div>
            
            // Stats cards
            <div class="stats-grid">
                <div class="stat-card">
                    <div class="stat-card-icon domains-icon">
                        <svg viewBox="0 0 24 24" fill="currentColor">
                            <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-1 17.93c-3.95-.49-7-3.85-7-7.93 0-.62.08-1.21.21-1.79L9 15v1c0 1.1.9 2 2 2v1.93zm6.9-2.54c-.26-.81-1-1.39-1.9-1.39h-1v-3c0-.55-.45-1-1-1H8v-2h2c.55 0 1-.45 1-1V7h2c1.1 0 2-.9 2-2v-.41c2.93 1.19 5 4.06 5 7.41 0 2.08-.8 3.97-2.1 5.39z"/>
                        </svg>
                    </div>
                    <div class="stat-card-content">
                        <span class="stat-card-value">
                            {move || match current_domain() {
                                Some(d) => d.mailbox_count.to_string(),
                                None => total_stats().0.to_string()
                            }}
                        </span>
                        <span class="stat-card-label">"Mailboxes"</span>
                    </div>
                </div>
                
                <div class="stat-card">
                    <div class="stat-card-icon email-icon">
                        <svg viewBox="0 0 24 24" fill="currentColor">
                            <path d="M20 4H4c-1.1 0-2 .9-2 2v12c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V6c0-1.1-.9-2-2-2zm0 4l-8 5-8-5V6l8 5 8-5v2z"/>
                        </svg>
                    </div>
                    <div class="stat-card-content">
                        <span class="stat-card-value">
                            {move || match current_domain() {
                                Some(d) => d.emails_today.to_string(),
                                None => total_stats().1.to_string()
                            }}
                        </span>
                        <span class="stat-card-label">"Emails Today"</span>
                    </div>
                </div>
                
                <div class="stat-card">
                    <div class="stat-card-icon ai-icon">
                        <svg viewBox="0 0 24 24" fill="currentColor">
                            <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm0 3c1.66 0 3 1.34 3 3s-1.34 3-3 3-3-1.34-3-3 1.34-3 3-3zm0 14.2c-2.5 0-4.71-1.28-6-3.22.03-1.99 4-3.08 6-3.08 1.99 0 5.97 1.09 6 3.08-1.29 1.94-3.5 3.22-6 3.22z"/>
                        </svg>
                    </div>
                    <div class="stat-card-content">
                        <span class="stat-card-value">
                            {move || match current_domain() {
                                Some(d) => d.ai_replies.to_string(),
                                None => total_stats().2.to_string()
                            }}
                        </span>
                        <span class="stat-card-label">"AI Replies"</span>
                    </div>
                </div>
                
                <div class="stat-card highlight">
                    <div class="stat-card-icon pending-icon">
                        <svg viewBox="0 0 24 24" fill="currentColor">
                            <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-2h2v2zm0-4h-2V7h2v6z"/>
                        </svg>
                    </div>
                    <div class="stat-card-content">
                        <span class="stat-card-value">
                            {move || match current_domain() {
                                Some(d) => d.pending_review.to_string(),
                                None => total_stats().3.to_string()
                            }}
                        </span>
                        <span class="stat-card-label">"Pending Review"</span>
                    </div>
                </div>
            </div>
            
            // Domain overview cards
            <div class="section">
                <div class="section-header">
                    <h2 class="section-title">"Domains Overview"</h2>
                    <a href="/domains" class="section-action">"Manage Domains →"</a>
                </div>
                <div class="domains-overview-grid">
                    {move || domains.get().into_iter().map(|domain| {
                        view! {
                            <div class="domain-overview-card">
                                <div class="domain-card-header">
                                    <div class=format!("domain-status-indicator status-{}", domain.status)></div>
                                    <h3 class="domain-name">{&domain.name}</h3>
                                </div>
                                <div class="domain-card-stats">
                                    <div class="domain-stat">
                                        <span class="domain-stat-value">{domain.mailbox_count}</span>
                                        <span class="domain-stat-label">"Mailboxes"</span>
                                    </div>
                                    <div class="domain-stat">
                                        <span class="domain-stat-value">{domain.emails_today}</span>
                                        <span class="domain-stat-label">"Today"</span>
                                    </div>
                                    <div class="domain-stat">
                                        <span class="domain-stat-value">{domain.ai_replies}</span>
                                        <span class="domain-stat-label">"AI Sent"</span>
                                    </div>
                                    <div class="domain-stat">
                                        <span class="domain-stat-value">{domain.pending_review}</span>
                                        <span class="domain-stat-label">"Pending"</span>
                                    </div>
                                </div>
                                <button class="domain-card-action">"View Inbox →"</button>
                            </div>
                        }
                    }).collect_view()}
                </div>
            </div>
            
            // Agent status overview
            <div class="section">
                <div class="section-header">
                    <h2 class="section-title">"AI Agent Status"</h2>
                    <span class="section-badge">{move || stats.get().total_agents}" agents"</span>
                </div>
                <div class="agents-overview">
                    <For
                        each=move || agents.get().into_iter()
                        key=|(id, _)| id.clone()
                        children=move |(_id, agent)| {
                            let status_class = match agent.status.as_str() {
                                "idle" => "status-idle",
                                "busy" => "status-busy",
                                "error" => "status-error",
                                _ => "status-unknown",
                            };
                            view! {
                                <div class="agent-status-card">
                                    <div class=format!("agent-status-indicator {}", status_class)></div>
                                    <div class="agent-status-info">
                                        <span class="agent-name">{agent.name.clone()}</span>
                                    </div>
                                    <div class="agent-stats">
                                        <span class="agent-tasks">{agent.completed_tasks}" completed"</span>
                                    </div>
                                </div>
                            }
                        }
                    />
                </div>
            </div>
        </div>
    }
}

// Sample data removed - now using real API data
