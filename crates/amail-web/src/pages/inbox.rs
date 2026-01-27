//! Inbox Page - Gmail-like email interface
//!
//! Table of Contents:
//! 1. InboxPage - Main inbox view
//! 2. EmailList - Email list with selection
//! 3. EmailRow - Individual email row
//! 4. EmailView - Email detail view
//! 5. ComposeModal - New email composer

use leptos::*;
use wasm_bindgen_futures::spawn_local;
use crate::api::{self, InboxEmailResponse, MailboxResponse, DomainResponse};

// =============================================================================
// Email Data Types
// =============================================================================

#[derive(Clone, Debug, PartialEq)]
pub struct Email {
    pub id: String,
    pub from: EmailContact,
    pub to: Vec<EmailContact>,
    pub subject: String,
    pub preview: String,
    pub body_text: Option<String>,
    pub body_html: Option<String>,
    pub timestamp: String,
    pub is_read: bool,
    pub is_starred: bool,
    pub labels: Vec<String>,
    pub attachments: Vec<Attachment>,
    pub agent_response: Option<AgentResponse>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct EmailContact {
    pub name: Option<String>,
    pub email: String,
}

impl EmailContact {
    pub fn display(&self) -> String {
        self.name.clone().unwrap_or_else(|| self.email.clone())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Attachment {
    pub name: String,
    pub size: u64,
    pub mime_type: String,
    /// P2P attachment fields
    pub is_p2p: bool,
    pub magnet_link: Option<String>,
    pub transfer_id: Option<String>,
    pub download_status: Option<String>,
    pub progress_percent: Option<f64>,
}

impl Attachment {
    /// Format size for display
    pub fn size_display(&self) -> String {
        const KB: u64 = 1024;
        const MB: u64 = KB * 1024;
        const GB: u64 = MB * 1024;
        
        if self.size >= GB {
            format!("{:.2} GB", self.size as f64 / GB as f64)
        } else if self.size >= MB {
            format!("{:.2} MB", self.size as f64 / MB as f64)
        } else if self.size >= KB {
            format!("{:.2} KB", self.size as f64 / KB as f64)
        } else {
            format!("{} bytes", self.size)
        }
    }
    
    /// Check if download is in progress
    pub fn is_downloading(&self) -> bool {
        self.is_p2p && matches!(self.download_status.as_deref(), Some("Downloading") | Some("Queued"))
    }
    
    /// Check if download is complete
    pub fn is_downloaded(&self) -> bool {
        !self.is_p2p || matches!(self.download_status.as_deref(), Some("Completed"))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct AgentResponse {
    pub agent_name: String,
    pub response: String,
    pub confidence: f32,
    pub timestamp: String,
}

// =============================================================================
// 1. Inbox Page
// =============================================================================

/// Domain data for domain selection
#[derive(Clone, Debug, PartialEq)]
pub struct Domain {
    pub id: String,
    pub display_name: String,
}

/// Mailbox data for domain/mailbox selection
#[derive(Clone, Debug, PartialEq)]
pub struct Mailbox {
    pub id: String,
    pub email: String,
    pub display_name: String,
    pub domain_id: String,
    pub unread_count: usize,
}

#[component]
pub fn InboxPage() -> impl IntoView {
    // Loading and error state
    let (loading, set_loading) = create_signal(true);
    let (error, set_error) = create_signal::<Option<String>>(None);
    
    // Domain and mailbox selection - fetched from API
    let (domains, set_domains) = create_signal::<Vec<Domain>>(vec![]);
    let (mailboxes, set_mailboxes) = create_signal::<Vec<Mailbox>>(vec![]);
    let (selected_domain, set_selected_domain) = create_signal::<Option<String>>(None);
    let (selected_mailbox, set_selected_mailbox) = create_signal::<Option<String>>(None);
    
    // Email state - fetched from API
    let (emails, set_emails) = create_signal::<Vec<Email>>(vec![]);
    let (selected_email, set_selected_email) = create_signal::<Option<Email>>(None);
    let (selected_ids, set_selected_ids) = create_signal::<Vec<String>>(vec![]);
    let (show_compose, set_show_compose) = create_signal(false);
    let (current_folder, set_current_folder) = create_signal("inbox".to_string());
    let (search_query, set_search_query) = create_signal(String::new());
    let (refreshing, set_refreshing) = create_signal(false);
    
    // Helper function to fetch emails
    let fetch_emails = {
        let set_emails = set_emails.clone();
        let set_error = set_error.clone();
        let set_refreshing = set_refreshing.clone();
        
        move |mailbox: Option<String>, folder: String| {
            let set_emails = set_emails.clone();
            let set_error = set_error.clone();
            let set_refreshing = set_refreshing.clone();
            
            spawn_local(async move {
                set_refreshing.set(true);
                
                match api::fetch_inbox(mailbox.as_deref(), Some(&folder), None, Some(50), None).await {
                    Ok(email_list) => {
                        let emails: Vec<Email> = email_list.into_iter().map(|e| Email {
                            id: e.id,
                            from: EmailContact {
                                name: e.from_name,
                                email: e.from_address,
                            },
                            to: e.to.into_iter().map(|addr| EmailContact {
                                name: None,
                                email: addr,
                            }).collect(),
                            subject: e.subject,
                            preview: e.preview.unwrap_or_default(),
                            body_text: None,
                            body_html: None,
                            timestamp: e.timestamp,
                            is_read: e.is_read,
                            is_starred: e.is_starred,
                            labels: e.labels,
                            attachments: vec![],
                            agent_response: e.ai_confidence.map(|conf| AgentResponse {
                                agent_name: "AI Agent".to_string(),
                                response: String::new(),
                                confidence: conf as f32,
                                timestamp: String::new(),
                            }),
                        }).collect();
                        set_emails.set(emails);
                        set_error.set(None);
                    }
                    Err(e) => {
                        log::error!("Failed to fetch inbox: {}", e);
                        set_error.set(Some(format!("Failed to load inbox: {}", e)));
                    }
                }
                
                set_refreshing.set(false);
            });
        }
    };
    
    // Clone for refresh button
    let fetch_emails_refresh = fetch_emails.clone();
    
    // Fetch initial data from API
    {
        let set_domains = set_domains.clone();
        let set_mailboxes = set_mailboxes.clone();
        let set_emails = set_emails.clone();
        let set_loading = set_loading.clone();
        let set_error = set_error.clone();
        
        spawn_local(async move {
            // Fetch domains - store ID and display_name for filtering
            match api::fetch_domains().await {
                Ok(domain_list) => {
                    let domains: Vec<Domain> = domain_list.iter().map(|d| Domain {
                        id: d.id.clone(),
                        display_name: d.display_name.clone(),
                    }).collect();
                    set_domains.set(domains);
                }
                Err(e) => {
                    log::warn!("Failed to fetch domains: {}", e);
                }
            }
            
            // Fetch mailboxes
            match api::fetch_mailboxes().await {
                Ok(mailbox_list) => {
                    let mailboxes: Vec<Mailbox> = mailbox_list.into_iter().map(|m| Mailbox {
                        id: m.id,
                        email: m.email.clone(),
                        display_name: m.display_name.unwrap_or_else(|| m.email.clone()),
                        domain_id: m.domain,
                        unread_count: m.unread_count as usize,
                    }).collect();
                    set_mailboxes.set(mailboxes);
                }
                Err(e) => {
                    log::warn!("Failed to fetch mailboxes: {}", e);
                }
            }
            
            // Fetch inbox emails
            match api::fetch_inbox(None, Some("inbox"), None, Some(50), None).await {
                Ok(email_list) => {
                    let emails: Vec<Email> = email_list.into_iter().map(|e| Email {
                        id: e.id,
                        from: EmailContact {
                            name: e.from_name,
                            email: e.from_address,
                        },
                        to: e.to.into_iter().map(|addr| EmailContact {
                            name: None,
                            email: addr,
                        }).collect(),
                        subject: e.subject,
                        preview: e.preview.unwrap_or_default(),
                        body_text: None,
                        body_html: None,
                        timestamp: e.timestamp,
                        is_read: e.is_read,
                        is_starred: e.is_starred,
                        labels: e.labels,
                        attachments: vec![],
                        agent_response: e.ai_confidence.map(|conf| AgentResponse {
                            agent_name: "AI Agent".to_string(),
                            response: String::new(),
                            confidence: conf as f32,
                            timestamp: String::new(),
                        }),
                    }).collect();
                    set_emails.set(emails);
                }
                Err(e) => {
                    log::error!("Failed to fetch inbox: {}", e);
                    set_error.set(Some(format!("Failed to load inbox: {}", e)));
                }
            }
            
            set_loading.set(false);
        });
    }
    
    // Filter mailboxes by selected domain
    let filtered_mailboxes = move || {
        let domain_id = selected_domain.get();
        mailboxes.get().into_iter()
            .filter(|m| domain_id.is_none() || Some(m.domain_id.clone()) == domain_id)
            .collect::<Vec<_>>()
    };

    // Filter emails based on folder, mailbox, and search
    let filtered_emails = move || {
        let query = search_query.get().to_lowercase();
        let mailbox = selected_mailbox.get();
        emails.get()
            .into_iter()
            .filter(|e| {
                // Filter by mailbox if selected
                let mailbox_match = mailbox.is_none() || 
                    e.to.iter().any(|t| Some(t.email.clone()) == mailbox);
                
                // Filter by search query
                let query_match = if query.is_empty() {
                    true
                } else {
                    e.subject.to_lowercase().contains(&query) ||
                    e.from.email.to_lowercase().contains(&query) ||
                    e.preview.to_lowercase().contains(&query)
                };
                
                mailbox_match && query_match
            })
            .collect::<Vec<_>>()
    };

    let unread_count = move || filtered_emails().iter().filter(|e| !e.is_read).count();
    let total_unread = move || emails.get().iter().filter(|e| !e.is_read).count();

    view! {
        <div class="inbox-page">
            // Sidebar with domain/mailbox selector and folders
            <aside class="inbox-sidebar">
                // Domain selector
                <div class="mailbox-selector">
                    <label class="selector-label">"Domain"</label>
                    <select 
                        class="domain-select"
                        on:change=move |e| {
                            let value = event_target_value(&e);
                            if value.is_empty() {
                                set_selected_domain.set(None);
                                set_selected_mailbox.set(None);
                            } else {
                                set_selected_domain.set(Some(value));
                                set_selected_mailbox.set(None);
                            }
                        }
                    >
                        <option value="">"All Domains"</option>
                        {move || domains.get().into_iter().map(|d| {
                            view! { <option value=d.id.clone()>{d.display_name.clone()}</option> }
                        }).collect_view()}
                    </select>
                </div>
                
                // Mailbox selector
                <div class="mailbox-selector">
                    <label class="selector-label">"Mailbox"</label>
                    <select 
                        class="mailbox-select"
                        on:change=move |e| {
                            let value = event_target_value(&e);
                            if value.is_empty() {
                                set_selected_mailbox.set(None);
                            } else {
                                set_selected_mailbox.set(Some(value));
                            }
                        }
                    >
                        <option value="">"All Mailboxes (" {total_unread} " unread)"</option>
                        {move || filtered_mailboxes().into_iter().map(|m| {
                            let unread = if m.unread_count > 0 { format!(" ({})", m.unread_count) } else { String::new() };
                            view! { 
                                <option value=m.email.clone()>
                                    {m.display_name.clone()}{unread}
                                </option> 
                            }
                        }).collect_view()}
                    </select>
                </div>
                
                <button class="compose-btn" on:click=move |_| set_show_compose.set(true)>
                    <svg viewBox="0 0 24 24" fill="currentColor">
                        <path d="M3 17.25V21h3.75L17.81 9.94l-3.75-3.75L3 17.25zM20.71 7.04c.39-.39.39-1.02 0-1.41l-2.34-2.34c-.39-.39-1.02-.39-1.41 0l-1.83 1.83 3.75 3.75 1.83-1.83z"/>
                    </svg>
                    "Compose"
                </button>
                
                <nav class="folder-list">
                    <FolderItem 
                        name="Inbox" 
                        icon="inbox" 
                        count=unread_count()
                        active=current_folder.get() == "inbox"
                        on_click=move |_| set_current_folder.set("inbox".to_string())
                    />
                    <FolderItem 
                        name="Starred" 
                        icon="star" 
                        count=0
                        active=current_folder.get() == "starred"
                        on_click=move |_| set_current_folder.set("starred".to_string())
                    />
                    <FolderItem 
                        name="Sent" 
                        icon="send" 
                        count=0
                        active=current_folder.get() == "sent"
                        on_click=move |_| set_current_folder.set("sent".to_string())
                    />
                    <FolderItem 
                        name="Drafts" 
                        icon="draft" 
                        count=0
                        active=current_folder.get() == "drafts"
                        on_click=move |_| set_current_folder.set("drafts".to_string())
                    />
                    <FolderItem 
                        name="Spam" 
                        icon="spam" 
                        count=0
                        active=current_folder.get() == "spam"
                        on_click=move |_| set_current_folder.set("spam".to_string())
                    />
                    <FolderItem 
                        name="Trash" 
                        icon="trash" 
                        count=0
                        active=current_folder.get() == "trash"
                        on_click=move |_| set_current_folder.set("trash".to_string())
                    />
                </nav>
                
                <div class="sidebar-labels">
                    <h4>"Labels"</h4>
                    <div class="label-item">
                        <span class="label-dot" style="background: #ea4335"></span>
                        "Important"
                    </div>
                    <div class="label-item">
                        <span class="label-dot" style="background: #34a853"></span>
                        "Work"
                    </div>
                    <div class="label-item">
                        <span class="label-dot" style="background: #4285f4"></span>
                        "Personal"
                    </div>
                </div>
            </aside>
            
            // Main content area
            <div class="inbox-main">
                // Toolbar
                <div class="inbox-toolbar">
                    <div class="toolbar-left">
                        <label class="checkbox-wrapper">
                            <input type="checkbox" 
                                on:change=move |e| {
                                    let checked = event_target_checked(&e);
                                    if checked {
                                        set_selected_ids.set(filtered_emails().iter().map(|e| e.id.clone()).collect());
                                    } else {
                                        set_selected_ids.set(vec![]);
                                    }
                                }
                            />
                            <span class="checkmark"></span>
                        </label>
                        
                        <Show when=move || !selected_ids.get().is_empty()>
                            <div class="bulk-actions">
                                <button class="toolbar-btn" title="Archive">
                                    <svg viewBox="0 0 24 24" fill="currentColor">
                                        <path d="M20.54 5.23l-1.39-1.68C18.88 3.21 18.47 3 18 3H6c-.47 0-.88.21-1.16.55L3.46 5.23C3.17 5.57 3 6.02 3 6.5V19c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2V6.5c0-.48-.17-.93-.46-1.27zM12 17.5L6.5 12H10v-2h4v2h3.5L12 17.5zM5.12 5l.81-1h12l.94 1H5.12z"/>
                                    </svg>
                                </button>
                                <button class="toolbar-btn" title="Delete">
                                    <svg viewBox="0 0 24 24" fill="currentColor">
                                        <path d="M6 19c0 1.1.9 2 2 2h8c1.1 0 2-.9 2-2V7H6v12zM19 4h-3.5l-1-1h-5l-1 1H5v2h14V4z"/>
                                    </svg>
                                </button>
                                <button class="toolbar-btn" title="Mark as read">
                                    <svg viewBox="0 0 24 24" fill="currentColor">
                                        <path d="M20 4H4c-1.1 0-2 .9-2 2v12c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V6c0-1.1-.9-2-2-2zm0 4l-8 5-8-5V6l8 5 8-5v2z"/>
                                    </svg>
                                </button>
                            </div>
                        </Show>
                    </div>
                    
                    <div class="toolbar-center">
                        <div class="search-box">
                            <svg viewBox="0 0 24 24" fill="currentColor" class="search-icon">
                                <path d="M15.5 14h-.79l-.28-.27C15.41 12.59 16 11.11 16 9.5 16 5.91 13.09 3 9.5 3S3 5.91 3 9.5 5.91 16 9.5 16c1.61 0 3.09-.59 4.23-1.57l.27.28v.79l5 4.99L20.49 19l-4.99-5zm-6 0C7.01 14 5 11.99 5 9.5S7.01 5 9.5 5 14 7.01 14 9.5 11.99 14 9.5 14z"/>
                            </svg>
                            <input 
                                type="text" 
                                placeholder="Search mail"
                                prop:value=move || search_query.get()
                                on:input=move |e| set_search_query.set(event_target_value(&e))
                            />
                        </div>
                    </div>
                    
                    <div class="toolbar-right">
                        <button 
                            class=move || if refreshing.get() { "toolbar-btn refreshing" } else { "toolbar-btn" }
                            title="Refresh"
                            disabled=move || refreshing.get()
                            on:click={
                                let fetch_emails = fetch_emails_refresh.clone();
                                move |_| {
                                    let mailbox = selected_mailbox.get();
                                    let folder = current_folder.get();
                                    fetch_emails(mailbox, folder);
                                }
                            }
                        >
                            <svg viewBox="0 0 24 24" fill="currentColor">
                                <path d="M17.65 6.35C16.2 4.9 14.21 4 12 4c-4.42 0-7.99 3.58-7.99 8s3.57 8 7.99 8c3.73 0 6.84-2.55 7.73-6h-2.08c-.82 2.33-3.04 4-5.65 4-3.31 0-6-2.69-6-6s2.69-6 6-6c1.66 0 3.14.69 4.22 1.78L13 11h7V4l-2.35 2.35z"/>
                            </svg>
                        </button>
                    </div>
                </div>
                
                // Email list or detail view
                <div class="inbox-content">
                    <Show 
                        when=move || selected_email.get().is_none()
                        fallback=move || view! {
                            <EmailView 
                                email=selected_email.get().unwrap()
                                on_back=move |_| set_selected_email.set(None)
                            />
                        }
                    >
                        <EmailList 
                            emails=filtered_emails()
                            selected_ids=selected_ids
                            set_selected_ids=set_selected_ids
                            on_select=move |email| set_selected_email.set(Some(email))
                        />
                    </Show>
                </div>
            </div>
            
            // Compose modal
            <Show when=move || show_compose.get()>
                <ComposeModal on_close=move |_| set_show_compose.set(false)/>
            </Show>
        </div>
    }
}

// =============================================================================
// 2. Folder Item
// =============================================================================

#[component]
fn FolderItem(
    name: &'static str,
    icon: &'static str,
    count: usize,
    active: bool,
    on_click: impl Fn(ev::MouseEvent) + 'static,
) -> impl IntoView {
    let has_count = count > 0;
    let icon_svg = match icon {
        "inbox" => view! { <svg viewBox="0 0 24 24" fill="currentColor"><path d="M19 3H4.99c-1.11 0-1.98.89-1.98 2L3 19c0 1.1.88 2 1.99 2H19c1.1 0 2-.9 2-2V5c0-1.11-.9-2-2-2zm0 12h-4c0 1.66-1.35 3-3 3s-3-1.34-3-3H4.99V5H19v10z"/></svg> }.into_view(),
        "star" => view! { <svg viewBox="0 0 24 24" fill="currentColor"><path d="M12 17.27L18.18 21l-1.64-7.03L22 9.24l-7.19-.61L12 2 9.19 8.63 2 9.24l5.46 4.73L5.82 21z"/></svg> }.into_view(),
        "send" => view! { <svg viewBox="0 0 24 24" fill="currentColor"><path d="M2.01 21L23 12 2.01 3 2 10l15 2-15 2z"/></svg> }.into_view(),
        "draft" => view! { <svg viewBox="0 0 24 24" fill="currentColor"><path d="M21.99 8c0-.72-.37-1.35-.94-1.7L12 1 2.95 6.3C2.38 6.65 2 7.28 2 8v10c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2l-.01-10zM12 13L3.74 7.84 12 3l8.26 4.84L12 13z"/></svg> }.into_view(),
        "spam" => view! { <svg viewBox="0 0 24 24" fill="currentColor"><path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-2h2v2zm0-4h-2V7h2v6z"/></svg> }.into_view(),
        "trash" => view! { <svg viewBox="0 0 24 24" fill="currentColor"><path d="M6 19c0 1.1.9 2 2 2h8c1.1 0 2-.9 2-2V7H6v12zM19 4h-3.5l-1-1h-5l-1 1H5v2h14V4z"/></svg> }.into_view(),
        _ => view! { <svg viewBox="0 0 24 24" fill="currentColor"><path d="M20 6h-8l-2-2H4c-1.1 0-1.99.9-1.99 2L2 18c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V8c0-1.1-.9-2-2-2z"/></svg> }.into_view(),
    };

    view! {
        <button 
            class=if active { "folder-item active" } else { "folder-item" }
            on:click=on_click
        >
            <span class="folder-icon">{icon_svg}</span>
            <span class="folder-name">{name}</span>
            <Show when=move || has_count>
                <span class="folder-count">{count}</span>
            </Show>
        </button>
    }
}

// =============================================================================
// 3. Email List
// =============================================================================

#[component]
fn EmailList(
    emails: Vec<Email>,
    selected_ids: ReadSignal<Vec<String>>,
    set_selected_ids: WriteSignal<Vec<String>>,
    on_select: impl Fn(Email) + Clone + 'static,
) -> impl IntoView {
    let is_empty = emails.is_empty();
    let email_rows = emails.into_iter().map(|email| {
        let email_clone = email.clone();
        let email_id = email.id.clone();
        let on_select = on_select.clone();
        view! {
            <EmailRow 
                email=email
                selected=selected_ids.get().contains(&email_id)
                on_toggle_select=move |id| {
                    set_selected_ids.update(|ids| {
                        if ids.contains(&id) {
                            ids.retain(|i| i != &id);
                        } else {
                            ids.push(id);
                        }
                    });
                }
                on_click=move |_| on_select(email_clone.clone())
            />
        }
    }).collect_view();
    
    view! {
        <div class="email-list">
            {if is_empty {
                view! {
                    <div class="empty-state">
                        <svg viewBox="0 0 24 24" fill="currentColor" class="empty-icon">
                            <path d="M20 4H4c-1.1 0-2 .9-2 2v12c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V6c0-1.1-.9-2-2-2zm0 4l-8 5-8-5V6l8 5 8-5v2z"/>
                        </svg>
                        <h3>"No emails"</h3>
                        <p>"Your inbox is empty"</p>
                    </div>
                }.into_view()
            } else {
                email_rows.into_view()
            }}
        </div>
    }
}

// =============================================================================
// 4. Email Row
// =============================================================================

#[component]
fn EmailRow(
    email: Email,
    selected: bool,
    on_toggle_select: impl Fn(String) + 'static,
    on_click: impl Fn(ev::MouseEvent) + 'static,
) -> impl IntoView {
    let email_id = email.id.clone();
    let has_agent_response = email.agent_response.is_some();
    
    view! {
        <div class=move || {
            let mut classes = vec!["email-row"];
            if !email.is_read { classes.push("unread"); }
            if selected { classes.push("selected"); }
            classes.join(" ")
        }>
            <div class="email-row-select" on:click=move |e| {
                e.stop_propagation();
                on_toggle_select(email_id.clone());
            }>
                <label class="checkbox-wrapper">
                    <input type="checkbox" checked=selected/>
                    <span class="checkmark"></span>
                </label>
            </div>
            
            <button class="email-row-star" on:click=|e| e.stop_propagation()>
                <svg viewBox="0 0 24 24" fill=if email.is_starred { "currentColor" } else { "none" } stroke="currentColor">
                    <path d="M12 17.27L18.18 21l-1.64-7.03L22 9.24l-7.19-.61L12 2 9.19 8.63 2 9.24l5.46 4.73L5.82 21z"/>
                </svg>
            </button>
            
            <div class="email-row-content" on:click=on_click>
                <div class="email-row-sender">
                    <span class="sender-name">{email.from.display()}</span>
                    <Show when=move || has_agent_response>
                        <span class="ai-badge" title="AI responded">
                            <svg viewBox="0 0 24 24" fill="currentColor">
                                <path d="M21 10.12h-6.78l2.74-2.82c-2.73-2.7-7.15-2.8-9.88-.1-2.73 2.71-2.73 7.08 0 9.79s7.15 2.71 9.88 0C18.32 15.65 19 14.08 19 12.1h2c0 1.98-.88 4.55-2.64 6.29-3.51 3.48-9.21 3.48-12.72 0-3.5-3.47-3.53-9.11-.02-12.58s9.14-3.47 12.65 0L21 3v7.12z"/>
                            </svg>
                        </span>
                    </Show>
                </div>
                
                <div class="email-row-subject">
                    <span class="subject-text">{&email.subject}</span>
                    <span class="preview-text">" - "{&email.preview}</span>
                </div>
                
                <div class="email-row-meta">
                    <Show when=move || !email.attachments.is_empty()>
                        <svg viewBox="0 0 24 24" fill="currentColor" class="attachment-icon">
                            <path d="M16.5 6v11.5c0 2.21-1.79 4-4 4s-4-1.79-4-4V5c0-1.38 1.12-2.5 2.5-2.5s2.5 1.12 2.5 2.5v10.5c0 .55-.45 1-1 1s-1-.45-1-1V6H10v9.5c0 1.38 1.12 2.5 2.5 2.5s2.5-1.12 2.5-2.5V5c0-2.21-1.79-4-4-4S7 2.79 7 5v12.5c0 3.04 2.46 5.5 5.5 5.5s5.5-2.46 5.5-5.5V6h-1.5z"/>
                        </svg>
                    </Show>
                    <span class="email-time">{&email.timestamp}</span>
                </div>
            </div>
        </div>
    }
}

// =============================================================================
// 5. Email View
// =============================================================================

#[component]
fn EmailView(
    email: Email,
    on_back: impl Fn(ev::MouseEvent) + 'static,
) -> impl IntoView {
    let has_agent_response = email.agent_response.is_some();
    let agent_response = email.agent_response.clone();
    let has_attachments = !email.attachments.is_empty();
    let attachments = email.attachments.clone();
    let attachments_len = attachments.len();
    
    view! {
        <div class="email-view">
            <div class="email-view-header">
                <button class="back-btn" on:click=on_back>
                    <svg viewBox="0 0 24 24" fill="currentColor">
                        <path d="M20 11H7.83l5.59-5.59L12 4l-8 8 8 8 1.41-1.41L7.83 13H20v-2z"/>
                    </svg>
                </button>
                
                <div class="email-view-actions">
                    <button class="action-btn" title="Archive">
                        <svg viewBox="0 0 24 24" fill="currentColor">
                            <path d="M20.54 5.23l-1.39-1.68C18.88 3.21 18.47 3 18 3H6c-.47 0-.88.21-1.16.55L3.46 5.23C3.17 5.57 3 6.02 3 6.5V19c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2V6.5c0-.48-.17-.93-.46-1.27zM12 17.5L6.5 12H10v-2h4v2h3.5L12 17.5zM5.12 5l.81-1h12l.94 1H5.12z"/>
                        </svg>
                    </button>
                    <button class="action-btn" title="Delete">
                        <svg viewBox="0 0 24 24" fill="currentColor">
                            <path d="M6 19c0 1.1.9 2 2 2h8c1.1 0 2-.9 2-2V7H6v12zM19 4h-3.5l-1-1h-5l-1 1H5v2h14V4z"/>
                        </svg>
                    </button>
                    <button class="action-btn" title="Mark as unread">
                        <svg viewBox="0 0 24 24" fill="currentColor">
                            <path d="M20 4H4c-1.1 0-2 .9-2 2v12c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V6c0-1.1-.9-2-2-2zm0 4l-8 5-8-5V6l8 5 8-5v2z"/>
                        </svg>
                    </button>
                    <button class="action-btn" title="Reply">
                        <svg viewBox="0 0 24 24" fill="currentColor">
                            <path d="M10 9V5l-7 7 7 7v-4.1c5 0 8.5 1.6 11 5.1-1-5-4-10-11-11z"/>
                        </svg>
                    </button>
                    <button class="action-btn" title="Forward">
                        <svg viewBox="0 0 24 24" fill="currentColor">
                            <path d="M14 9V5l7 7-7 7v-4.1c-5 0-8.5 1.6-11 5.1 1-5 4-10 11-11z"/>
                        </svg>
                    </button>
                </div>
            </div>
            
            <div class="email-view-content">
                <h1 class="email-subject">{&email.subject}</h1>
                
                <div class="email-meta">
                    <div class="sender-avatar">
                        {email.from.display().chars().next().unwrap_or('?')}
                    </div>
                    <div class="email-meta-info">
                        <div class="sender-line">
                            <span class="sender-name">{email.from.display()}</span>
                            <span class="sender-email">"<"{&email.from.email}">"</span>
                        </div>
                        <div class="recipient-line">
                            "to "
                            {email.to.iter().map(|c| c.display()).collect::<Vec<_>>().join(", ")}
                        </div>
                    </div>
                    <div class="email-date">{&email.timestamp}</div>
                </div>
                
                // AI Response section
                <Show when=move || has_agent_response>
                    {agent_response.clone().map(|response| view! {
                        <div class="ai-response-card">
                            <div class="ai-response-header">
                                <svg viewBox="0 0 24 24" fill="currentColor" class="ai-icon">
                                    <path d="M21 10.12h-6.78l2.74-2.82c-2.73-2.7-7.15-2.8-9.88-.1-2.73 2.71-2.73 7.08 0 9.79s7.15 2.71 9.88 0C18.32 15.65 19 14.08 19 12.1h2c0 1.98-.88 4.55-2.64 6.29-3.51 3.48-9.21 3.48-12.72 0-3.5-3.47-3.53-9.11-.02-12.58s9.14-3.47 12.65 0L21 3v7.12z"/>
                                </svg>
                                <span class="ai-agent-name">{&response.agent_name}" responded"</span>
                                <span class="ai-confidence">
                                    {format!("{:.0}% confidence", response.confidence * 100.0)}
                                </span>
                            </div>
                            <div class="ai-response-body">
                                {&response.response}
                            </div>
                            <div class="ai-response-actions">
                                <button class="btn btn-sm btn-primary">"Approve & Send"</button>
                                <button class="btn btn-sm btn-secondary">"Edit Response"</button>
                                <button class="btn btn-sm btn-ghost">"Reject"</button>
                            </div>
                        </div>
                    })}
                </Show>
                
                <div class="email-body">
                    {if let Some(html) = email.body_html.clone() {
                    view! { <div inner_html=html></div> }.into_view()
                } else {
                    view! { <pre class="email-text">{email.body_text.clone().unwrap_or_default()}</pre> }.into_view()
                }}
                </div>
                
                // Attachments
                <Show when=move || has_attachments>
                    <div class="attachments-section">
                        <h4>"Attachments ("{attachments_len}")"</h4>
                        <div class="attachments-grid">
                            {attachments.iter().map(|att| {
                                let is_p2p = att.is_p2p;
                                let is_downloading = att.is_downloading();
                                let is_downloaded = att.is_downloaded();
                                let progress = att.progress_percent.unwrap_or(0.0);
                                let magnet = att.magnet_link.clone();
                                
                                view! {
                                    <div class=move || {
                                        let mut classes = vec!["attachment-card"];
                                        if is_p2p { classes.push("p2p"); }
                                        if is_downloading { classes.push("downloading"); }
                                        if is_downloaded { classes.push("completed"); }
                                        classes.join(" ")
                                    }>
                                        // Icon based on type
                                        {if is_p2p {
                                            view! {
                                                <div class="attachment-icon p2p-icon">
                                                    <svg viewBox="0 0 24 24" fill="currentColor">
                                                        <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-1 17.93c-3.95-.49-7-3.85-7-7.93 0-.62.08-1.21.21-1.79L9 15v1c0 1.1.9 2 2 2v1.93zm6.9-2.54c-.26-.81-1-1.39-1.9-1.39h-1v-3c0-.55-.45-1-1-1H8v-2h2c.55 0 1-.45 1-1V7h2c1.1 0 2-.9 2-2v-.41c2.93 1.19 5 4.06 5 7.41 0 2.08-.8 3.97-2.1 5.39z"/>
                                                    </svg>
                                                    <span class="p2p-badge">"P2P"</span>
                                                </div>
                                            }.into_view()
                                        } else {
                                            view! {
                                                <svg viewBox="0 0 24 24" fill="currentColor" class="attachment-icon">
                                                    <path d="M14 2H6c-1.1 0-1.99.9-1.99 2L4 20c0 1.1.89 2 1.99 2H18c1.1 0 2-.9 2-2V8l-6-6zm4 18H6V4h7v5h5v11z"/>
                                                </svg>
                                            }.into_view()
                                        }}
                                        
                                        <div class="attachment-info">
                                            <span class="attachment-name">{&att.name}</span>
                                            <span class="attachment-size">{att.size_display()}</span>
                                            
                                            // P2P download progress
                                            {if is_p2p && is_downloading {
                                                view! {
                                                    <div class="p2p-progress">
                                                        <div class="p2p-progress-bar">
                                                            <div 
                                                                class="p2p-progress-fill" 
                                                                style=format!("width: {}%", progress)
                                                            ></div>
                                                        </div>
                                                        <span class="p2p-progress-text">
                                                            {format!("{:.0}%", progress)}
                                                        </span>
                                                    </div>
                                                }.into_view()
                                            } else {
                                                view! { <span></span> }.into_view()
                                            }}
                                        </div>
                                        
                                        // Download button for P2P attachments
                                        {if is_p2p {
                                            let magnet_clone = magnet.clone();
                                            view! {
                                                <button 
                                                    class=move || {
                                                        if is_downloaded { "p2p-download-btn completed" }
                                                        else if is_downloading { "p2p-download-btn downloading" }
                                                        else { "p2p-download-btn" }
                                                    }
                                                    disabled=is_downloading
                                                    on:click=move |_| {
                                                        if let Some(ref m) = magnet_clone {
                                                            log::info!("Starting P2P download: {}", m);
                                                            // TODO: Call API to start download
                                                        }
                                                    }
                                                >
                                                    {if is_downloaded {
                                                        view! {
                                                            <svg viewBox="0 0 24 24" fill="currentColor">
                                                                <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z"/>
                                                            </svg>
                                                        }.into_view()
                                                    } else if is_downloading {
                                                        view! {
                                                            <svg viewBox="0 0 24 24" fill="currentColor" class="spin">
                                                                <path d="M12 4V1L8 5l4 4V6c3.31 0 6 2.69 6 6 0 1.01-.25 1.97-.7 2.8l1.46 1.46C19.54 15.03 20 13.57 20 12c0-4.42-3.58-8-8-8zm0 14c-3.31 0-6-2.69-6-6 0-1.01.25-1.97.7-2.8L5.24 7.74C4.46 8.97 4 10.43 4 12c0 4.42 3.58 8 8 8v3l4-4-4-4v3z"/>
                                                            </svg>
                                                        }.into_view()
                                                    } else {
                                                        view! {
                                                            <svg viewBox="0 0 24 24" fill="currentColor">
                                                                <path d="M19 9h-4V3H9v6H5l7 7 7-7zM5 18v2h14v-2H5z"/>
                                                            </svg>
                                                        }.into_view()
                                                    }}
                                                </button>
                                            }.into_view()
                                        } else {
                                            view! {
                                                <button class="download-btn">
                                                    <svg viewBox="0 0 24 24" fill="currentColor">
                                                        <path d="M19 9h-4V3H9v6H5l7 7 7-7zM5 18v2h14v-2H5z"/>
                                                    </svg>
                                                </button>
                                            }.into_view()
                                        }}
                                    </div>
                                }
                            }).collect_view()}
                        </div>
                    </div>
                </Show>
            </div>
        </div>
    }
}

// =============================================================================
// 6. Compose Modal
// =============================================================================

#[component]
fn ComposeModal(on_close: impl Fn(ev::MouseEvent) + 'static) -> impl IntoView {
    let (to, set_to) = create_signal(String::new());
    let (subject, set_subject) = create_signal(String::new());
    let (body, set_body) = create_signal(String::new());
    let (minimized, set_minimized) = create_signal(false);

    view! {
        <div class=move || if minimized.get() { "compose-modal minimized" } else { "compose-modal" }>
            <div class="compose-header">
                <span class="compose-title">"New Message"</span>
                <div class="compose-controls">
                    <button class="compose-btn-icon" on:click=move |_| set_minimized.update(|m| *m = !*m)>
                        <svg viewBox="0 0 24 24" fill="currentColor">
                            <path d="M19 13H5v-2h14v2z"/>
                        </svg>
                    </button>
                    <button class="compose-btn-icon" on:click=on_close>
                        <svg viewBox="0 0 24 24" fill="currentColor">
                            <path d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z"/>
                        </svg>
                    </button>
                </div>
            </div>
            
            <Show when=move || !minimized.get()>
                <div class="compose-body">
                    <div class="compose-field">
                        <label>"To"</label>
                        <input 
                            type="text" 
                            prop:value=move || to.get()
                            on:input=move |e| set_to.set(event_target_value(&e))
                        />
                    </div>
                    <div class="compose-field">
                        <label>"Subject"</label>
                        <input 
                            type="text"
                            prop:value=move || subject.get()
                            on:input=move |e| set_subject.set(event_target_value(&e))
                        />
                    </div>
                    <div class="compose-content">
                        <textarea
                            prop:value=move || body.get()
                            on:input=move |e| set_body.set(event_target_value(&e))
                            placeholder="Write your message..."
                        ></textarea>
                    </div>
                </div>
                
                <div class="compose-footer">
                    <button class="btn btn-primary">"Send"</button>
                    <div class="compose-tools">
                        <button class="tool-btn" title="Attach file">
                            <svg viewBox="0 0 24 24" fill="currentColor">
                                <path d="M16.5 6v11.5c0 2.21-1.79 4-4 4s-4-1.79-4-4V5c0-1.38 1.12-2.5 2.5-2.5s2.5 1.12 2.5 2.5v10.5c0 .55-.45 1-1 1s-1-.45-1-1V6H10v9.5c0 1.38 1.12 2.5 2.5 2.5s2.5-1.12 2.5-2.5V5c0-2.21-1.79-4-4-4S7 2.79 7 5v12.5c0 3.04 2.46 5.5 5.5 5.5s5.5-2.46 5.5-5.5V6h-1.5z"/>
                            </svg>
                        </button>
                        <button class="tool-btn" title="AI assist">
                            <svg viewBox="0 0 24 24" fill="currentColor">
                                <path d="M21 10.12h-6.78l2.74-2.82c-2.73-2.7-7.15-2.8-9.88-.1-2.73 2.71-2.73 7.08 0 9.79s7.15 2.71 9.88 0C18.32 15.65 19 14.08 19 12.1h2c0 1.98-.88 4.55-2.64 6.29-3.51 3.48-9.21 3.48-12.72 0-3.5-3.47-3.53-9.11-.02-12.58s9.14-3.47 12.65 0L21 3v7.12z"/>
                            </svg>
                        </button>
                    </div>
                </div>
            </Show>
        </div>
    }
}

// =============================================================================
// Helper Functions
// =============================================================================

fn format_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}

// Sample data functions removed - now using real API data
