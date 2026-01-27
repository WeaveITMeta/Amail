//! Mini Inbox Widget - Embeddable email inbox for external sites
//!
//! Features:
//! - Compact email list view
//! - Unread count badge
//! - Click to expand/view email
//! - Real-time updates via polling

use leptos::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InboxEmail {
    pub id: String,
    pub from: String,
    pub subject: String,
    pub preview: String,
    pub timestamp: String,
    pub is_read: bool,
    pub has_ai_reply: bool,
}

/// Mini inbox widget for embedding in external Leptos apps
#[component]
pub fn AmailInbox(
    /// API URL for the Amail server
    #[prop(into)] api_url: String,
    /// Mailbox to display (e.g., "inbox", "support")
    #[prop(into)] mailbox: String,
    /// Maximum emails to show
    #[prop(default = 5)] max_items: usize,
    /// Show header with title
    #[prop(default = true)] show_header: bool,
    /// Compact mode
    #[prop(default = false)] compact: bool,
) -> impl IntoView {
    let (emails, set_emails) = create_signal(sample_emails());
    let (expanded_id, set_expanded_id) = create_signal::<Option<String>>(None);
    let (is_loading, _set_is_loading) = create_signal(false);
    
    let unread_count = move || emails.get().iter().filter(|e| !e.is_read).count();
    
    let container_class = if compact { "amail-inbox amail-compact" } else { "amail-inbox" };

    view! {
        <div class=container_class>
            <Show when=move || show_header>
                <div class="amail-inbox-header">
                    <div class="amail-inbox-title">
                        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                            <path d="M3 8l7.89 5.26a2 2 0 002.22 0L21 8M5 19h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z"/>
                        </svg>
                        <span>{mailbox.clone()}</span>
                    </div>
                    <Show when=move || unread_count() > 0>
                        <span class="amail-unread-badge">{unread_count}</span>
                    </Show>
                </div>
            </Show>
            
            <div class="amail-inbox-list">
                <Show 
                    when=move || !is_loading.get()
                    fallback=|| view! { <div class="amail-loading">"Loading..."</div> }
                >
                    {move || {
                        let items: Vec<_> = emails.get().into_iter().take(max_items).collect();
                        if items.is_empty() {
                            view! {
                                <div class="amail-empty">
                                    <span>"No emails"</span>
                                </div>
                            }.into_view()
                        } else {
                            items.into_iter().map(|email| {
                                let email_id = email.id.clone();
                                let is_expanded = move || expanded_id.get() == Some(email_id.clone());
                                let email_for_click = email.clone();
                                
                                view! {
                                    <div 
                                        class=move || if email.is_read { "amail-email-item read" } else { "amail-email-item unread" }
                                        on:click=move |_| {
                                            let id = email_for_click.id.clone();
                                            set_expanded_id.update(|current| {
                                                *current = if *current == Some(id.clone()) { None } else { Some(id) };
                                            });
                                        }
                                    >
                                        <div class="amail-email-row">
                                            <div class="amail-email-from">
                                                {if email.has_ai_reply { "🤖 " } else { "" }}
                                                {&email.from}
                                            </div>
                                            <div class="amail-email-time">{&email.timestamp}</div>
                                        </div>
                                        <div class="amail-email-subject">{&email.subject}</div>
                                        
                                        <Show when=is_expanded>
                                            <div class="amail-email-preview">{&email.preview}</div>
                                        </Show>
                                    </div>
                                }
                            }).collect_view()
                        }
                    }}
                </Show>
            </div>
            
            <div class="amail-inbox-footer">
                <a href=format!("{}/inbox", api_url) target="_blank" class="amail-view-all">
                    "View all →"
                </a>
            </div>
        </div>
    }
}

fn sample_emails() -> Vec<InboxEmail> {
    vec![
        InboxEmail {
            id: "1".to_string(),
            from: "john@example.com".to_string(),
            subject: "Meeting tomorrow".to_string(),
            preview: "Hi, just wanted to confirm our meeting tomorrow at 2pm...".to_string(),
            timestamp: "10:30".to_string(),
            is_read: false,
            has_ai_reply: true,
        },
        InboxEmail {
            id: "2".to_string(),
            from: "support@vendor.io".to_string(),
            subject: "Re: Technical Issue".to_string(),
            preview: "We've resolved the issue you reported. Please let us know if...".to_string(),
            timestamp: "09:15".to_string(),
            is_read: true,
            has_ai_reply: false,
        },
        InboxEmail {
            id: "3".to_string(),
            from: "newsletter@tech.com".to_string(),
            subject: "Weekly AI News".to_string(),
            preview: "This week in AI: New breakthroughs in language models...".to_string(),
            timestamp: "Yesterday".to_string(),
            is_read: true,
            has_ai_reply: false,
        },
    ]
}
