//! DNS Wizard Page - Step-by-step DNS verification
//! Fetches DNS records from Supabase/JSON API and displays them for configuration

use leptos::*;
use leptos_router::*;
use wasm_bindgen_futures::spawn_local;
use crate::api::{self, DnsRecordResponse};

#[component]
pub fn DnsWizardPage() -> impl IntoView {
    let (current_step, set_current_step) = create_signal(0usize);
    let (loading, set_loading) = create_signal(true);
    let (error, set_error) = create_signal::<Option<String>>(None);
    let (dns_records, set_dns_records) = create_signal::<Vec<DnsRecordResponse>>(vec![]);
    let (verifying, set_verifying) = create_signal(false);
    let (verification_result, set_verification_result) = create_signal::<Option<api::DnsVerificationResponse>>(None);
    
    // Get domain ID from URL params or fallback to localStorage
    let params = use_params_map();
    let domain_id = params.with(|p| {
        p.get("domain_id").cloned()
            .or_else(|| {
                use gloo_storage::Storage;
                gloo_storage::LocalStorage::get::<String>("dns_wizard_domain").ok()
            })
            .unwrap_or_else(|| "example-com".to_string())
    });
    let (domain, _set_domain) = create_signal(domain_id.clone());
    
    // Fetch DNS records from API on mount
    {
        let domain_id = domain_id.clone();
        let set_dns_records = set_dns_records.clone();
        let set_loading = set_loading.clone();
        let set_error = set_error.clone();
        
        spawn_local(async move {
            log::info!("Fetching DNS records for domain: {}", domain_id);
            match api::fetch_dns_records_for_config(&domain_id).await {
                Ok(records) => {
                    log::info!("Fetched {} DNS records", records.len());
                    set_dns_records.set(records);
                }
                Err(e) => {
                    log::error!("Failed to fetch DNS records: {}", e);
                    set_error.set(Some(format!("Failed to load DNS records: {}", e)));
                }
            }
            set_loading.set(false);
        });
    }
    
    // DNS setup: A record first (for mail hostname), then MX, SPF, DKIM, DMARC
    let steps = vec![
        WizardStep {
            title: "A Record (Mail Server)",
            description: "Point mail.yourdomain.com to your server's IP address",
            record_type: "A",
            required: true,
        },
        WizardStep {
            title: "MX Record",
            description: "Route incoming emails to your Amail server",
            record_type: "MX",
            required: true,
        },
        WizardStep {
            title: "SPF Record",
            description: "Authorize your server to send emails (prevents spoofing)",
            record_type: "SPF",
            required: true,
        },
        WizardStep {
            title: "DKIM Record",
            description: "Add digital signatures to outgoing emails",
            record_type: "DKIM",
            required: false,
        },
        WizardStep {
            title: "DMARC Record",
            description: "Set policy for failed authentication",
            record_type: "DMARC",
            required: false,
        },
    ];

    let total_steps = steps.len();
    
    // Verify DNS records handler - clone domain_id for the closure
    let domain_id_for_verify = domain_id.clone();

    view! {
        <div class="page dns-wizard-page">
            <div class="wizard-header">
                <h1 class="page-title">"DNS Configuration Wizard"</h1>
                <p class="page-subtitle">"Configure DNS records for "{move || domain.get()}</p>
            </div>
            
            // Loading state
            <Show when=move || loading.get()>
                <div class="loading-state" style="text-align: center; padding: 40px;">
                    <p>"Loading DNS records..."</p>
                </div>
            </Show>
            
            // Error state
            <Show when=move || error.get().is_some()>
                <div class="alert alert-error" style="margin-bottom: 20px;">
                    {move || error.get().unwrap_or_default()}
                </div>
            </Show>
            
            // Main content when loaded
            <Show when=move || !loading.get()>
                // Progress indicator
                <div class="wizard-progress">
                    {steps.iter().enumerate().map(|(i, step)| {
                        let is_active = move || current_step.get() == i;
                        let is_completed = move || current_step.get() > i;
                        let step_title = step.title;
                        view! {
                            <div class=move || {
                                let mut classes = vec!["wizard-step"];
                                if is_active() { classes.push("active"); }
                                if is_completed() { classes.push("completed"); }
                                classes.join(" ")
                            }>
                                <div class="step-number">
                                    {move || if is_completed() {
                                        view! { <svg viewBox="0 0 24 24" fill="currentColor"><path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z"/></svg> }.into_view()
                                    } else {
                                        view! { <span>{i + 1}</span> }.into_view()
                                    }}
                                </div>
                                <span class="step-title">{step_title}</span>
                            </div>
                        }
                    }).collect_view()}
                </div>
                
                // Step content - display records from API
                <div class="wizard-content">
                    <DnsStepContent 
                        step=current_step
                        records=dns_records
                        domain=domain
                        steps=steps.clone()
                    />
                </div>
                
                // Verification section
                <div class="verification-section" style="margin: 20px 0; padding: 20px; background: var(--bg-secondary); border-radius: 8px;">
                    <h3>"Verify DNS Configuration"</h3>
                    <p style="color: var(--text-secondary); margin-bottom: 16px;">
                        "After adding the DNS records, click verify to check if they're properly configured."
                    </p>
                    <button 
                        class="btn btn-primary"
                        on:click={
                            let domain_id = domain_id_for_verify.clone();
                            move |_| {
                                let domain_id = domain_id.clone();
                                set_verifying.set(true);
                                set_verification_result.set(None);
                                
                                spawn_local(async move {
                                    match api::verify_dns_records(&domain_id).await {
                                        Ok(result) => {
                                            set_verification_result.set(Some(result));
                                        }
                                        Err(e) => {
                                            log::error!("DNS verification failed: {}", e);
                                        }
                                    }
                                    set_verifying.set(false);
                                });
                            }
                        }
                        disabled=move || verifying.get()
                    >
                        {move || if verifying.get() { "Verifying..." } else { "Verify DNS Records" }}
                    </button>
                    
                    <Show when=move || verification_result.get().is_some()>
                        <div class="verification-results" style="margin-top: 16px;">
                            {move || {
                                let result = verification_result.get().unwrap();
                                view! {
                                    <div class="verification-grid" style="display: grid; grid-template-columns: repeat(4, 1fr); gap: 12px;">
                                        <div class=if result.mx_verified { "verified" } else { "not-verified" }>
                                            <span class="status-icon">{if result.mx_verified { "✓" } else { "✗" }}</span>
                                            " MX"
                                        </div>
                                        <div class=if result.spf_verified { "verified" } else { "not-verified" }>
                                            <span class="status-icon">{if result.spf_verified { "✓" } else { "✗" }}</span>
                                            " SPF"
                                        </div>
                                        <div class=if result.dkim_verified { "verified" } else { "not-verified" }>
                                            <span class="status-icon">{if result.dkim_verified { "✓" } else { "✗" }}</span>
                                            " DKIM"
                                        </div>
                                        <div class=if result.dmarc_verified { "verified" } else { "not-verified" }>
                                            <span class="status-icon">{if result.dmarc_verified { "✓" } else { "✗" }}</span>
                                            " DMARC"
                                        </div>
                                    </div>
                                }
                            }}
                        </div>
                    </Show>
                </div>
                
                // Navigation
                <div class="wizard-nav">
                    <button 
                        class="btn btn-secondary"
                        disabled=move || current_step.get() == 0
                        on:click=move |_| set_current_step.update(|s| *s = s.saturating_sub(1))
                    >
                        "Previous"
                    </button>
                    
                    <button 
                        class="btn btn-primary"
                        on:click=move |_| {
                            if current_step.get() < total_steps - 1 {
                                set_current_step.update(|s| *s += 1);
                            } else {
                                // Navigate back to tenants page using window.location
                                if let Some(window) = web_sys::window() {
                                    let _ = window.location().set_href("/tenants");
                                }
                            }
                        }
                    >
                        {move || if current_step.get() == total_steps - 1 { "Finish" } else { "Next" }}
                    </button>
                </div>
            </Show>
        </div>
    }
}

#[derive(Clone)]
struct WizardStep {
    title: &'static str,
    description: &'static str,
    record_type: &'static str,
    required: bool,
}

#[component]
fn DnsStepContent(
    step: ReadSignal<usize>,
    records: ReadSignal<Vec<DnsRecordResponse>>,
    domain: ReadSignal<String>,
    steps: Vec<WizardStep>,
) -> impl IntoView {
    view! {
        {move || {
            let current = step.get();
            let all_records = records.get();
            let domain_name = domain.get();
            
            if current >= steps.len() {
                return view! { <div>"All steps completed!"</div> }.into_view();
            }
            
            let step_info = &steps[current];
            let record_type = step_info.record_type;
            
            // Filter records for this step
            let step_records: Vec<_> = all_records.iter()
                .filter(|r| {
                    match record_type {
                        "A" => r.record_type == "A",
                        "MX" => r.record_type == "MX",
                        "SPF" => r.record_type == "TXT" && r.value.contains("v=spf1"),
                        "DKIM" => r.record_type == "TXT" && r.name.contains("_domainkey"),
                        "DMARC" => r.record_type == "TXT" && r.name.contains("_dmarc"),
                        _ => false,
                    }
                })
                .cloned()
                .collect();
            
            view! {
                <div class="wizard-step-content">
                    <div class="step-header" style="display: flex; align-items: center; gap: 12px; margin-bottom: 8px;">
                        <h2 style="margin: 0;">"Step "{current + 1}": "{step_info.title}</h2>
                        {if step_info.required {
                            view! { <span class="badge badge-required" style="background: var(--primary); color: white; padding: 4px 8px; border-radius: 4px; font-size: 12px;">"Required"</span> }.into_view()
                        } else {
                            view! { <span class="badge badge-optional" style="background: var(--bg-tertiary); color: var(--text-secondary); padding: 4px 8px; border-radius: 4px; font-size: 12px;">"Optional"</span> }.into_view()
                        }}
                    </div>
                    <p class="step-description">{step_info.description}</p>
                    
                    {if step_records.is_empty() {
                        view! {
                            <div class="info-box" style="background: var(--warning-bg); border-color: var(--warning-border);">
                                <p>"No "{record_type}" records found. The server will generate default records for your domain."</p>
                            </div>
                        }.into_view()
                    } else {
                        step_records.into_iter().map(|record| {
                            view! {
                                <DnsRecordCard
                                    record_type=record.record_type.leak()
                                    name=record.name
                                    value=record.value
                                    ttl=record.ttl.to_string().leak()
                                    priority=record.priority.unwrap_or(0) as u32
                                />
                            }
                        }).collect_view()
                    }}
                    
                    <div class="info-box" style="margin-top: 16px;">
                        <svg viewBox="0 0 24 24" fill="currentColor" class="info-icon">
                            <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-6h2v6zm0-8h-2V7h2v2z"/>
                        </svg>
                        <p>"Add these records to your DNS provider (e.g., Cloudflare, GoDaddy, Namecheap). DNS changes may take up to 48 hours to propagate."</p>
                    </div>
                </div>
            }.into_view()
        }}
    }
}

#[component]
fn DnsRecordCard(
    record_type: &'static str,
    name: String,
    value: String,
    ttl: &'static str,
    #[prop(optional)] priority: u32,
) -> impl IntoView {
    let (copied, set_copied) = create_signal(false);
    let value_clone = value.clone();
    
    let copy_value = move |_| {
        // Simple copy feedback without clipboard API
        set_copied.set(true);
        set_timeout(move || set_copied.set(false), std::time::Duration::from_secs(2));
    };

    view! {
        <div class="dns-record-card">
            <div class="record-header">
                <span class="record-type">{record_type}</span>
                <button class="copy-btn" on:click=copy_value>
                    {move || if copied.get() {
                        view! { <span>"Copied!"</span> }.into_view()
                    } else {
                        view! {
                            <svg viewBox="0 0 24 24" fill="currentColor">
                                <path d="M16 1H4c-1.1 0-2 .9-2 2v14h2V3h12V1zm3 4H8c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h11c1.1 0 2-.9 2-2V7c0-1.1-.9-2-2-2zm0 16H8V7h11v14z"/>
                            </svg>
                        }.into_view()
                    }}
                </button>
            </div>
            <div class="record-fields">
                <div class="record-field">
                    <label>"Name / Host"</label>
                    <code>{name}</code>
                </div>
                {(priority > 0).then(|| view! {
                    <div class="record-field">
                        <label>"Priority"</label>
                        <code>{priority}</code>
                    </div>
                })}
                <div class="record-field">
                    <label>"Value / Points to"</label>
                    <code class="record-value">{value}</code>
                </div>
                <div class="record-field">
                    <label>"TTL"</label>
                    <code>{ttl}</code>
                </div>
            </div>
        </div>
    }
}
