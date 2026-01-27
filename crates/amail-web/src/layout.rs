//! Layout Components - Navigation, Header, Footer
//!
//! Table of Contents:
//! 1. AppLayout - Main layout wrapper with top nav
//! 2. TopNavBar - Global navigation (Landing ↔ Amail Center)
//! 3. NavSidebar - Collapsible sidebar for Amail Center
//! 4. Footer - Global footer with legal links

use leptos::*;
use leptos_router::*;
use crate::state::{DashboardState, ConnectionStatus};

/// Application view mode
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AppView {
    Landing,
    AmailCenter,
    Docs,
}

/// Current route/page within Amail Center
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Route {
    Dashboard,
    Inbox,
    Reviews,
    Analytics,
    Tenants,
    Domains,
    DnsWizard,
    Agents,
    Settings,
    Docs,
    WorkspaceRules,
}

impl Route {
    pub fn path(&self) -> &'static str {
        match self {
            Route::Dashboard => "/",
            Route::Inbox => "/inbox",
            Route::Reviews => "/reviews",
            Route::Analytics => "/analytics",
            Route::Tenants => "/tenants",
            Route::Domains => "/domains",
            Route::DnsWizard => "/dns-wizard",
            Route::Agents => "/agents",
            Route::Settings => "/settings",
            Route::Docs => "/docs",
            Route::WorkspaceRules => "/workspace-rules",
        }
    }
    
    pub fn label(&self) -> &'static str {
        match self {
            Route::Dashboard => "Dashboard",
            Route::Inbox => "Inbox",
            Route::Reviews => "Reviews",
            Route::Analytics => "Analytics",
            Route::Tenants => "Tenants",
            Route::Domains => "DNS Setup",
            Route::DnsWizard => "DNS Wizard",
            Route::Agents => "AI Agents",
            Route::WorkspaceRules => "Rules",
            Route::Settings => "Settings",
            Route::Docs => "Documentation",
        }
    }
}

/// Main application layout with top nav and conditional sidebar
#[component]
pub fn AppLayout(
    children: Children,
) -> impl IntoView {
    let (app_view, set_app_view) = create_signal(AppView::AmailCenter);
    let (sidebar_collapsed, set_sidebar_collapsed) = create_signal(false);
    let (mobile_menu_open, set_mobile_menu_open) = create_signal(false);
    
    let state = use_context::<DashboardState>().expect("DashboardState not provided");
    let connection_status = state.connection_status.read_only();
    
    // Store children in a fragment that can be rendered
    let children_view = children().into_view();

    view! {
        <div class="app-root">
            <TopNavBar app_view=app_view set_app_view=set_app_view connection_status=Signal::from(connection_status)/>
            
            {move || match app_view.get() {
                AppView::Landing => view! { <LandingPage set_app_view=set_app_view/> }.into_view(),
                AppView::Docs => view! { <crate::pages::DocsPage/> }.into_view(),
                AppView::AmailCenter => view! {
                    <div class=move || {
                        let mut classes = vec!["app-layout"];
                        if sidebar_collapsed.get() { classes.push("sidebar-collapsed"); }
                        if mobile_menu_open.get() { classes.push("mobile-menu-open"); }
                        classes.join(" ")
                    }>
                        <Show when=move || mobile_menu_open.get()>
                            <div class="mobile-overlay" on:click=move |_| set_mobile_menu_open.set(false)></div>
                        </Show>
                        
                        <NavSidebar
                            collapsed=sidebar_collapsed
                            set_collapsed=set_sidebar_collapsed
                            mobile_open=mobile_menu_open
                            set_mobile_open=set_mobile_menu_open
                        />
                        
                        <div class="main-area">
                            <CenterHeader set_mobile_menu_open=set_mobile_menu_open/>
                            <main class="main-content">{children_view.clone()}</main>
                        </div>
                    </div>
                }.into_view(),
            }}
            
            <GlobalFooter/>
        </div>
    }
}

/// Global top navigation bar
#[component]
pub fn TopNavBar(
    app_view: ReadSignal<AppView>,
    set_app_view: WriteSignal<AppView>,
    #[prop(into)] connection_status: Signal<ConnectionStatus>,
    #[prop(optional)] on_docs_click: Option<Callback<()>>,
) -> impl IntoView {
    view! {
        <nav class="top-nav-bar">
            <div class="top-nav-container">
                <div class="top-nav-brand" on:click=move |_| set_app_view.set(AppView::Landing)>
                    <svg class="brand-icon" viewBox="0 0 24 24" fill="currentColor">
                        <path d="M20 4H4c-1.1 0-2 .9-2 2v12c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V6c0-1.1-.9-2-2-2zm0 4l-8 5-8-5V6l8 5 8-5v2z"/>
                    </svg>
                    <span class="brand-text">"Amail"</span>
                    <span class="brand-tagline">"Autonomous Mail"</span>
                </div>
                
                <div class="top-nav-links">
                    <button class=move || if app_view.get() == AppView::Landing { "top-nav-link active" } else { "top-nav-link" } on:click=move |_| set_app_view.set(AppView::Landing)>
                        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6"/></svg>
                        "Home"
                    </button>
                    <button class=move || if app_view.get() == AppView::AmailCenter { "top-nav-link active" } else { "top-nav-link" } on:click=move |_| set_app_view.set(AppView::AmailCenter)>
                        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M3 8l7.89 5.26a2 2 0 002.22 0L21 8M5 19h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z"/></svg>
                        "Amail"
                    </button>
                    <button class=move || if app_view.get() == AppView::Docs { "top-nav-link active" } else { "top-nav-link" } on:click=move |_| {
                        set_app_view.set(AppView::Docs);
                        if let Some(cb) = on_docs_click {
                            cb.call(());
                        }
                    }>
                        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"/></svg>
                        "Documentation"
                    </button>
                    <a href="https://github.com/your-repo/amail" target="_blank" class="top-nav-link">
                        <svg viewBox="0 0 24 24" fill="currentColor"><path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z"/></svg>
                        "GitHub"
                    </a>
                </div>
                
                <div class="top-nav-right">
                    {move || {
                        let status = connection_status.get();
                        view! {
                            <div class=format!("status-badge {}", status.css_class())>
                                <span class="status-dot"></span>
                                {status.label()}
                            </div>
                        }
                    }}
                </div>
            </div>
        </nav>
    }
}

/// Landing page content
#[component]
fn LandingPage(set_app_view: WriteSignal<AppView>) -> impl IntoView {
    view! {
        <div class="landing-page">
            <div class="landing-hero">
                <div class="hero-content">
                    <h1 class="hero-title">"The Ultimate "<span class="highlight">"AI-Powered"</span>" Mail Server"</h1>
                    <p class="hero-subtitle">"Self-hosted. Open source. Free forever. Your AI staff handles email while you focus on what matters."</p>
                    
                    <div class="hero-features">
                        <div class="feature-pill"><svg viewBox="0 0 24 24" fill="currentColor"><path d="M12 1L3 5v6c0 5.55 3.84 10.74 9 12 5.16-1.26 9-6.45 9-12V5l-9-4z"/></svg>"100% Self-Hosted"</div>
                        <div class="feature-pill"><svg viewBox="0 0 24 24" fill="currentColor"><path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-2 15l-5-5 1.41-1.41L10 14.17l7.59-7.59L19 8l-9 9z"/></svg>"No Subscriptions"</div>
                        <div class="feature-pill"><svg viewBox="0 0 24 24" fill="currentColor"><path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm0 3c1.66 0 3 1.34 3 3s-1.34 3-3 3-3-1.34-3-3 1.34-3 3-3zm0 14.2c-2.5 0-4.71-1.28-6-3.22.03-1.99 4-3.08 6-3.08 1.99 0 5.97 1.09 6 3.08-1.29 1.94-3.5 3.22-6 3.22z"/></svg>"30+ AI Agents"</div>
                        <div class="feature-pill"><svg viewBox="0 0 24 24" fill="currentColor"><path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2z"/></svg>"Unlimited Domains"</div>
                    </div>
                    
                    <div class="hero-cta">
                        <button class="cta-primary" on:click=move |_| set_app_view.set(AppView::AmailCenter)>
                            "Open Amail Center"
                            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M5 12h14M12 5l7 7-7 7"/></svg>
                        </button>
                        <a href="https://github.com/your-repo/amail" target="_blank" class="cta-secondary">"View on GitHub"</a>
                    </div>
                </div>
                
                <div class="hero-visual">
                    <div class="visual-card">
                        <div class="visual-header">
                            <span class="visual-dot red"></span>
                            <span class="visual-dot yellow"></span>
                            <span class="visual-dot green"></span>
                        </div>
                        <div class="visual-content">
                            <div class="visual-inbox">
                                <div class="inbox-item"><div class="inbox-avatar">"J"</div><div class="inbox-text"><strong>"John Smith"</strong><span>"Product inquiry about..."</span></div><div class="inbox-badge ai">"AI Replied"</div></div>
                                <div class="inbox-item"><div class="inbox-avatar">"S"</div><div class="inbox-text"><strong>"Support Request"</strong><span>"Need help with setup..."</span></div><div class="inbox-badge pending">"Pending"</div></div>
                                <div class="inbox-item"><div class="inbox-avatar">"M"</div><div class="inbox-text"><strong>"Meeting Request"</strong><span>"Can we schedule a call..."</span></div><div class="inbox-badge ai">"AI Replied"</div></div>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
            
            <div class="landing-features">
                <h2>"Why Amail?"</h2>
                <div class="features-grid">
                    <div class="feature-card">
                        <div class="feature-icon"><svg viewBox="0 0 24 24" fill="currentColor"><path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm0 3c1.66 0 3 1.34 3 3s-1.34 3-3 3-3-1.34-3-3 1.34-3 3-3zm0 14.2c-2.5 0-4.71-1.28-6-3.22.03-1.99 4-3.08 6-3.08 1.99 0 5.97 1.09 6 3.08-1.29 1.94-3.5 3.22-6 3.22z"/></svg></div>
                        <h3>"AI Agent Staff"</h3>
                        <p>"30+ specialized AI agents handle customer service, sales, HR, legal, and more - 24/7."</p>
                    </div>
                    <div class="feature-card">
                        <div class="feature-icon"><svg viewBox="0 0 24 24" fill="currentColor"><path d="M12 1L3 5v6c0 5.55 3.84 10.74 9 12 5.16-1.26 9-6.45 9-12V5l-9-4z"/></svg></div>
                        <h3>"Your Data, Your Server"</h3>
                        <p>"Complete privacy. No third-party access. Your emails stay on your infrastructure."</p>
                    </div>
                    <div class="feature-card">
                        <div class="feature-icon"><svg viewBox="0 0 24 24" fill="currentColor"><path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-2 15l-5-5 1.41-1.41L10 14.17l7.59-7.59L19 8l-9 9z"/></svg></div>
                        <h3>"Free Forever"</h3>
                        <p>"No subscriptions. No hidden fees. Open source and community-driven."</p>
                    </div>
                    <div class="feature-card">
                        <div class="feature-icon"><svg viewBox="0 0 24 24" fill="currentColor"><path d="M11.99 2C6.47 2 2 6.48 2 12s4.47 10 9.99 10C17.52 22 22 17.52 22 12S17.52 2 11.99 2zM12 20c-4.42 0-8-3.58-8-8s3.58-8 8-8 8 3.58 8 8-3.58 8-8 8zm.5-13H11v6l5.25 3.15.75-1.23-4.5-2.67z"/></svg></div>
                        <h3>"Auto-Startup at 8am"</h3>
                        <p>"Your AI staff is ready every morning, handling emails before you even start your day."</p>
                    </div>
                    <div class="feature-card">
                        <div class="feature-icon"><svg viewBox="0 0 24 24" fill="currentColor"><path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-1 17.93c-3.95-.49-7-3.85-7-7.93 0-.62.08-1.21.21-1.79L9 15v1c0 1.1.9 2 2 2v1.93zm6.9-2.54c-.26-.81-1-1.39-1.9-1.39h-1v-3c0-.55-.45-1-1-1H8v-2h2c.55 0 1-.45 1-1V7h2c1.1 0 2-.9 2-2v-.41c2.93 1.19 5 4.06 5 7.41 0 2.08-.8 3.97-2.1 5.39z"/></svg></div>
                        <h3>"P2P File Transfer"</h3>
                        <p>"No attachment limits. Send gigabyte files via BitTorrent-style P2P - direct, fast, unlimited."</p>
                    </div>
                    <div class="feature-card">
                        <div class="feature-icon"><svg viewBox="0 0 24 24" fill="currentColor"><path d="M19 3H5c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2V5c0-1.1-.9-2-2-2zm-7 3c1.93 0 3.5 1.57 3.5 3.5S13.93 13 12 13s-3.5-1.57-3.5-3.5S10.07 6 12 6zm7 13H5v-.23c0-.62.28-1.2.76-1.58C7.47 15.82 9.64 15 12 15s4.53.82 6.24 2.19c.48.38.76.97.76 1.58V19z"/></svg></div>
                        <h3>"Local LLM Privacy"</h3>
                        <p>"AI runs on YOUR server via Ollama. No cloud APIs, no data leaks, no training on your emails."</p>
                    </div>
                    <div class="feature-card">
                        <div class="feature-icon"><svg viewBox="0 0 24 24" fill="currentColor"><path d="M12 1L3 5v6c0 5.55 3.84 10.74 9 12 5.16-1.26 9-6.45 9-12V5l-9-4zm0 10.99h7c-.53 4.12-3.28 7.79-7 8.94V12H5V6.3l7-3.11v8.8z"/></svg></div>
                        <h3>"Built-in Compliance"</h3>
                        <p>"GDPR, CAN-SPAM, DMCA, COPPA ready. Legal compliance modules included out of the box."</p>
                    </div>
                    <div class="feature-card">
                        <div class="feature-icon"><svg viewBox="0 0 24 24" fill="currentColor"><path d="M19.43 12.98c.04-.32.07-.64.07-.98s-.03-.66-.07-.98l2.11-1.65c.19-.15.24-.42.12-.64l-2-3.46c-.12-.22-.39-.3-.61-.22l-2.49 1c-.52-.4-1.08-.73-1.69-.98l-.38-2.65C14.46 2.18 14.25 2 14 2h-4c-.25 0-.46.18-.49.42l-.38 2.65c-.61.25-1.17.59-1.69.98l-2.49-1c-.23-.09-.49 0-.61.22l-2 3.46c-.13.22-.07.49.12.64l2.11 1.65c-.04.32-.07.65-.07.98s.03.66.07.98l-2.11 1.65c-.19.15-.24.42-.12.64l2 3.46c.12.22.39.3.61.22l2.49-1c.52.4 1.08.73 1.69.98l.38 2.65c.03.24.24.42.49.42h4c.25 0 .46-.18.49-.42l.38-2.65c.61-.25 1.17-.59 1.69-.98l2.49 1c.23.09.49 0 .61-.22l2-3.46c.12-.22.07-.49-.12-.64l-2.11-1.65zM12 15.5c-1.93 0-3.5-1.57-3.5-3.5s1.57-3.5 3.5-3.5 3.5 1.57 3.5 3.5-1.57 3.5-3.5 3.5z"/></svg></div>
                        <h3>"Self-Healing Agents"</h3>
                        <p>"Erlang-style supervision. Agents auto-restart on failure. Your email never stops processing."</p>
                    </div>
                </div>
            </div>
        </div>
    }
}

/// Amail Center header
#[component]
fn CenterHeader(set_mobile_menu_open: WriteSignal<bool>) -> impl IntoView {
    view! {
        <header class="center-header">
            <button class="hamburger-btn mobile-only" on:click=move |_| set_mobile_menu_open.set(true)>
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M4 6h16M4 12h16M4 18h16"/></svg>
            </button>
            <div class="header-breadcrumb"><span class="breadcrumb-item">"Amail"</span></div>
        </header>
    }
}

/// Navigation sidebar for Amail Center - uses leptos_router for URL-based navigation
#[component]
pub fn NavSidebar(
    collapsed: ReadSignal<bool>,
    set_collapsed: WriteSignal<bool>,
    mobile_open: ReadSignal<bool>,
    set_mobile_open: WriteSignal<bool>,
) -> impl IntoView {
    let nav_items = vec![
        ("/", "Dashboard", "M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6"),
        ("/tenants", "Tenants", "M19 21V5a2 2 0 00-2-2H7a2 2 0 00-2 2v16m14 0h2m-2 0h-5m-9 0H3m2 0h5M9 7h1m-1 4h1m4-4h1m-1 4h1m-5 10v-5a1 1 0 011-1h2a1 1 0 011 1v5m-4 0h4"),
        ("/inbox", "Inbox", "M3 8l7.89 5.26a2 2 0 002.22 0L21 8M5 19h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z"),
        ("/agents", "AI Agents", "M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z"),
        ("/workspace-rules", "Rules", "M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"),
        ("/reviews", "Reviews", "M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"),
        ("/analytics", "Analytics", "M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z"),
        ("/settings", "Settings", "M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z M15 12a3 3 0 11-6 0 3 3 0 016 0z"),
    ];

    view! {
        <aside class=move || {
            let mut classes = vec!["nav-sidebar"];
            if collapsed.get() { classes.push("collapsed"); }
            if mobile_open.get() { classes.push("mobile-open"); }
            classes.join(" ")
        }>
            <div class="sidebar-header">
                <button class="collapse-btn desktop-only" on:click=move |_| set_collapsed.update(|c| *c = !*c)>
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        {move || if collapsed.get() { view! { <path d="M9 18l6-6-6-6"/> }.into_view() } else { view! { <path d="M15 18l-6-6 6-6"/> }.into_view() }}
                    </svg>
                </button>
                <button class="close-btn mobile-only" on:click=move |_| set_mobile_open.set(false)>
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M6 18L18 6M6 6l12 12"/></svg>
                </button>
            </div>
            
            <nav class="sidebar-nav">
                {nav_items.into_iter().map(|(path, label, icon_path)| {
                    let set_mobile = set_mobile_open.clone();
                    view! {
                        <A href=path class="nav-item" on:click=move |_| set_mobile.set(false)>
                            <svg class="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d=icon_path/></svg>
                            <Show when=move || !collapsed.get()><span class="nav-label">{label}</span></Show>
                        </A>
                    }
                }).collect_view()}
            </nav>
            
            <div class="sidebar-footer"><Show when=move || !collapsed.get()><div class="version-info"><span class="version">"v0.1.0"</span></div></Show></div>
        </aside>
    }
}

/// Global footer with legal links
#[component]
pub fn GlobalFooter() -> impl IntoView {
    view! {
        <footer class="global-footer">
            <div class="footer-container">
                <div class="footer-brand">
                    <svg viewBox="0 0 24 24" fill="currentColor"><path d="M20 4H4c-1.1 0-2 .9-2 2v12c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V6c0-1.1-.9-2-2-2zm0 4l-8 5-8-5V6l8 5 8-5v2z"/></svg>
                    <span>"Amail"</span>
                </div>
                
                <div class="footer-links">
                    <div class="footer-section">
                        <h4>"Product"</h4>
                        <a href="#">"Features"</a>
                        <a href="#">"Documentation"</a>
                        <a href="#">"Self-Hosting Guide"</a>
                    </div>
                    <div class="footer-section">
                        <h4>"Community"</h4>
                        <a href="https://github.com/your-repo/amail" target="_blank">"GitHub"</a>
                        <a href="#">"Discord"</a>
                        <a href="#">"Contributing"</a>
                    </div>
                    <div class="footer-section">
                        <h4>"Legal"</h4>
                        <a href="#">"Privacy Policy"</a>
                        <a href="#">"Terms of Service"</a>
                        <a href="#">"License (MIT)"</a>
                    </div>
                </div>
                
                <div class="footer-bottom">
                    <p class="copyright">"© 2024 Amail. Open source under MIT License."</p>
                    <p class="tagline">"The ultimate AI-powered mail server. Self-hosted. Free forever."</p>
                </div>
            </div>
        </footer>
    }
}
