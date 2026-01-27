//! Domains Page - Domain and Mailbox management

use leptos::*;
use wasm_bindgen_futures::spawn_local;
use crate::api;

#[derive(Clone, Debug, PartialEq)]
struct Domain {
    id: String,
    name: String,
    display_name: String,
    enabled: bool,
    primary: bool,
    mx_verified: bool,
    spf_verified: bool,
    dkim_verified: bool,
    dmarc_verified: bool,
    mailboxes: Vec<Mailbox>,
}

#[derive(Clone, Debug, PartialEq)]
struct Mailbox {
    id: String,
    email: String,
    display_name: String,
    enabled: bool,
}

#[component]
pub fn DomainsPage() -> impl IntoView {
    // State
    let (domains, set_domains) = create_signal::<Vec<Domain>>(vec![]);
    let (loading, set_loading) = create_signal(true);
    let (error, set_error) = create_signal::<Option<String>>(None);
    
    // Add domain modal state
    let (show_add_modal, set_show_add_modal) = create_signal(false);
    let (new_domain, set_new_domain) = create_signal(String::new());
    let (adding, set_adding) = create_signal(false);
    let (add_error, set_add_error) = create_signal::<Option<String>>(None);
    let (add_success, set_add_success) = create_signal::<Option<String>>(None);
    
    // Delete domain confirmation state
    let (show_delete_modal, set_show_delete_modal) = create_signal(false);
    let (domain_to_delete, set_domain_to_delete) = create_signal::<Option<Domain>>(None);
    let (deleting, set_deleting) = create_signal(false);
    
    // Add mailbox modal state
    let (show_add_mailbox_modal, set_show_add_mailbox_modal) = create_signal(false);
    let (mailbox_domain_id, set_mailbox_domain_id) = create_signal::<Option<String>>(None);
    let (mailbox_domain_name, set_mailbox_domain_name) = create_signal(String::new());
    let (new_mailbox_local, set_new_mailbox_local) = create_signal(String::new());
    let (new_mailbox_name, set_new_mailbox_name) = create_signal(String::new());
    let (adding_mailbox, set_adding_mailbox) = create_signal(false);
    let (add_mailbox_error, set_add_mailbox_error) = create_signal::<Option<String>>(None);
    
    // Delete mailbox confirmation state
    let (show_delete_mailbox_modal, set_show_delete_mailbox_modal) = create_signal(false);
    let (mailbox_to_delete, set_mailbox_to_delete) = create_signal::<Option<(String, Mailbox)>>(None);
    let (deleting_mailbox, set_deleting_mailbox) = create_signal(false);
    
    // Expanded domains state
    let (expanded_domains, set_expanded_domains) = create_signal::<Vec<String>>(vec![]);
    
    // Fetch domains and mailboxes from API on mount
    {
        let set_domains = set_domains.clone();
        let set_loading = set_loading.clone();
        let set_error = set_error.clone();
        
        spawn_local(async move {
            match api::fetch_domains().await {
                Ok(domain_list) => {
                    let mut domains: Vec<Domain> = vec![];
                    for d in domain_list {
                        // Fetch mailboxes for each domain
                        let mailboxes = match api::fetch_mailboxes_by_domain(&d.id).await {
                            Ok(mbs) => mbs.into_iter().map(|m| Mailbox {
                                id: m.id,
                                email: m.email,
                                display_name: m.display_name.unwrap_or_default(),
                                enabled: m.is_active,
                            }).collect(),
                            Err(_) => vec![],
                        };
                        
                        domains.push(Domain {
                            id: d.id,
                            name: d.name,
                            display_name: d.display_name,
                            enabled: d.enabled,
                            primary: d.primary,
                            mx_verified: d.mx_verified,
                            spf_verified: d.spf_verified,
                            dkim_verified: d.dkim_verified,
                            dmarc_verified: d.dmarc_verified,
                            mailboxes,
                        });
                    }
                    set_domains.set(domains);
                }
                Err(e) => {
                    log::error!("Failed to fetch domains: {}", e);
                    set_error.set(Some(format!("Failed to load domains: {}", e)));
                }
            }
            set_loading.set(false);
        });
    }

    // Add domain handler
    let add_domain = move |_| {
        let domain_name = new_domain.get().trim().to_string();
        log::info!("Add domain clicked, domain_name: '{}'", domain_name);
        
        if domain_name.is_empty() {
            log::warn!("Domain name is empty");
            set_add_error.set(Some("Please enter a domain name".to_string()));
            return;
        }
        
        log::info!("Starting domain creation for: {}", domain_name);
        set_adding.set(true);
        set_add_error.set(None);
        set_add_success.set(None);
        
        let set_domains = set_domains.clone();
        let set_new_domain = set_new_domain.clone();
        let set_show_add_modal = set_show_add_modal.clone();
        let set_adding = set_adding.clone();
        let set_add_error = set_add_error.clone();
        let set_add_success = set_add_success.clone();
        
        spawn_local(async move {
            log::info!("Calling API to create domain: {}", domain_name);
            match api::create_domain(&domain_name).await {
                Ok(new_d) => {
                    log::info!("Domain created successfully: {} (id: {})", new_d.name, new_d.id);
                    set_domains.update(|d| {
                        d.push(Domain {
                            id: new_d.id.clone(),
                            name: new_d.name.clone(),
                            display_name: new_d.display_name,
                            enabled: new_d.enabled,
                            primary: new_d.primary,
                            mx_verified: new_d.mx_verified,
                            spf_verified: new_d.spf_verified,
                            dkim_verified: new_d.dkim_verified,
                            dmarc_verified: new_d.dmarc_verified,
                            mailboxes: vec![],
                        });
                    });
                    set_add_success.set(Some(format!("Domain '{}' added successfully!", new_d.name)));
                    set_new_domain.set(String::new());
                    
                    // Close modal after 1.5 seconds
                    let set_show_add_modal = set_show_add_modal.clone();
                    let set_add_success = set_add_success.clone();
                    gloo_timers::callback::Timeout::new(1500, move || {
                        set_show_add_modal.set(false);
                        set_add_success.set(None);
                    }).forget();
                }
                Err(e) => {
                    log::error!("Failed to create domain: {}", e);
                    set_add_error.set(Some(format!("Failed to add domain: {}", e)));
                }
            }
            set_adding.set(false);
        });
    };
    
    // Add mailbox handler
    let add_mailbox = move |_| {
        let local_part = new_mailbox_local.get();
        let display_name = new_mailbox_name.get();
        let domain_id = match mailbox_domain_id.get() {
            Some(id) => id,
            None => return,
        };
        
        if local_part.is_empty() {
            set_add_mailbox_error.set(Some("Please enter an email address".to_string()));
            return;
        }
        
        set_adding_mailbox.set(true);
        set_add_mailbox_error.set(None);
        
        let set_domains = set_domains.clone();
        let set_show_add_mailbox_modal = set_show_add_mailbox_modal.clone();
        let set_adding_mailbox = set_adding_mailbox.clone();
        let set_add_mailbox_error = set_add_mailbox_error.clone();
        let set_new_mailbox_local = set_new_mailbox_local.clone();
        let set_new_mailbox_name = set_new_mailbox_name.clone();
        let domain_name = mailbox_domain_name.get();
        
        spawn_local(async move {
            let display = if display_name.is_empty() { None } else { Some(display_name.as_str()) };
            match api::create_mailbox(&domain_id, &local_part, display).await {
                Ok(_) => {
                    let email = format!("{}@{}", local_part, domain_name);
                    set_domains.update(|domains| {
                        if let Some(domain) = domains.iter_mut().find(|d| d.id == domain_id) {
                            domain.mailboxes.push(Mailbox {
                                id: format!("{}-{}", domain_id, local_part),
                                email,
                                display_name: display.unwrap_or(&local_part).to_string(),
                                enabled: true,
                            });
                        }
                    });
                    set_new_mailbox_local.set(String::new());
                    set_new_mailbox_name.set(String::new());
                    set_show_add_mailbox_modal.set(false);
                }
                Err(e) => {
                    set_add_mailbox_error.set(Some(format!("Failed to create mailbox: {}", e)));
                }
            }
            set_adding_mailbox.set(false);
        });
    };

    view! {
        <div class="page domains-page">
            <div class="page-header">
                <h1 class="page-title">"Domains"</h1>
                <button class="btn btn-primary" on:click=move |_| set_show_add_modal.set(true)>
                    <svg viewBox="0 0 24 24" fill="currentColor" class="btn-icon">
                        <path d="M19 13h-6v6h-2v-6H5v-2h6V5h2v6h6v2z"/>
                    </svg>
                    "Add Domain"
                </button>
            </div>
            
            // Loading state
            <Show when=move || loading.get()>
                <div class="loading-state">
                    <p>"Loading domains..."</p>
                </div>
            </Show>
            
            // Error state
            <Show when=move || error.get().is_some()>
                <div class="alert alert-error">
                    {move || error.get().unwrap_or_default()}
                </div>
            </Show>
            
            // Domain count
            <div class="domains-summary">
                <span class="domain-count">{move || domains.get().len()}" domains configured"</span>
            </div>
            
            <div class="domains-list">
                <For
                    each=move || domains.get()
                    key=|d| d.id.clone()
                    children=move |domain| {
                        let domain_id = domain.id.clone();
                        let domain_id_expand = domain.id.clone();
                        let domain_id_mailbox = domain.id.clone();
                        let domain_name_mailbox = domain.name.clone();
                        let domain_for_delete = domain.clone();
                        
                        let is_expanded = move || expanded_domains.get().contains(&domain_id_expand);
                        
                        let toggle_expand = {
                            let domain_id = domain_id.clone();
                            move |_| {
                                set_expanded_domains.update(|ids| {
                                    if ids.contains(&domain_id) {
                                        ids.retain(|id| id != &domain_id);
                                    } else {
                                        ids.push(domain_id.clone());
                                    }
                                });
                            }
                        };
                        
                        let on_delete = {
                            let domain = domain_for_delete.clone();
                            move |_| {
                                set_domain_to_delete.set(Some(domain.clone()));
                                set_show_delete_modal.set(true);
                            }
                        };
                        
                        let on_add_mailbox = {
                            let domain_id = domain_id_mailbox.clone();
                            let domain_name = domain_name_mailbox.clone();
                            move |_| {
                                set_mailbox_domain_id.set(Some(domain_id.clone()));
                                set_mailbox_domain_name.set(domain_name.clone());
                                set_new_mailbox_local.set(String::new());
                                set_new_mailbox_name.set(String::new());
                                set_add_mailbox_error.set(None);
                                set_show_add_mailbox_modal.set(true);
                            }
                        };
                        
                        let on_delete_mailbox = {
                            let set_mailbox_to_delete = set_mailbox_to_delete.clone();
                            let set_show_delete_mailbox_modal = set_show_delete_mailbox_modal.clone();
                            move |domain_id: String, mailbox: Mailbox| {
                                set_mailbox_to_delete.set(Some((domain_id, mailbox)));
                                set_show_delete_mailbox_modal.set(true);
                            }
                        };
                        
                        view! {
                            <DomainCard 
                                domain=domain.clone()
                                is_expanded=is_expanded
                                on_toggle=toggle_expand
                                on_delete=on_delete
                                on_add_mailbox=on_add_mailbox
                                on_delete_mailbox=on_delete_mailbox
                            />
                        }
                    }
                />
            </div>
            
            // Add Domain Modal
            <Show when=move || show_add_modal.get()>
                <div class="modal-overlay" on:click=move |_| set_show_add_modal.set(false)>
                    <div class="modal" on:click=|e| e.stop_propagation()>
                        <div class="modal-header">
                            <h2>"Add New Domain"</h2>
                            <button class="modal-close" on:click=move |_| set_show_add_modal.set(false)>
                                "×"
                            </button>
                        </div>
                        <div class="modal-body">
                            // Success message
                            <Show when=move || add_success.get().is_some()>
                                <div class="alert alert-success">
                                    <svg viewBox="0 0 24 24" fill="currentColor" class="alert-icon">
                                        <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z"/>
                                    </svg>
                                    {move || add_success.get().unwrap_or_default()}
                                </div>
                            </Show>
                            
                            // Error message
                            <Show when=move || add_error.get().is_some()>
                                <div class="alert alert-error">
                                    <svg viewBox="0 0 24 24" fill="currentColor" class="alert-icon">
                                        <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-2h2v2zm0-4h-2V7h2v6z"/>
                                    </svg>
                                    {move || add_error.get().unwrap_or_default()}
                                </div>
                            </Show>
                            
                            <div class="form-group">
                                <label for="domain-name">"Domain Name"</label>
                                <input
                                    type="text"
                                    id="domain-name"
                                    class="form-input"
                                    placeholder="example.com"
                                    prop:value=move || new_domain.get()
                                    on:input=move |e| set_new_domain.set(event_target_value(&e))
                                    disabled=move || adding.get()
                                />
                                <p class="form-hint">"Enter the domain you want to receive emails for"</p>
                            </div>
                        </div>
                        <div class="modal-footer">
                            <button 
                                class="btn btn-secondary" 
                                on:click=move |_| set_show_add_modal.set(false)
                                disabled=move || adding.get()
                            >
                                "Cancel"
                            </button>
                            <button 
                                class="btn btn-primary" 
                                on:click=add_domain
                                disabled=move || adding.get()
                            >
                                {move || if adding.get() { "Adding..." } else { "Add Domain" }}
                            </button>
                        </div>
                    </div>
                </div>
            </Show>
            
            // Delete Confirmation Modal
            <Show when=move || show_delete_modal.get()>
                <div class="modal-overlay" on:click=move |_| {
                    if !deleting.get() {
                        set_show_delete_modal.set(false);
                        set_domain_to_delete.set(None);
                    }
                }>
                    <div class="modal modal-danger" on:click=|e| e.stop_propagation()>
                        <div class="modal-header">
                            <h2>"Delete Domain"</h2>
                            <button 
                                class="modal-close" 
                                on:click=move |_| {
                                    set_show_delete_modal.set(false);
                                    set_domain_to_delete.set(None);
                                }
                                disabled=move || deleting.get()
                            >
                                "×"
                            </button>
                        </div>
                        <div class="modal-body">
                            <div class="alert alert-warning">
                                <svg viewBox="0 0 24 24" fill="currentColor" class="alert-icon">
                                    <path d="M1 21h22L12 2 1 21zm12-3h-2v-2h2v2zm0-4h-2v-4h2v4z"/>
                                </svg>
                                <div>
                                    <strong>"Warning: This action cannot be undone!"</strong>
                                    <p>"All mailboxes and data associated with this domain will be permanently deleted."</p>
                                </div>
                            </div>
                            <p class="delete-confirm-text">
                                "Are you sure you want to delete "
                                <strong>{move || domain_to_delete.get().map(|d| d.name).unwrap_or_default()}</strong>
                                "?"
                            </p>
                        </div>
                        <div class="modal-footer">
                            <button 
                                class="btn btn-secondary" 
                                on:click=move |_| {
                                    set_show_delete_modal.set(false);
                                    set_domain_to_delete.set(None);
                                }
                                disabled=move || deleting.get()
                            >
                                "Cancel"
                            </button>
                            <button 
                                class="btn btn-danger" 
                                on:click=move |_| {
                                    if let Some(domain) = domain_to_delete.get() {
                                        set_deleting.set(true);
                                        let domain_id = domain.id.clone();
                                        let set_domains = set_domains.clone();
                                        let set_deleting = set_deleting.clone();
                                        let set_show_delete_modal = set_show_delete_modal.clone();
                                        let set_domain_to_delete = set_domain_to_delete.clone();
                                        
                                        spawn_local(async move {
                                            match api::delete_domain(&domain_id).await {
                                                Ok(_) => {
                                                    log::info!("Domain deleted: {}", domain_id);
                                                    set_domains.update(|d| {
                                                        d.retain(|dom| dom.id != domain_id);
                                                    });
                                                    set_show_delete_modal.set(false);
                                                    set_domain_to_delete.set(None);
                                                }
                                                Err(e) => {
                                                    log::error!("Failed to delete domain: {}", e);
                                                }
                                            }
                                            set_deleting.set(false);
                                        });
                                    }
                                }
                                disabled=move || deleting.get()
                            >
                                {move || if deleting.get() { "Deleting..." } else { "Delete Domain" }}
                            </button>
                        </div>
                    </div>
                </div>
            </Show>
            
            // Add Mailbox Modal
            <Show when=move || show_add_mailbox_modal.get()>
                <div class="modal-overlay" on:click=move |_| set_show_add_mailbox_modal.set(false)>
                    <div class="modal" on:click=|e| e.stop_propagation()>
                        <div class="modal-header">
                            <h2>"Add Mailbox"</h2>
                            <button class="modal-close" on:click=move |_| set_show_add_mailbox_modal.set(false)>
                                "×"
                            </button>
                        </div>
                        <div class="modal-body">
                            <Show when=move || add_mailbox_error.get().is_some()>
                                <div class="alert alert-error">
                                    {move || add_mailbox_error.get().unwrap_or_default()}
                                </div>
                            </Show>
                            
                            <div class="form-group">
                                <label>"Email Address"</label>
                                <div class="email-input-group">
                                    <input
                                        type="text"
                                        class="form-input"
                                        placeholder="username"
                                        prop:value=move || new_mailbox_local.get()
                                        on:input=move |e| set_new_mailbox_local.set(event_target_value(&e))
                                        disabled=move || adding_mailbox.get()
                                    />
                                    <span class="email-at">"@"</span>
                                    <span class="email-domain">{move || mailbox_domain_name.get()}</span>
                                </div>
                            </div>
                            
                            <div class="form-group">
                                <label>"Display Name (optional)"</label>
                                <input
                                    type="text"
                                    class="form-input"
                                    placeholder="John Doe"
                                    prop:value=move || new_mailbox_name.get()
                                    on:input=move |e| set_new_mailbox_name.set(event_target_value(&e))
                                    disabled=move || adding_mailbox.get()
                                />
                            </div>
                        </div>
                        <div class="modal-footer">
                            <button 
                                class="btn btn-secondary" 
                                on:click=move |_| set_show_add_mailbox_modal.set(false)
                                disabled=move || adding_mailbox.get()
                            >
                                "Cancel"
                            </button>
                            <button 
                                class="btn btn-primary" 
                                on:click=add_mailbox
                                disabled=move || adding_mailbox.get()
                            >
                                {move || if adding_mailbox.get() { "Creating..." } else { "Create Mailbox" }}
                            </button>
                        </div>
                    </div>
                </div>
            </Show>
            
            // Delete Mailbox Confirmation Modal
            <Show when=move || show_delete_mailbox_modal.get()>
                <div class="modal-overlay" on:click=move |_| {
                    if !deleting_mailbox.get() {
                        set_show_delete_mailbox_modal.set(false);
                        set_mailbox_to_delete.set(None);
                    }
                }>
                    <div class="modal modal-danger" on:click=|e| e.stop_propagation()>
                        <div class="modal-header">
                            <h2>"Delete Mailbox"</h2>
                            <button 
                                class="modal-close" 
                                on:click=move |_| {
                                    set_show_delete_mailbox_modal.set(false);
                                    set_mailbox_to_delete.set(None);
                                }
                                disabled=move || deleting_mailbox.get()
                            >
                                "×"
                            </button>
                        </div>
                        <div class="modal-body">
                            <div class="alert alert-warning">
                                <svg viewBox="0 0 24 24" fill="currentColor" class="alert-icon">
                                    <path d="M1 21h22L12 2 1 21zm12-3h-2v-2h2v2zm0-4h-2v-4h2v4z"/>
                                </svg>
                                <div>
                                    <strong>"Warning: This action cannot be undone!"</strong>
                                    <p>"All emails in this mailbox will be permanently deleted."</p>
                                </div>
                            </div>
                            <p class="delete-confirm-text">
                                "Are you sure you want to delete "
                                <strong>{move || mailbox_to_delete.get().map(|(_, m)| m.email).unwrap_or_default()}</strong>
                                "?"
                            </p>
                        </div>
                        <div class="modal-footer">
                            <button 
                                class="btn btn-secondary" 
                                on:click=move |_| {
                                    set_show_delete_mailbox_modal.set(false);
                                    set_mailbox_to_delete.set(None);
                                }
                                disabled=move || deleting_mailbox.get()
                            >
                                "Cancel"
                            </button>
                            <button 
                                class="btn btn-danger" 
                                on:click=move |_| {
                                    if let Some((domain_id, mailbox)) = mailbox_to_delete.get() {
                                        set_deleting_mailbox.set(true);
                                        let mailbox_id = mailbox.id.clone();
                                        let set_domains = set_domains.clone();
                                        let set_deleting_mailbox = set_deleting_mailbox.clone();
                                        let set_show_delete_mailbox_modal = set_show_delete_mailbox_modal.clone();
                                        let set_mailbox_to_delete = set_mailbox_to_delete.clone();
                                        
                                        spawn_local(async move {
                                            match api::delete_mailbox(&mailbox_id).await {
                                                Ok(_) => {
                                                    set_domains.update(|domains| {
                                                        if let Some(domain) = domains.iter_mut().find(|d| d.id == domain_id) {
                                                            domain.mailboxes.retain(|m| m.id != mailbox_id);
                                                        }
                                                    });
                                                    set_show_delete_mailbox_modal.set(false);
                                                    set_mailbox_to_delete.set(None);
                                                }
                                                Err(e) => {
                                                    log::error!("Failed to delete mailbox: {}", e);
                                                }
                                            }
                                            set_deleting_mailbox.set(false);
                                        });
                                    }
                                }
                                disabled=move || deleting_mailbox.get()
                            >
                                {move || if deleting_mailbox.get() { "Deleting..." } else { "Delete Mailbox" }}
                            </button>
                        </div>
                    </div>
                </div>
            </Show>
        </div>
    }
}

#[component]
fn DomainCard<F1, F2, F3, F4, F5>(
    domain: Domain,
    is_expanded: F1,
    on_toggle: F2,
    on_delete: F3,
    on_add_mailbox: F4,
    on_delete_mailbox: F5,
) -> impl IntoView 
where
    F1: Fn() -> bool + Clone + 'static,
    F2: Fn(ev::MouseEvent) + 'static,
    F3: Fn(ev::MouseEvent) + 'static,
    F4: Fn(ev::MouseEvent) + 'static,
    F5: Fn(String, Mailbox) + Clone + 'static,
{
    let verification_count = [
        domain.mx_verified,
        domain.spf_verified,
        domain.dkim_verified,
        domain.dmarc_verified,
    ].iter().filter(|&&v| v).count();
    
    let domain_id = domain.id.clone();
    let mailbox_count = domain.mailboxes.len();
    let mailboxes = domain.mailboxes.clone();

    let is_expanded_icon = is_expanded.clone();
    let is_expanded_mailboxes = is_expanded;
    
    view! {
        <div class="domain-card">
            <div class="domain-card-header">
                <div class="domain-info">
                    <button class="expand-btn" on:click=on_toggle>
                        <svg viewBox="0 0 24 24" fill="currentColor" class=move || if is_expanded_icon() { "expand-icon expanded" } else { "expand-icon" }>
                            <path d="M8.59 16.59L13.17 12 8.59 7.41 10 6l6 6-6 6-1.41-1.41z"/>
                        </svg>
                    </button>
                    <h3 class="domain-name">{&domain.name}</h3>
                    <span class="badge badge-info">{mailbox_count}" mailboxes"</span>
                    <span class=if domain.enabled { "badge badge-success" } else { "badge badge-warning" }>
                        {if domain.enabled { "Active" } else { "Disabled" }}
                    </span>
                    {if domain.primary {
                        view! { <span class="badge badge-primary">"Primary"</span> }.into_view()
                    } else {
                        view! {}.into_view()
                    }}
                </div>
                <div class="domain-actions">
                    <a href=format!("/dns-wizard?domain={}", domain_id) class="btn btn-sm btn-secondary">
                        "DNS"
                    </a>
                    <button class="btn btn-sm btn-danger" on:click=on_delete>
                        "Delete"
                    </button>
                </div>
            </div>
            
            <div class="domain-verification">
                <div class="verification-progress">
                    <div class="progress-bar">
                        <div 
                            class="progress-fill" 
                            style=format!("width: {}%", verification_count * 25)
                        ></div>
                    </div>
                    <span class="progress-text">{verification_count}"/4 DNS records"</span>
                </div>
                
                <div class="verification-items">
                    <VerificationItem label="MX" verified=domain.mx_verified/>
                    <VerificationItem label="SPF" verified=domain.spf_verified/>
                    <VerificationItem label="DKIM" verified=domain.dkim_verified/>
                    <VerificationItem label="DMARC" verified=domain.dmarc_verified/>
                </div>
            </div>
            
            // Expandable mailbox section - render conditionally based on is_expanded
            <div class="domain-mailboxes" style:display=move || if is_expanded_mailboxes() { "block" } else { "none" }>
                <div class="mailboxes-header">
                    <h4>"Mailboxes"</h4>
                    <button class="btn btn-sm btn-primary" on:click=on_add_mailbox>
                        <svg viewBox="0 0 24 24" fill="currentColor" class="btn-icon-sm">
                            <path d="M19 13h-6v6h-2v-6H5v-2h6V5h2v6h6v2z"/>
                        </svg>
                        "Add Mailbox"
                    </button>
                </div>
                
                {if mailboxes.is_empty() {
                    view! {
                        <div class="empty-mailboxes">
                            <p>"No mailboxes configured. Add one to start receiving emails."</p>
                        </div>
                    }.into_view()
                } else {
                    let domain_id_inner = domain.id.clone();
                    view! {
                        <div class="mailbox-list">
                            {mailboxes.into_iter().map(|mailbox| {
                                let email = mailbox.email.clone();
                                let display_name = mailbox.display_name.clone();
                                let enabled = mailbox.enabled;
                                let mailbox_clone = mailbox.clone();
                                let domain_id = domain_id_inner.clone();
                                let on_delete = on_delete_mailbox.clone();
                                view! {
                                    <div class="mailbox-item">
                                        <div class="mailbox-info">
                                            <svg viewBox="0 0 24 24" fill="currentColor" class="mailbox-icon">
                                                <path d="M20 4H4c-1.1 0-1.99.9-1.99 2L2 18c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V6c0-1.1-.9-2-2-2zm0 4l-8 5-8-5V6l8 5 8-5v2z"/>
                                            </svg>
                                            <div>
                                                <span class="mailbox-email">{email}</span>
                                                <span class="mailbox-name">{display_name}</span>
                                            </div>
                                        </div>
                                        <div class="mailbox-actions">
                                            <span class=if enabled { "badge badge-success badge-sm" } else { "badge badge-warning badge-sm" }>
                                                {if enabled { "Active" } else { "Disabled" }}
                                            </span>
                                            <button 
                                                class="btn btn-xs btn-danger"
                                                on:click=move |_| {
                                                    on_delete(domain_id.clone(), mailbox_clone.clone());
                                                }
                                            >
                                                "Delete"
                                            </button>
                                        </div>
                                    </div>
                                }
                            }).collect_view()}
                        </div>
                    }.into_view()
                }}
            </div>
        </div>
    }
}

#[component]
fn VerificationItem(label: &'static str, verified: bool) -> impl IntoView {
    view! {
        <div class=if verified { "verification-item verified" } else { "verification-item" }>
            <span class="verification-icon">
                {if verified {
                    view! { <svg viewBox="0 0 24 24" fill="currentColor"><path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z"/></svg> }.into_view()
                } else {
                    view! { <svg viewBox="0 0 24 24" fill="currentColor"><path d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z"/></svg> }.into_view()
                }}
            </span>
            <span class="verification-label">{label}</span>
        </div>
    }
}
