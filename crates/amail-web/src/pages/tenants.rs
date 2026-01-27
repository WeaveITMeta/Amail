//! Domains Page - Self-Hosted Domain Management
//! AMAIL - 100% Open Source, Self-Hosted, FREE Forever

use leptos::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use crate::api;

#[derive(Clone, Debug, PartialEq, Default)]
pub struct DomainData {
    pub id: String,
    pub domain_name: String,
    pub status: String,
    pub verification_progress: u8,
    pub mailbox_count: u32,
    pub created_at: String,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct MailboxData {
    pub id: String,
    pub email_address: String,
    pub display_name: Option<String>,
    pub agent_role: Option<String>,
    pub is_active: bool,
}

#[component]
pub fn TenantsPage() -> impl IntoView {
    let (domains, set_domains) = create_signal::<Vec<DomainData>>(vec![]);
    let (selected_domain, set_selected_domain) = create_signal::<Option<DomainData>>(None);
    let (show_add_modal, set_show_add_modal) = create_signal(false);
    let (search_query, set_search_query) = create_signal(String::new());
    let (loading, set_loading) = create_signal(true);
    
    // Fetch domains from API
    {
        let set_domains = set_domains.clone();
        let set_loading = set_loading.clone();
        
        spawn_local(async move {
            match api::fetch_domains().await {
                Ok(domain_list) => {
                    let domains: Vec<DomainData> = domain_list.into_iter().map(|d| {
                        // Calculate verification progress based on DNS records
                        let verified_count = [d.mx_verified, d.spf_verified, d.dkim_verified, d.dmarc_verified]
                            .iter().filter(|&&v| v).count();
                        let progress = (verified_count * 25) as u8;
                        
                        DomainData {
                            id: d.id,
                            domain_name: d.name,
                            status: d.status,
                            verification_progress: progress,
                            mailbox_count: d.mailbox_count as u32,
                            created_at: d.created_at,
                        }
                    }).collect();
                    set_domains.set(domains);
                }
                Err(e) => {
                    log::error!("Failed to fetch domains: {}", e);
                }
            }
            set_loading.set(false);
        });
    }

    let filtered_domains = move || {
        let query = search_query.get().to_lowercase();
        domains.get().into_iter()
            .filter(|d| query.is_empty() || d.domain_name.to_lowercase().contains(&query))
            .collect::<Vec<_>>()
    };

    let stats = move || {
        let all = domains.get();
        (all.len(), all.iter().filter(|d| d.status == "active").count(), 
         all.iter().map(|d| d.mailbox_count as usize).sum::<usize>())
    };

    view! {
        <div class="tenants-page">
            <Show 
                when=move || selected_domain.get().is_none()
                fallback=move || {
                    let domain = selected_domain.get().unwrap();
                    view! { <DomainDetailView domain=domain on_back=move |_| set_selected_domain.set(None)/> }
                }
            >
                <div class="page-header">
                    <div>
                        <h1 class="page-title">"Your Domains"</h1>
                        <p class="page-subtitle">"Manage your email domains - unlimited, free forever"</p>
                    </div>
                    <button class="btn btn-primary" on:click=move |_| set_show_add_modal.set(true)>
                        <svg viewBox="0 0 24 24" fill="currentColor" class="btn-icon"><path d="M19 13h-6v6h-2v-6H5v-2h6V5h2v6h6v2z"/></svg>
                        "Add Domain"
                    </button>
                </div>
                
                <div class="philosophy-banner">
                    <div class="philosophy-icon">
                        <svg viewBox="0 0 24 24" fill="currentColor"><path d="M12 1L3 5v6c0 5.55 3.84 10.74 9 12 5.16-1.26 9-6.45 9-12V5l-9-4z"/></svg>
                    </div>
                    <div class="philosophy-text">
                        <strong>"100% Open Source & Self-Hosted"</strong>
                        <span>" - No subscriptions. No limits. Your data stays on YOUR server."</span>
                    </div>
                </div>
                
                <div class="tenant-stats">
                    {move || {
                        let (total, active, mailboxes) = stats();
                        view! {
                            <div class="stat-card">
                                <div class="stat-card-icon total-icon"><svg viewBox="0 0 24 24" fill="currentColor"><path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2z"/></svg></div>
                                <div class="stat-card-content"><span class="stat-card-value">{total}</span><span class="stat-card-label">"Domains"</span></div>
                            </div>
                            <div class="stat-card">
                                <div class="stat-card-icon active-icon"><svg viewBox="0 0 24 24" fill="currentColor"><path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z"/></svg></div>
                                <div class="stat-card-content"><span class="stat-card-value">{active}</span><span class="stat-card-label">"Active"</span></div>
                            </div>
                            <div class="stat-card">
                                <div class="stat-card-icon b2b-icon"><svg viewBox="0 0 24 24" fill="currentColor"><path d="M20 4H4c-1.1 0-2 .9-2 2v12c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V6c0-1.1-.9-2-2-2z"/></svg></div>
                                <div class="stat-card-content"><span class="stat-card-value">{mailboxes}</span><span class="stat-card-label">"Mailboxes"</span></div>
                            </div>
                            <div class="stat-card">
                                <div class="stat-card-icon b2c-icon"><svg viewBox="0 0 24 24" fill="currentColor"><path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2z"/></svg></div>
                                <div class="stat-card-content"><span class="stat-card-value">{total}</span><span class="stat-card-label">"AI Agents"</span></div>
                            </div>
                        }
                    }}
                </div>
                
                <div class="tenant-filters">
                    <div class="search-box">
                        <svg viewBox="0 0 24 24" fill="currentColor" class="search-icon"><path d="M15.5 14h-.79l-.28-.27C15.41 12.59 16 11.11 16 9.5 16 5.91 13.09 3 9.5 3S3 5.91 3 9.5 5.91 16 9.5 16c1.61 0 3.09-.59 4.23-1.57l.27.28v.79l5 4.99L20.49 19l-4.99-5zm-6 0C7.01 14 5 11.99 5 9.5S7.01 5 9.5 5 14 7.01 14 9.5 11.99 14 9.5 14z"/></svg>
                        <input type="text" placeholder="Search domains..." prop:value=move || search_query.get() on:input=move |e| set_search_query.set(event_target_value(&e))/>
                    </div>
                </div>
                
                <div class="tenant-list">
                    {move || filtered_domains().into_iter().map(|domain| {
                        let dc = domain.clone();
                        view! { <DomainCard domain=domain on_click=move |_| set_selected_domain.set(Some(dc.clone()))/> }
                    }).collect_view()}
                </div>
            </Show>
            
            <Show when=move || show_add_modal.get()>
                <AddDomainModal 
                    on_close=move || set_show_add_modal.set(false)
                    on_domain_added=move |domain: DomainData| {
                        set_domains.update(|d| d.push(domain));
                    }
                />
            </Show>
        </div>
    }
}

/// Convert domain name to PascalCase display format
fn to_pascal_case(s: &str) -> String {
    s.split('.')
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().chain(chars).collect(),
            }
        })
        .collect::<Vec<_>>()
        .join(".")
}

#[component]
fn DomainCard(
    domain: DomainData, 
    on_click: impl Fn(ev::MouseEvent) + 'static,
    #[prop(optional)] on_inbox: Option<Box<dyn Fn(String) + 'static>>,
) -> impl IntoView {
    let status_class = if domain.status == "active" { "status-active" } else { "status-pending" };
    let display_name = to_pascal_case(&domain.domain_name);
    
    view! {
        <div class="tenant-card" on:click=on_click>
            <div class="tenant-card-header">
                <div class="tenant-avatar"><svg viewBox="0 0 24 24" fill="currentColor"><path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2z"/></svg></div>
                <div class="tenant-info">
                    <h3 class="tenant-name">{display_name}</h3>
                </div>
                <span class=format!("tenant-status-badge {}", status_class)>{&domain.status}</span>
            </div>
            <div class="tenant-card-body">
                <div class="tenant-metrics">
                    <div class="metric"><span class="metric-value">{domain.mailbox_count}</span><span class="metric-label">"Mailboxes"</span></div>
                    <div class="metric"><span class="metric-value">{domain.verification_progress}"%"</span><span class="metric-label">"Verified"</span></div>
                </div>
                <div class="tenant-usage">
                    <div class="usage-header"><span class="usage-label">"DNS Verification"</span></div>
                    <div class="usage-bar"><div class="usage-fill" style=format!("width: {}%", domain.verification_progress)></div></div>
                </div>
            </div>
            <div class="tenant-card-footer">
                <span class="tenant-created">"Added "{&domain.created_at}</span>
                <div class="tenant-card-actions">
                    <button class="btn btn-sm btn-inbox" title="Open Inbox">
                        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                            <path d="M3 8l7.89 5.26a2 2 0 002.22 0L21 8M5 19h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z"/>
                        </svg>
                        "Inbox"
                    </button>
                    <button class="btn btn-sm btn-ghost">"Configure →"</button>
                </div>
            </div>
        </div>
    }
}

#[component]
fn AddDomainModal(
    on_close: impl Fn() + Clone + 'static,
    on_domain_added: impl Fn(DomainData) + Clone + 'static,
) -> impl IntoView {
    let (domain_name, set_domain_name) = create_signal(String::new());
    let (adding, set_adding) = create_signal(false);
    let (error, set_error) = create_signal::<Option<String>>(None);
    let (success, set_success) = create_signal::<Option<String>>(None);
    
    let on_close_overlay = on_close.clone();
    let on_close_x = on_close.clone();
    let on_close_cancel = on_close.clone();
    let on_close_success = on_close.clone();
    let on_domain_added_clone = on_domain_added.clone();
    
    let handle_add = move |_| {
        let name = domain_name.get().trim().to_string();
        log::info!("Add domain button clicked, domain: '{}'", name);
        
        if name.is_empty() {
            set_error.set(Some("Please enter a domain name".to_string()));
            return;
        }
        
        set_adding.set(true);
        set_error.set(None);
        
        let set_adding = set_adding.clone();
        let set_error = set_error.clone();
        let set_success = set_success.clone();
        let set_domain_name = set_domain_name.clone();
        let on_close = on_close_success.clone();
        let on_domain_added = on_domain_added_clone.clone();
        
        spawn_local(async move {
            log::info!("Calling API to create domain: {}", name);
            match api::create_domain(&name).await {
                Ok(new_domain) => {
                    log::info!("Domain '{}' created successfully with id: {}", new_domain.name, new_domain.id);
                    
                    // Calculate verification progress
                    let verified_count = [new_domain.mx_verified, new_domain.spf_verified, new_domain.dkim_verified, new_domain.dmarc_verified]
                        .iter().filter(|&&v| v).count();
                    let progress = (verified_count * 25) as u8;
                    
                    let domain_data = DomainData {
                        id: new_domain.id,
                        domain_name: new_domain.name.clone(),
                        status: new_domain.status,
                        verification_progress: progress,
                        mailbox_count: 0,
                        created_at: new_domain.created_at,
                    };
                    
                    set_success.set(Some(format!("Domain '{}' added successfully!", new_domain.name)));
                    set_domain_name.set(String::new());
                    
                    // Notify parent and close after delay
                    let on_domain_added = on_domain_added.clone();
                    on_domain_added(domain_data);
                    
                    gloo_timers::callback::Timeout::new(1500, move || {
                        on_close();
                    }).forget();
                }
                Err(e) => {
                    log::error!("Failed to create domain: {}", e);
                    set_error.set(Some(format!("Failed to add domain: {}", e)));
                }
            }
            set_adding.set(false);
        });
    };
    
    view! {
        <div class="modal-overlay" on:click=move |_| on_close_overlay()>
            <div class="modal tenant-modal" on:click=|e| e.stop_propagation()>
                <div class="modal-header">
                    <h2>"Add New Domain"</h2>
                    <button class="modal-close" on:click=move |_| on_close_x() disabled=move || adding.get()>"×"</button>
                </div>
                <div class="modal-body">
                    <div class="philosophy-banner" style="margin-bottom: 20px;">
                        <div class="philosophy-icon"><svg viewBox="0 0 24 24" fill="currentColor"><path d="M12 1L3 5v6c0 5.55 3.84 10.74 9 12 5.16-1.26 9-6.45 9-12V5l-9-4z"/></svg></div>
                        <div class="philosophy-text"><strong>"No limits!"</strong><span>" Add as many domains as you need."</span></div>
                    </div>
                    
                    // Success message
                    <Show when=move || success.get().is_some()>
                        <div class="alert alert-success" style="margin-bottom: 16px;">
                            <svg viewBox="0 0 24 24" fill="currentColor" class="alert-icon">
                                <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z"/>
                            </svg>
                            {move || success.get().unwrap_or_default()}
                        </div>
                    </Show>
                    
                    // Error message
                    <Show when=move || error.get().is_some()>
                        <div class="alert alert-error" style="margin-bottom: 16px;">
                            <svg viewBox="0 0 24 24" fill="currentColor" class="alert-icon">
                                <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-2h2v2zm0-4h-2V7h2v6z"/>
                            </svg>
                            {move || error.get().unwrap_or_default()}
                        </div>
                    </Show>
                    
                    <div class="form-group">
                        <label>"Domain Name"</label>
                        <input 
                            type="text" 
                            class="form-input" 
                            placeholder="example.com" 
                            prop:value=move || domain_name.get() 
                            on:input=move |e| set_domain_name.set(event_target_value(&e))
                            on:keydown=move |e: ev::KeyboardEvent| {
                                if e.key() == "Enter" && !adding.get() {
                                    e.prevent_default();
                                    // Trigger the add button click
                                    if let Some(btn) = document().query_selector(".modal .btn-primary").ok().flatten() {
                                        if let Some(btn) = btn.dyn_ref::<web_sys::HtmlElement>() {
                                            btn.click();
                                        }
                                    }
                                }
                            }
                            disabled=move || adding.get()
                        />
                        <span class="form-hint">"Enter the domain you want to receive emails for"</span>
                    </div>
                </div>
                <div class="modal-footer">
                    <button 
                        class="btn btn-secondary" 
                        on:click=move |_| on_close_cancel()
                        disabled=move || adding.get()
                    >
                        "Cancel"
                    </button>
                    <button 
                        class="btn btn-primary" 
                        on:click=handle_add
                        disabled=move || adding.get()
                    >
                        {move || if adding.get() { "Adding..." } else { "Add Domain" }}
                    </button>
                </div>
            </div>
        </div>
    }
}

#[component]
fn DomainDetailView(domain: DomainData, on_back: impl Fn(ev::MouseEvent) + 'static) -> impl IntoView {
    let (mailboxes, set_mailboxes) = create_signal::<Vec<MailboxData>>(vec![]);
    let (show_add, set_show_add) = create_signal(false);
    let (delete_mailbox_id, set_delete_mailbox_id) = create_signal::<Option<String>>(None);
    let (delete_mailbox_email, set_delete_mailbox_email) = create_signal::<Option<String>>(None);
    let (show_delete_domain, set_show_delete_domain) = create_signal(false);
    
    // Create reactive signals for domain data that can be updated
    let (current_domain, set_current_domain) = create_signal(domain.clone());
    
    // Fetch fresh domain data and mailboxes from API
    {
        let domain_id = domain.id.clone();
        let set_mailboxes = set_mailboxes.clone();
        let set_current_domain = set_current_domain.clone();
        
        spawn_local(async move {
            // Fetch fresh domain data to get updated verification status
            if let Ok(domain_list) = api::fetch_domains().await {
                if let Some(d) = domain_list.into_iter().find(|d| d.id.to_lowercase() == domain_id.to_lowercase()) {
                    let verified_count = [d.mx_verified, d.spf_verified, d.dkim_verified, d.dmarc_verified]
                        .iter().filter(|&&v| v).count();
                    let progress = (verified_count * 25) as u8;
                    
                    set_current_domain.set(DomainData {
                        id: d.id,
                        domain_name: d.name,
                        status: d.status,
                        verification_progress: progress,
                        mailbox_count: d.mailbox_count as u32,
                        created_at: d.created_at,
                    });
                }
            }
            
            // Fetch mailboxes
            match api::fetch_mailboxes_by_domain(&domain_id).await {
                Ok(mailbox_list) => {
                    let mailboxes: Vec<MailboxData> = mailbox_list.into_iter().map(|m| MailboxData {
                        id: m.id,
                        email_address: m.email,
                        display_name: m.display_name,
                        agent_role: Some(m.mailbox_type),
                        is_active: m.is_active,
                    }).collect();
                    set_mailboxes.set(mailboxes);
                }
                Err(e) => {
                    log::warn!("Failed to fetch mailboxes: {}", e);
                }
            }
        });
    }
    
    // Store domain_id for closures before moving into view
    let domain_id_for_wizard = domain.id.clone();
    let domain_id_for_modal = domain.id.clone();
    let domain_name_for_modal = domain.domain_name.clone();
    let domain_id_for_delete = domain.id.clone();
    let domain_name_for_delete = domain.domain_name.clone();
    
    view! {
        <div class="tenant-detail">
            <div class="detail-header">
                <button class="back-btn icon-only" on:click=on_back title="Back">
                    <svg viewBox="0 0 24 24" fill="currentColor" width="24" height="24">
                        <path d="M15.41 7.41L14 6l-6 6 6 6 1.41-1.41L10.83 12z"/>
                    </svg>
                </button>
                <div class="detail-title">
                    <div class="tenant-avatar large"><svg viewBox="0 0 24 24" fill="currentColor"><path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2z"/></svg></div>
                    <div><h1>{move || current_domain.get().domain_name}</h1><span class="tenant-id">"ID: "{move || current_domain.get().id}</span></div>
                </div>
                <button 
                    class="btn btn-danger btn-sm" 
                    style="margin-left: auto;"
                    on:click=move |_| set_show_delete_domain.set(true)
                    title="Delete Domain"
                >
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" style="width: 16px; height: 16px; margin-right: 6px;">
                        <path d="M3 6h18M19 6v14a2 2 0 01-2 2H7a2 2 0 01-2-2V6m3 0V4a2 2 0 012-2h4a2 2 0 012 2v2"/>
                    </svg>
                    "Delete Domain"
                </button>
            </div>
            <div class="detail-info-cards">
                <div class="info-card"><h4>"Domain Info"</h4><dl><dt>"Status"</dt><dd>{move || current_domain.get().status}</dd><dt>"Mailboxes"</dt><dd>{move || mailboxes.get().len()}</dd></dl></div>
                <div class="info-card">
                    <h4>"DNS Verification"</h4>
                    <dl>
                        <dt>"Progress"</dt><dd>{move || format!("{}%", current_domain.get().verification_progress)}</dd>
                        <dt>"MX"</dt><dd>{move || if current_domain.get().verification_progress >= 25 { "✓" } else { "✗" }}</dd>
                        <dt>"SPF"</dt><dd>{move || if current_domain.get().verification_progress >= 50 { "✓" } else { "✗" }}</dd>
                        <dt>"DKIM"</dt><dd>{move || if current_domain.get().verification_progress >= 75 { "✓" } else { "✗" }}</dd>
                        <dt>"DMARC"</dt><dd>{move || if current_domain.get().verification_progress >= 100 { "✓" } else { "✗" }}</dd>
                    </dl>
                    <a 
                        class="btn btn-primary btn-sm"
                        style="margin-top: 12px; display: inline-block; text-decoration: none;"
                        href={format!("/dns-wizard/{}", domain_id_for_wizard)}
                    >
                        "Configure DNS Records"
                    </a>
                </div>
            </div>
            <div class="tab-content">
                <div class="tab-header"><h3>"Mailboxes"</h3><button class="btn btn-primary btn-sm" on:click=move |_| set_show_add.set(true)>"+ Add Mailbox"</button></div>
                <div class="mailboxes-table">
                    <table>
                        <thead><tr><th>"Email"</th><th>"Name"</th><th>"Status"</th><th style="width: 50px;"></th></tr></thead>
                        <tbody>
                            {move || mailboxes.get().into_iter().map(|m| {
                                let mailbox_id = m.id.clone();
                                let mailbox_email = m.email_address.clone();
                                view! {
                                    <tr>
                                        <td class="email-address">{m.email_address.clone()}</td>
                                        <td>{m.display_name.clone().unwrap_or("-".to_string())}</td>
                                        <td><span class=if m.is_active { "status-badge status-active" } else { "status-badge" }>{if m.is_active { "Active" } else { "Inactive" }}</span></td>
                                        <td>
                                            <button 
                                                class="btn-icon btn-delete" 
                                                title="Delete mailbox"
                                                on:click=move |_| {
                                                    set_delete_mailbox_id.set(Some(mailbox_id.clone()));
                                                    set_delete_mailbox_email.set(Some(mailbox_email.clone()));
                                                }
                                            >
                                                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                                    <path d="M3 6h18M19 6v14a2 2 0 01-2 2H7a2 2 0 01-2-2V6m3 0V4a2 2 0 012-2h4a2 2 0 012 2v2"/>
                                                </svg>
                                            </button>
                                        </td>
                                    </tr>
                                }
                            }).collect_view()}
                        </tbody>
                    </table>
                </div>
            </div>
            <Show when=move || show_add.get()>
                <AddMailboxModal 
                    domain_id=domain_id_for_modal.clone()
                    domain_name=domain_name_for_modal.clone()
                    on_close=move || set_show_add.set(false)
                    on_mailbox_added=move |mailbox: MailboxData| {
                        set_mailboxes.update(|m| m.push(mailbox));
                    }
                />
            </Show>
            <Show when=move || delete_mailbox_id.get().is_some()>
                <DeleteMailboxModal 
                    mailbox_id=delete_mailbox_id.get().unwrap_or_default()
                    mailbox_email=delete_mailbox_email.get().unwrap_or_default()
                    on_close=move || {
                        set_delete_mailbox_id.set(None);
                        set_delete_mailbox_email.set(None);
                    }
                    on_deleted=move |deleted_id: String| {
                        set_mailboxes.update(|m| m.retain(|mb| mb.id != deleted_id));
                        set_delete_mailbox_id.set(None);
                        set_delete_mailbox_email.set(None);
                    }
                />
            </Show>
            <Show when=move || show_delete_domain.get()>
                <DeleteDomainModal 
                    domain_id=domain_id_for_delete.clone()
                    domain_name=domain_name_for_delete.clone()
                    on_close=move || set_show_delete_domain.set(false)
                />
            </Show>
        </div>
    }
}

#[component]
fn AddMailboxModal(
    domain_id: String,
    domain_name: String,
    on_close: impl Fn() + Clone + 'static,
    on_mailbox_added: impl Fn(MailboxData) + Clone + 'static,
) -> impl IntoView {
    let (local_part, set_local_part) = create_signal(String::new());
    let (display_name, set_display_name) = create_signal(String::new());
    let (adding, set_adding) = create_signal(false);
    let (error, set_error) = create_signal::<Option<String>>(None);
    let (success, set_success) = create_signal::<Option<String>>(None);
    
    let on_close_overlay = on_close.clone();
    let on_close_x = on_close.clone();
    let on_close_cancel = on_close.clone();
    let on_close_success = on_close.clone();
    let on_mailbox_added_clone = on_mailbox_added.clone();
    let domain_id_clone = domain_id.clone();
    let domain_name_clone = domain_name.clone();
    
    let handle_add = move |_| {
        let local = local_part.get().trim().to_lowercase();
        log::info!("Add mailbox clicked: {}@{}", local, domain_name_clone);
        
        if local.is_empty() {
            set_error.set(Some("Please enter an email address".to_string()));
            return;
        }
        
        let display = display_name.get();
        if display.is_empty() {
            set_error.set(Some("Please enter a display name".to_string()));
            return;
        }
        
        set_adding.set(true);
        set_error.set(None);
        
        let domain_id = domain_id_clone.clone();
        let domain_name = domain_name_clone.clone();
        let display = display_name.get();
        let set_adding = set_adding.clone();
        let set_error = set_error.clone();
        let set_success = set_success.clone();
        let set_local_part = set_local_part.clone();
        let set_display_name = set_display_name.clone();
        let on_close = on_close_success.clone();
        let on_mailbox_added = on_mailbox_added_clone.clone();
        
        spawn_local(async move {
            let display_opt = if display.is_empty() { None } else { Some(display.as_str()) };
            log::info!("Calling API to create mailbox: {}@{}", local, domain_name);
            
            match api::create_mailbox(&domain_id, &local, display_opt).await {
                Ok(_) => {
                    log::info!("Mailbox created successfully");
                    let email = format!("{}@{}", local, domain_name);
                    
                    let mailbox_data = MailboxData {
                        id: format!("{}-{}", domain_id, local),
                        email_address: email.clone(),
                        display_name: if display.is_empty() { None } else { Some(display.clone()) },
                        agent_role: Some("inbox".to_string()),
                        is_active: true,
                    };
                    
                    set_success.set(Some(format!("Mailbox '{}' created!", email)));
                    set_local_part.set(String::new());
                    set_display_name.set(String::new());
                    
                    on_mailbox_added(mailbox_data);
                    
                    gloo_timers::callback::Timeout::new(1500, move || {
                        on_close();
                    }).forget();
                }
                Err(e) => {
                    log::error!("Failed to create mailbox: {}", e);
                    set_error.set(Some(format!("Failed to create mailbox: {}", e)));
                }
            }
            set_adding.set(false);
        });
    };
    
    let handle_add_clone = handle_add.clone();
    
    view! {
        <div class="modal-overlay" on:click=move |_| if !adding.get() { on_close_overlay() }>
            <div class="modal" on:click=|e| e.stop_propagation()>
                <div class="modal-header">
                    <h2>"Add Mailbox"</h2>
                    <button class="modal-close" on:click=move |_| on_close_x() disabled=move || adding.get()>"×"</button>
                </div>
                <div class="modal-body">
                    // Success message
                    <Show when=move || success.get().is_some()>
                        <div class="alert alert-success" style="margin-bottom: 16px;">
                            <svg viewBox="0 0 24 24" fill="currentColor" class="alert-icon">
                                <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z"/>
                            </svg>
                            {move || success.get().unwrap_or_default()}
                        </div>
                    </Show>
                    
                    // Error message
                    <Show when=move || error.get().is_some()>
                        <div class="alert alert-error" style="margin-bottom: 16px;">
                            <svg viewBox="0 0 24 24" fill="currentColor" class="alert-icon">
                                <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-2h2v2zm0-4h-2V7h2v6z"/>
                            </svg>
                            {move || error.get().unwrap_or_default()}
                        </div>
                    </Show>
                    
                    <div class="form-group">
                        <label>"Email Address"</label>
                        <div class="email-input-group" style="display: flex; align-items: center; gap: 8px;">
                            <input 
                                type="text" 
                                class="form-input" 
                                id="email-input"
                                placeholder="Support" 
                                prop:value=move || local_part.get() 
                                on:input=move |e| set_local_part.set(event_target_value(&e))
                                on:keydown=move |e: ev::KeyboardEvent| {
                                    if e.key() == "Enter" && !adding.get() {
                                        e.prevent_default();
                                        // Focus the display name input
                                        if let Some(window) = web_sys::window() {
                                            if let Some(document) = window.document() {
                                                if let Some(el) = document.get_element_by_id("display-name-input") {
                                                    if let Ok(input) = el.dyn_into::<web_sys::HtmlInputElement>() {
                                                        let _ = input.focus();
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                disabled=move || adding.get()
                            />
                            <span style="color: #666;">"@"{domain_name.clone()}</span>
                        </div>
                    </div>
                    
                    <div class="form-group">
                        <label>"Display Name"</label>
                        <input 
                            type="text" 
                            class="form-input" 
                            id="display-name-input"
                            placeholder="e.g. Support Team, Legal Department" 
                            prop:value=move || display_name.get() 
                            on:input=move |e| set_display_name.set(event_target_value(&e))
                            on:keydown=move |e: ev::KeyboardEvent| {
                                if e.key() == "Enter" && !adding.get() {
                                    e.prevent_default();
                                    handle_add_clone(ev::MouseEvent::new("click").unwrap());
                                }
                            }
                            disabled=move || adding.get()
                            required=true
                        />
                        <small style="color: #666; font-size: 12px; margin-top: 4px; display: block;">"The name shown in email headers and mailbox list"</small>
                    </div>
                </div>
                <div class="modal-footer">
                    <button 
                        class="btn btn-secondary" 
                        on:click=move |_| on_close_cancel()
                        disabled=move || adding.get()
                    >
                        "Cancel"
                    </button>
                    <button 
                        class="btn btn-primary" 
                        on:click=handle_add
                        disabled=move || adding.get()
                    >
                        {move || if adding.get() { "Adding..." } else { "Add" }}
                    </button>
                </div>
            </div>
        </div>
    }
}

/// Delete mailbox confirmation modal
#[component]
fn DeleteMailboxModal(
    mailbox_id: String,
    mailbox_email: String,
    on_close: impl Fn() + Clone + 'static,
    on_deleted: impl Fn(String) + Clone + 'static,
) -> impl IntoView {
    let (deleting, set_deleting) = create_signal(false);
    let (error, set_error) = create_signal::<Option<String>>(None);
    
    let on_close_overlay = on_close.clone();
    let on_close_cancel = on_close.clone();
    let on_close_x = on_close.clone();
    
    let mailbox_id_for_delete = mailbox_id.clone();
    let on_deleted_clone = on_deleted.clone();
    
    let handle_delete = move |_| {
        let mailbox_id = mailbox_id_for_delete.clone();
        let set_deleting = set_deleting.clone();
        let set_error = set_error.clone();
        let on_deleted = on_deleted_clone.clone();
        
        set_deleting.set(true);
        set_error.set(None);
        
        spawn_local(async move {
            match api::delete_mailbox(&mailbox_id).await {
                Ok(_) => {
                    on_deleted(mailbox_id);
                }
                Err(e) => {
                    log::error!("Failed to delete mailbox: {}", e);
                    set_error.set(Some(format!("Failed to delete: {}", e)));
                    set_deleting.set(false);
                }
            }
        });
    };
    
    view! {
        <div class="modal-overlay" on:click=move |_| if !deleting.get() { on_close_overlay() }>
            <div class="modal modal-sm" on:click=|e| e.stop_propagation()>
                <div class="modal-header">
                    <h2>"Delete Mailbox"</h2>
                    <button class="modal-close" on:click=move |_| on_close_x() disabled=move || deleting.get()>"×"</button>
                </div>
                <div class="modal-body">
                    <Show when=move || error.get().is_some()>
                        <div class="alert alert-error" style="margin-bottom: 16px;">
                            {move || error.get().unwrap_or_default()}
                        </div>
                    </Show>
                    
                    <p style="margin-bottom: 16px;">"Are you sure you want to delete this mailbox?"</p>
                    <p style="font-weight: 600; color: var(--color-text);">{mailbox_email.clone()}</p>
                    <p style="margin-top: 12px; color: var(--color-text-secondary); font-size: 14px;">"This action cannot be undone. All emails in this mailbox will be permanently deleted."</p>
                </div>
                <div class="modal-footer">
                    <button 
                        class="btn btn-secondary" 
                        on:click=move |_| on_close_cancel()
                        disabled=move || deleting.get()
                    >
                        "Cancel"
                    </button>
                    <button 
                        class="btn btn-danger" 
                        on:click=handle_delete
                        disabled=move || deleting.get()
                    >
                        {move || if deleting.get() { "Deleting..." } else { "Delete" }}
                    </button>
                </div>
            </div>
        </div>
    }
}

/// Delete domain confirmation modal
#[component]
fn DeleteDomainModal(
    domain_id: String,
    domain_name: String,
    on_close: impl Fn() + Clone + 'static,
) -> impl IntoView {
    let (deleting, set_deleting) = create_signal(false);
    let (error, set_error) = create_signal::<Option<String>>(None);
    
    let on_close_overlay = on_close.clone();
    let on_close_cancel = on_close.clone();
    let on_close_x = on_close.clone();
    
    let domain_id_for_delete = domain_id.clone();
    
    let handle_delete = move |_| {
        let domain_id = domain_id_for_delete.clone();
        let set_deleting = set_deleting.clone();
        let set_error = set_error.clone();
        
        set_deleting.set(true);
        set_error.set(None);
        
        spawn_local(async move {
            match api::delete_domain(&domain_id).await {
                Ok(_) => {
                    // Navigate back to domains list
                    if let Some(window) = web_sys::window() {
                        let _ = window.location().set_href("/tenants");
                    }
                }
                Err(e) => {
                    log::error!("Failed to delete domain: {}", e);
                    set_error.set(Some(format!("Failed to delete: {}", e)));
                    set_deleting.set(false);
                }
            }
        });
    };
    
    view! {
        <div class="modal-overlay" on:click=move |_| if !deleting.get() { on_close_overlay() }>
            <div class="modal modal-sm" on:click=|e| e.stop_propagation()>
                <div class="modal-header">
                    <h2>"Delete Domain"</h2>
                    <button class="modal-close" on:click=move |_| on_close_x() disabled=move || deleting.get()>"×"</button>
                </div>
                <div class="modal-body">
                    <Show when=move || error.get().is_some()>
                        <div class="alert alert-error" style="margin-bottom: 16px;">
                            {move || error.get().unwrap_or_default()}
                        </div>
                    </Show>
                    
                    <p style="margin-bottom: 16px;">"Are you sure you want to delete this domain?"</p>
                    <p style="font-weight: 600; color: var(--color-text);">{domain_name.clone()}</p>
                    <p style="margin-top: 12px; color: var(--color-danger); font-size: 14px; font-weight: 500;">"⚠️ This will also delete all mailboxes associated with this domain."</p>
                    <p style="margin-top: 8px; color: var(--color-text-secondary); font-size: 14px;">"This action cannot be undone."</p>
                </div>
                <div class="modal-footer">
                    <button 
                        class="btn btn-secondary" 
                        on:click=move |_| on_close_cancel()
                        disabled=move || deleting.get()
                    >
                        "Cancel"
                    </button>
                    <button 
                        class="btn btn-danger" 
                        on:click=handle_delete
                        disabled=move || deleting.get()
                    >
                        {move || if deleting.get() { "Deleting..." } else { "Delete Domain" }}
                    </button>
                </div>
            </div>
        </div>
    }
}

// Sample data removed - now using real API data
