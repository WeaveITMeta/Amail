//! Mailboxes Page - Mailbox management for enterprise email
//!
//! Table of Contents:
//! 1. MailboxesPage - Main page component
//! 2. MailboxCard - Individual mailbox display
//! 3. AddMailboxModal - Create new mailbox form

use leptos::*;
use wasm_bindgen_futures::spawn_local;
use crate::api;

// =============================================================================
// 1. MailboxesPage
// =============================================================================

#[component]
pub fn MailboxesPage() -> impl IntoView {
    // State
    let (mailboxes, set_mailboxes) = create_signal::<Vec<Mailbox>>(vec![]);
    let (domains, set_domains) = create_signal::<Vec<DomainOption>>(vec![]);
    let (loading, set_loading) = create_signal(true);
    let (error, set_error) = create_signal::<Option<String>>(None);
    let (show_add_modal, set_show_add_modal) = create_signal(false);
    let (selected_domain, set_selected_domain) = create_signal::<Option<String>>(None);
    let (search_query, set_search_query) = create_signal(String::new());
    
    // Form state for new mailbox
    let (new_local_part, set_new_local_part) = create_signal(String::new());
    let (new_display_name, set_new_display_name) = create_signal(String::new());
    let (new_domain_id, set_new_domain_id) = create_signal(String::new());
    
    // Fetch data on mount
    {
        let set_mailboxes = set_mailboxes.clone();
        let set_domains = set_domains.clone();
        let set_loading = set_loading.clone();
        let set_error = set_error.clone();
        
        spawn_local(async move {
            // Fetch domains first
            match api::fetch_domains().await {
                Ok(domain_list) => {
                    let domain_options: Vec<DomainOption> = domain_list.into_iter().map(|d| DomainOption {
                        id: d.id,
                        name: d.name,
                    }).collect();
                    set_domains.set(domain_options);
                }
                Err(e) => {
                    log::error!("Failed to fetch domains: {}", e);
                }
            }
            
            // Fetch all mailboxes
            match api::fetch_mailboxes().await {
                Ok(mailbox_list) => {
                    let mailboxes: Vec<Mailbox> = mailbox_list.into_iter().map(|m| Mailbox {
                        id: m.id,
                        email: m.email,
                        display_name: m.display_name,
                        domain: m.domain,
                        mailbox_type: m.mailbox_type,
                        is_active: m.is_active,
                        unread_count: m.unread_count,
                    }).collect();
                    set_mailboxes.set(mailboxes);
                }
                Err(e) => {
                    log::error!("Failed to fetch mailboxes: {}", e);
                    set_error.set(Some(format!("Failed to load mailboxes: {}", e)));
                }
            }
            set_loading.set(false);
        });
    }
    
    // Filtered mailboxes based on search and domain filter
    let filtered_mailboxes = move || {
        let query = search_query.get().to_lowercase();
        let domain_filter = selected_domain.get();
        
        mailboxes.get().into_iter().filter(|m| {
            let matches_search = query.is_empty() 
                || m.email.to_lowercase().contains(&query)
                || m.display_name.as_ref().map(|n| n.to_lowercase().contains(&query)).unwrap_or(false);
            
            let matches_domain = domain_filter.as_ref().map(|d| &m.domain == d).unwrap_or(true);
            
            matches_search && matches_domain
        }).collect::<Vec<_>>()
    };
    
    // Add mailbox handler
    let add_mailbox = move |_| {
        let local_part = new_local_part.get();
        let display_name = new_display_name.get();
        let domain_id = new_domain_id.get();
        
        if !local_part.is_empty() && !domain_id.is_empty() {
            let set_mailboxes = set_mailboxes.clone();
            let set_show_add_modal = set_show_add_modal.clone();
            let set_new_local_part = set_new_local_part.clone();
            let set_new_display_name = set_new_display_name.clone();
            let domains = domains.get();
            
            spawn_local(async move {
                let display = if display_name.is_empty() { None } else { Some(display_name.as_str()) };
                
                match api::create_mailbox(&domain_id, &local_part, display).await {
                    Ok(_) => {
                        // Find domain name
                        let domain_name = domains.iter()
                            .find(|d| d.id == domain_id)
                            .map(|d| d.name.clone())
                            .unwrap_or_default();
                        
                        // Add to local state
                        set_mailboxes.update(|m| {
                            m.push(Mailbox {
                                id: format!("{}-{}", local_part, domain_id),
                                email: format!("{}@{}", local_part, domain_name),
                                display_name: if display_name.is_empty() { None } else { Some(display_name.clone()) },
                                domain: domain_name,
                                mailbox_type: "user".to_string(),
                                is_active: true,
                                unread_count: 0,
                            });
                        });
                        
                        set_new_local_part.set(String::new());
                        set_new_display_name.set(String::new());
                        set_show_add_modal.set(false);
                    }
                    Err(e) => {
                        log::error!("Failed to create mailbox: {}", e);
                    }
                }
            });
        }
    };

    view! {
        <div class="page mailboxes-page">
            <div class="page-header">
                <h1 class="page-title">"Mailboxes"</h1>
                <div class="page-actions">
                    <button class="btn btn-primary" on:click=move |_| set_show_add_modal.set(true)>
                        <svg viewBox="0 0 24 24" fill="currentColor" class="btn-icon">
                            <path d="M19 13h-6v6h-2v-6H5v-2h6V5h2v6h6v2z"/>
                        </svg>
                        "Add Mailbox"
                    </button>
                </div>
            </div>
            
            // Filters
            <div class="filters-bar">
                <div class="search-box">
                    <svg viewBox="0 0 24 24" fill="currentColor" class="search-icon">
                        <path d="M15.5 14h-.79l-.28-.27C15.41 12.59 16 11.11 16 9.5 16 5.91 13.09 3 9.5 3S3 5.91 3 9.5 5.91 16 9.5 16c1.61 0 3.09-.59 4.23-1.57l.27.28v.79l5 4.99L20.49 19l-4.99-5zm-6 0C7.01 14 5 11.99 5 9.5S7.01 5 9.5 5 14 7.01 14 9.5 11.99 14 9.5 14z"/>
                    </svg>
                    <input
                        type="text"
                        class="search-input"
                        placeholder="Search mailboxes..."
                        prop:value=move || search_query.get()
                        on:input=move |e| set_search_query.set(event_target_value(&e))
                    />
                </div>
                
                <select 
                    class="filter-select"
                    on:change=move |e| {
                        let value = event_target_value(&e);
                        set_selected_domain.set(if value.is_empty() { None } else { Some(value) });
                    }
                >
                    <option value="">"All Domains"</option>
                    <For
                        each=move || domains.get()
                        key=|d| d.id.clone()
                        children=move |domain| {
                            view! {
                                <option value=domain.name.clone()>{&domain.name}</option>
                            }
                        }
                    />
                </select>
            </div>
            
            // Stats summary
            <div class="mailbox-stats">
                <div class="stat-item">
                    <span class="stat-value">{move || mailboxes.get().len()}</span>
                    <span class="stat-label">"Total Mailboxes"</span>
                </div>
                <div class="stat-item">
                    <span class="stat-value">{move || mailboxes.get().iter().filter(|m| m.is_active).count()}</span>
                    <span class="stat-label">"Active"</span>
                </div>
                <div class="stat-item">
                    <span class="stat-value">{move || mailboxes.get().iter().map(|m| m.unread_count).sum::<i64>()}</span>
                    <span class="stat-label">"Unread Emails"</span>
                </div>
            </div>
            
            // Mailbox list
            <Show when=move || loading.get()>
                <div class="loading-spinner">"Loading mailboxes..."</div>
            </Show>
            
            <Show when=move || error.get().is_some()>
                <div class="error-message">{move || error.get().unwrap_or_default()}</div>
            </Show>
            
            <div class="mailboxes-grid">
                <For
                    each=filtered_mailboxes
                    key=|m| m.id.clone()
                    children=move |mailbox| {
                        view! { <MailboxCard mailbox=mailbox/> }
                    }
                />
            </div>
            
            // Empty state
            <Show when=move || !loading.get() && filtered_mailboxes().is_empty()>
                <div class="empty-state">
                    <svg viewBox="0 0 24 24" fill="currentColor" class="empty-icon">
                        <path d="M20 4H4c-1.1 0-1.99.9-1.99 2L2 18c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V6c0-1.1-.9-2-2-2zm0 4l-8 5-8-5V6l8 5 8-5v2z"/>
                    </svg>
                    <h3>"No mailboxes found"</h3>
                    <p>"Create a mailbox to start receiving email"</p>
                </div>
            </Show>
            
            // Add Mailbox Modal
            <Show when=move || show_add_modal.get()>
                <div class="modal-overlay" on:click=move |_| set_show_add_modal.set(false)>
                    <div class="modal modal-lg" on:click=|e| e.stop_propagation()>
                        <div class="modal-header">
                            <h2>"Create New Mailbox"</h2>
                            <button class="modal-close" on:click=move |_| set_show_add_modal.set(false)>
                                "×"
                            </button>
                        </div>
                        <div class="modal-body">
                            <div class="form-group">
                                <label for="domain-select">"Domain"</label>
                                <select
                                    id="domain-select"
                                    class="form-select"
                                    on:change=move |e| set_new_domain_id.set(event_target_value(&e))
                                >
                                    <option value="">"Select a domain..."</option>
                                    <For
                                        each=move || domains.get()
                                        key=|d| d.id.clone()
                                        children=move |domain| {
                                            view! {
                                                <option value=domain.id.clone()>{&domain.name}</option>
                                            }
                                        }
                                    />
                                </select>
                            </div>
                            
                            <div class="form-group">
                                <label for="local-part">"Email Address"</label>
                                <div class="email-input-group">
                                    <input
                                        type="text"
                                        id="local-part"
                                        class="form-input"
                                        placeholder="username"
                                        prop:value=move || new_local_part.get()
                                        on:input=move |e| set_new_local_part.set(event_target_value(&e))
                                    />
                                    <span class="email-at">"@"</span>
                                    <span class="email-domain">
                                        {move || {
                                            let domain_id = new_domain_id.get();
                                            domains.get().iter()
                                                .find(|d| d.id == domain_id)
                                                .map(|d| d.name.clone())
                                                .unwrap_or_else(|| "domain.com".to_string())
                                        }}
                                    </span>
                                </div>
                            </div>
                            
                            <div class="form-group">
                                <label for="display-name">"Display Name (optional)"</label>
                                <input
                                    type="text"
                                    id="display-name"
                                    class="form-input"
                                    placeholder="John Doe"
                                    prop:value=move || new_display_name.get()
                                    on:input=move |e| set_new_display_name.set(event_target_value(&e))
                                />
                            </div>
                        </div>
                        <div class="modal-footer">
                            <button class="btn btn-secondary" on:click=move |_| set_show_add_modal.set(false)>
                                "Cancel"
                            </button>
                            <button 
                                class="btn btn-primary" 
                                on:click=add_mailbox
                                disabled=move || new_local_part.get().is_empty() || new_domain_id.get().is_empty()
                            >
                                "Create Mailbox"
                            </button>
                        </div>
                    </div>
                </div>
            </Show>
        </div>
    }
}

// =============================================================================
// 2. MailboxCard
// =============================================================================

#[derive(Clone, Debug)]
struct Mailbox {
    id: String,
    email: String,
    display_name: Option<String>,
    domain: String,
    mailbox_type: String,
    is_active: bool,
    unread_count: i64,
}

#[derive(Clone, Debug)]
struct DomainOption {
    id: String,
    name: String,
}

#[component]
fn MailboxCard(mailbox: Mailbox) -> impl IntoView {
    let type_badge = match mailbox.mailbox_type.as_str() {
        "admin" => ("Admin", "badge-purple"),
        "shared" => ("Shared", "badge-blue"),
        "alias" => ("Alias", "badge-gray"),
        _ => ("User", "badge-green"),
    };
    
    // Clone values for use in closures
    let has_display_name = mailbox.display_name.is_some();
    let display_name_text = mailbox.display_name.clone().unwrap_or_default();
    let has_unread = mailbox.unread_count > 0;
    let unread_count = mailbox.unread_count;
    let is_active = mailbox.is_active;
    let inbox_link = format!("/inbox?mailbox={}", mailbox.id);

    view! {
        <div class="mailbox-card">
            <div class="mailbox-avatar">
                {mailbox.email.chars().next().unwrap_or('?').to_uppercase().to_string()}
            </div>
            
            <div class="mailbox-info">
                <div class="mailbox-header">
                    <h3 class="mailbox-email">{&mailbox.email}</h3>
                    <span class=format!("badge {}", type_badge.1)>{type_badge.0}</span>
                </div>
                
                <Show when=move || has_display_name>
                    <p class="mailbox-name">{display_name_text.clone()}</p>
                </Show>
                
                <div class="mailbox-meta">
                    <span class="mailbox-domain">{&mailbox.domain}</span>
                    <Show when=move || has_unread>
                        <span class="unread-badge">{unread_count}" unread"</span>
                    </Show>
                </div>
            </div>
            
            <div class="mailbox-actions">
                <span class=if is_active { "status-dot active" } else { "status-dot inactive" }></span>
                <a href=inbox_link.clone() class="btn btn-sm btn-secondary">
                    "View Inbox"
                </a>
                <button class="btn btn-sm btn-icon" title="Settings">
                    <svg viewBox="0 0 24 24" fill="currentColor">
                        <path d="M19.14 12.94c.04-.31.06-.63.06-.94 0-.31-.02-.63-.06-.94l2.03-1.58c.18-.14.23-.41.12-.61l-1.92-3.32c-.12-.22-.37-.29-.59-.22l-2.39.96c-.5-.38-1.03-.7-1.62-.94l-.36-2.54c-.04-.24-.24-.41-.48-.41h-3.84c-.24 0-.43.17-.47.41l-.36 2.54c-.59.24-1.13.57-1.62.94l-2.39-.96c-.22-.08-.47 0-.59.22L2.74 8.87c-.12.21-.08.47.12.61l2.03 1.58c-.04.31-.06.63-.06.94s.02.63.06.94l-2.03 1.58c-.18.14-.23.41-.12.61l1.92 3.32c.12.22.37.29.59.22l2.39-.96c.5.38 1.03.7 1.62.94l.36 2.54c.05.24.24.41.48.41h3.84c.24 0 .44-.17.47-.41l.36-2.54c.59-.24 1.13-.56 1.62-.94l2.39.96c.22.08.47 0 .59-.22l1.92-3.32c.12-.22.07-.47-.12-.61l-2.01-1.58zM12 15.6c-1.98 0-3.6-1.62-3.6-3.6s1.62-3.6 3.6-3.6 3.6 1.62 3.6 3.6-1.62 3.6-3.6 3.6z"/>
                    </svg>
                </button>
            </div>
        </div>
    }
}
