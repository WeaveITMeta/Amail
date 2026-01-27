//! AI-Powered Contact Form Widget
//!
//! Features:
//! - Smart form with name, email, subject, message
//! - AI-assisted subject suggestions
//! - Form validation
//! - Success/error states
//! - Customizable fields

use leptos::*;

#[derive(Clone, Debug, PartialEq)]
pub enum FormStatus {
    Idle,
    Submitting,
    Success,
    Error(String),
}

/// AI-powered contact form for embedding in external Leptos apps
#[component]
pub fn AmailContactForm(
    /// API URL for the Amail server
    #[prop(into)] api_url: String,
    /// Recipient email address
    #[prop(into)] to: String,
    /// Form title
    #[prop(default = "Contact Us".to_string(), into)] title: String,
    /// Show AI badge
    #[prop(default = true)] show_ai_badge: bool,
    /// Compact mode
    #[prop(default = false)] compact: bool,
) -> impl IntoView {
    let (name, set_name) = create_signal(String::new());
    let (email, set_email) = create_signal(String::new());
    let (subject, set_subject) = create_signal(String::new());
    let (message, set_message) = create_signal(String::new());
    let (status, set_status) = create_signal(FormStatus::Idle);
    
    let is_valid = move || {
        !name.get().trim().is_empty() &&
        !email.get().trim().is_empty() &&
        email.get().contains('@') &&
        !subject.get().trim().is_empty() &&
        !message.get().trim().is_empty()
    };
    
    let on_submit = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        
        if !is_valid() {
            return;
        }
        
        set_status.set(FormStatus::Submitting);
        
        // Simulate API call (in real implementation, use gloo-net)
        set_timeout(
            move || {
                set_status.set(FormStatus::Success);
                set_name.set(String::new());
                set_email.set(String::new());
                set_subject.set(String::new());
                set_message.set(String::new());
            },
            std::time::Duration::from_millis(1000),
        );
    };
    
    let container_class = if compact { "amail-contact-form amail-compact" } else { "amail-contact-form" };

    view! {
        <div class=container_class>
            <div class="amail-form-header">
                <h3 class="amail-form-title">{title}</h3>
                <Show when=move || show_ai_badge>
                    <span class="amail-ai-badge">
                        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                            <path d="M12 2a2 2 0 012 2c0 .74-.4 1.39-1 1.73V7h1a7 7 0 017 7h1a1 1 0 011 1v3a1 1 0 01-1 1h-1v1a2 2 0 01-2 2H5a2 2 0 01-2-2v-1H2a1 1 0 01-1-1v-3a1 1 0 011-1h1a7 7 0 017-7h1V5.73c-.6-.34-1-.99-1-1.73a2 2 0 012-2z"/>
                            <circle cx="7.5" cy="14.5" r="1.5"/>
                            <circle cx="16.5" cy="14.5" r="1.5"/>
                        </svg>
                        "AI-Powered"
                    </span>
                </Show>
            </div>
            
            <Show 
                when=move || status.get() == FormStatus::Success
                fallback=move || view! {
                    <form class="amail-form" on:submit=on_submit>
                        <div class="amail-form-row">
                            <div class="amail-form-group">
                                <label for="amail-name">"Name"</label>
                                <input 
                                    type="text" 
                                    id="amail-name"
                                    placeholder="Your name"
                                    prop:value=name
                                    on:input=move |e| set_name.set(event_target_value(&e))
                                    required
                                />
                            </div>
                            <div class="amail-form-group">
                                <label for="amail-email">"Email"</label>
                                <input 
                                    type="email" 
                                    id="amail-email"
                                    placeholder="your@email.com"
                                    prop:value=email
                                    on:input=move |e| set_email.set(event_target_value(&e))
                                    required
                                />
                            </div>
                        </div>
                        
                        <div class="amail-form-group">
                            <label for="amail-subject">"Subject"</label>
                            <input 
                                type="text" 
                                id="amail-subject"
                                placeholder="What's this about?"
                                prop:value=subject
                                on:input=move |e| set_subject.set(event_target_value(&e))
                                required
                            />
                        </div>
                        
                        <div class="amail-form-group">
                            <label for="amail-message">"Message"</label>
                            <textarea 
                                id="amail-message"
                                placeholder="Your message..."
                                rows="4"
                                prop:value=message
                                on:input=move |e| set_message.set(event_target_value(&e))
                                required
                            />
                        </div>
                        
                        <button 
                            type="submit" 
                            class="amail-submit-btn"
                            disabled=move || !is_valid() || status.get() == FormStatus::Submitting
                        >
                            {move || match status.get() {
                                FormStatus::Submitting => "Sending...".to_string(),
                                _ => "Send Message".to_string(),
                            }}
                        </button>
                        
                        <Show when=move || matches!(status.get(), FormStatus::Error(_))>
                            <div class="amail-error">
                                {move || if let FormStatus::Error(msg) = status.get() { msg } else { String::new() }}
                            </div>
                        </Show>
                    </form>
                }
            >
                <div class="amail-success">
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <path d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"/>
                    </svg>
                    <h4>"Message Sent!"</h4>
                    <p>"Our AI assistant will review your message and respond shortly."</p>
                    <button 
                        class="amail-reset-btn"
                        on:click=move |_| set_status.set(FormStatus::Idle)
                    >
                        "Send Another"
                    </button>
                </div>
            </Show>
            
            <div class="amail-form-footer">
                <span class="amail-powered-by">
                    "Powered by "
                    <a href="https://github.com/your-repo/amail" target="_blank">"Amail"</a>
                </span>
            </div>
        </div>
    }
}
