//! Settings Page - System configuration
//!
//! Table of Contents:
//! 1. Server Configuration
//! 2. LLM Configuration
//! 3. Auto-Start & Scheduling
//! 4. Email Settings
//! 5. Danger Zone

use leptos::*;

#[component]
pub fn SettingsPage() -> impl IntoView {
    // Server settings
    let (smtp_port, set_smtp_port) = create_signal("2525".to_string());
    let (http_port, set_http_port) = create_signal("4000".to_string());
    let (ollama_urls, set_ollama_urls) = create_signal(vec![
        "http://localhost:11434".to_string(),
    ]);
    let (new_ollama_url, set_new_ollama_url) = create_signal(String::new());
    let (llm_model, set_llm_model) = create_signal("mistral".to_string());
    
    // Auto-start settings
    let (auto_start_enabled, set_auto_start_enabled) = create_signal(true);
    let (daily_start_hour, set_daily_start_hour) = create_signal(8u32);
    let (daily_start_minute, set_daily_start_minute) = create_signal(0u32);
    let (daily_summary_enabled, set_daily_summary_enabled) = create_signal(true);
    let (process_overnight, set_process_overnight) = create_signal(true);
    let (auto_cleanup_enabled, set_auto_cleanup_enabled) = create_signal(true);
    let (retention_days, set_retention_days) = create_signal(30u32);
    let (activate_agents, set_activate_agents) = create_signal(true);
    let (telegram_digest, set_telegram_digest) = create_signal(true);
    let (show_install_script, set_show_install_script) = create_signal(false);
    
    // Format time display
    let time_display = move || {
        let h = daily_start_hour.get();
        let m = daily_start_minute.get();
        let period = if h >= 12 { "PM" } else { "AM" };
        let hour_12 = if h == 0 { 12 } else if h > 12 { h - 12 } else { h };
        format!("{:02}:{:02} {}", hour_12, m, period)
    };

    view! {
        <div class="page settings-page">
            <h1 class="page-title">"Settings"</h1>
            
            <div class="settings-sections">
                // Auto-Start & Scheduling Section
                <div class="settings-section scheduler-section">
                    <div class="section-header-row">
                        <h2 class="section-title">
                            <svg viewBox="0 0 24 24" fill="currentColor" class="section-icon">
                                <path d="M11.99 2C6.47 2 2 6.48 2 12s4.47 10 9.99 10C17.52 22 22 17.52 22 12S17.52 2 11.99 2zM12 20c-4.42 0-8-3.58-8-8s3.58-8 8-8 8 3.58 8 8-3.58 8-8 8zm.5-13H11v6l5.25 3.15.75-1.23-4.5-2.67z"/>
                            </svg>
                            "Auto-Start & Scheduling"
                        </h2>
                        <label class="toggle-switch">
                            <input 
                                type="checkbox" 
                                checked=move || auto_start_enabled.get()
                                on:change=move |e| set_auto_start_enabled.set(event_target_checked(&e))
                            />
                            <span class="toggle-slider"></span>
                        </label>
                    </div>
                    
                    <p class="section-description">
                        "Configure when Amail activates daily. The OS scheduler starts the server, and these settings control what happens at the scheduled time."
                    </p>
                    
                    <Show when=move || auto_start_enabled.get()>
                        <div class="scheduler-settings">
                            // Time picker
                            <div class="time-picker-group">
                                <label class="form-label">"Daily Start Time"</label>
                                <div class="time-picker">
                                    <select 
                                        class="time-select hour-select"
                                        on:change=move |e| {
                                            let val: u32 = event_target_value(&e).parse().unwrap_or(8);
                                            set_daily_start_hour.set(val);
                                        }
                                    >
                                        {(0..24).map(|h| {
                                            let period = if h >= 12 { "PM" } else { "AM" };
                                            let hour_12 = if h == 0 { 12 } else if h > 12 { h - 12 } else { h };
                                            let label = format!("{} {}", hour_12, period);
                                            view! {
                                                <option value=h.to_string() selected=move || daily_start_hour.get() == h>
                                                    {label}
                                                </option>
                                            }
                                        }).collect_view()}
                                    </select>
                                    <span class="time-separator">":"</span>
                                    <select 
                                        class="time-select minute-select"
                                        on:change=move |e| {
                                            let val: u32 = event_target_value(&e).parse().unwrap_or(0);
                                            set_daily_start_minute.set(val);
                                        }
                                    >
                                        {[0, 15, 30, 45].into_iter().map(|m| {
                                            view! {
                                                <option value=m.to_string() selected=move || daily_start_minute.get() == m>
                                                    {format!("{:02}", m)}
                                                </option>
                                            }
                                        }).collect_view()}
                                    </select>
                                </div>
                                <span class="current-time-display">"Currently set to " {time_display}</span>
                            </div>
                            
                            // Feature toggles
                            <div class="feature-toggles">
                                <h4 class="toggles-title">"Features to activate at " {time_display}</h4>
                                
                                <label class="feature-toggle">
                                    <input 
                                        type="checkbox" 
                                        checked=move || activate_agents.get()
                                        on:change=move |e| set_activate_agents.set(event_target_checked(&e))
                                    />
                                    <span class="feature-check"></span>
                                    <div class="feature-info">
                                        <span class="feature-name">"Activate AI Agents"</span>
                                        <span class="feature-desc">"Start all configured AI agents"</span>
                                    </div>
                                </label>
                                
                                <label class="feature-toggle">
                                    <input 
                                        type="checkbox" 
                                        checked=move || process_overnight.get()
                                        on:change=move |e| set_process_overnight.set(event_target_checked(&e))
                                    />
                                    <span class="feature-check"></span>
                                    <div class="feature-info">
                                        <span class="feature-name">"Process Overnight Emails"</span>
                                        <span class="feature-desc">"Route emails received while server was off"</span>
                                    </div>
                                </label>
                                
                                <label class="feature-toggle">
                                    <input 
                                        type="checkbox" 
                                        checked=move || daily_summary_enabled.get()
                                        on:change=move |e| set_daily_summary_enabled.set(event_target_checked(&e))
                                    />
                                    <span class="feature-check"></span>
                                    <div class="feature-info">
                                        <span class="feature-name">"Generate Daily Summary"</span>
                                        <span class="feature-desc">"Create KPI report for the previous day"</span>
                                    </div>
                                </label>
                                
                                <label class="feature-toggle">
                                    <input 
                                        type="checkbox" 
                                        checked=move || telegram_digest.get()
                                        on:change=move |e| set_telegram_digest.set(event_target_checked(&e))
                                    />
                                    <span class="feature-check"></span>
                                    <div class="feature-info">
                                        <span class="feature-name">"Send Telegram Digest"</span>
                                        <span class="feature-desc">"Send daily summary to Telegram"</span>
                                    </div>
                                </label>
                                
                                <label class="feature-toggle">
                                    <input 
                                        type="checkbox" 
                                        checked=move || auto_cleanup_enabled.get()
                                        on:change=move |e| set_auto_cleanup_enabled.set(event_target_checked(&e))
                                    />
                                    <span class="feature-check"></span>
                                    <div class="feature-info">
                                        <span class="feature-name">"Auto Cleanup"</span>
                                        <span class="feature-desc">"Remove old data beyond retention period"</span>
                                    </div>
                                </label>
                            </div>
                            
                            // Retention setting
                            <Show when=move || auto_cleanup_enabled.get()>
                                <div class="retention-setting">
                                    <label class="form-label">"Data Retention Period"</label>
                                    <div class="retention-input">
                                        <input 
                                            type="number" 
                                            min="7" 
                                            max="365"
                                            class="form-input retention-days"
                                            prop:value=move || retention_days.get()
                                            on:input=move |e| {
                                                let val: u32 = event_target_value(&e).parse().unwrap_or(30);
                                                set_retention_days.set(val.clamp(7, 365));
                                            }
                                        />
                                        <span class="retention-label">"days"</span>
                                    </div>
                                </div>
                            </Show>
                            
                            // OS Scheduler Installation
                            <div class="os-scheduler-section">
                                <h4 class="os-scheduler-title">"OS Scheduler Setup"</h4>
                                <p class="os-scheduler-desc">
                                    "To start Amail automatically at boot and at your scheduled time, install the OS scheduler task."
                                </p>
                                <div class="os-scheduler-buttons">
                                    <button 
                                        class="btn btn-secondary"
                                        on:click=move |_| set_show_install_script.update(|v| *v = !*v)
                                    >
                                        <svg viewBox="0 0 24 24" fill="currentColor" class="btn-icon">
                                            <path d="M9.4 16.6L4.8 12l4.6-4.6L8 6l-6 6 6 6 1.4-1.4zm5.2 0l4.6-4.6-4.6-4.6L16 6l6 6-6 6-1.4-1.4z"/>
                                        </svg>
                                        {move || if show_install_script.get() { "Hide Script" } else { "Show Install Script" }}
                                    </button>
                                    <button class="btn btn-primary">
                                        <svg viewBox="0 0 24 24" fill="currentColor" class="btn-icon">
                                            <path d="M19 9h-4V3H9v6H5l7 7 7-7zM5 18v2h14v-2H5z"/>
                                        </svg>
                                        "Download Script"
                                    </button>
                                </div>
                                
                                <Show when=move || show_install_script.get()>
                                    <div class="install-script-preview">
                                        <div class="script-tabs">
                                            <button class="script-tab active">"Windows"</button>
                                            <button class="script-tab">"Linux"</button>
                                            <button class="script-tab">"macOS"</button>
                                        </div>
                                        <pre class="script-code">
{format!(r#"# Amail Auto-Start Installation Script
# Run as Administrator

$TaskName = "Amail-AutoStart"
$ExePath = "C:\path\to\amail-server.exe"

# Create triggers
$DailyTrigger = New-ScheduledTaskTrigger -Daily -At "{:02}:{:02}"
$BootTrigger = New-ScheduledTaskTrigger -AtStartup

# Register task
Register-ScheduledTask -TaskName $TaskName `
    -Action (New-ScheduledTaskAction -Execute $ExePath) `
    -Trigger $DailyTrigger,$BootTrigger `
    -Description "Start Amail AI Mail Server"

Write-Host "Amail auto-start installed!"
"#, daily_start_hour.get(), daily_start_minute.get())}
                                        </pre>
                                    </div>
                                </Show>
                            </div>
                        </div>
                    </Show>
                </div>
                
                // Server Settings
                <div class="settings-section">
                    <h2 class="section-title">"Server Configuration"</h2>
                    <div class="settings-grid">
                        <div class="form-group">
                            <label for="smtp-port">"SMTP Port"</label>
                            <input
                                type="text"
                                id="smtp-port"
                                class="form-input"
                                prop:value=move || smtp_port.get()
                                on:input=move |e| set_smtp_port.set(event_target_value(&e))
                            />
                            <span class="form-hint">"Port for incoming SMTP connections"</span>
                        </div>
                        
                        <div class="form-group">
                            <label for="http-port">"HTTP Port"</label>
                            <input
                                type="text"
                                id="http-port"
                                class="form-input"
                                prop:value=move || http_port.get()
                                on:input=move |e| set_http_port.set(event_target_value(&e))
                            />
                            <span class="form-hint">"Port for HTTP API and dashboard"</span>
                        </div>
                    </div>
                </div>
                
                // LLM Settings
                <div class="settings-section">
                    <h2 class="section-title">"LLM Configuration"</h2>
                    <p class="section-description">
                        "Configure multiple Ollama instances for load balancing. More instances = higher throughput for 30+ agents."
                    </p>
                    
                    <div class="ollama-urls-section">
                        <label class="form-label">"Ollama Instances"</label>
                        <div class="ollama-url-list">
                            {move || ollama_urls.get().into_iter().enumerate().map(|(idx, url)| {
                                let url_display = url.clone();
                                view! {
                                    <div class="ollama-url-item">
                                        <span class="ollama-url-badge">{idx + 1}</span>
                                        <span class="ollama-url-text">{url_display}</span>
                                        <button 
                                            class="btn btn-xs btn-danger"
                                            on:click=move |_| {
                                                set_ollama_urls.update(|urls| {
                                                    if urls.len() > 1 {
                                                        urls.remove(idx);
                                                    }
                                                });
                                            }
                                            disabled=move || ollama_urls.get().len() <= 1
                                        >
                                            "Remove"
                                        </button>
                                    </div>
                                }
                            }).collect_view()}
                        </div>
                        
                        <div class="add-ollama-url">
                            <input
                                type="text"
                                class="form-input"
                                placeholder="http://localhost:11435"
                                prop:value=move || new_ollama_url.get()
                                on:input=move |e| set_new_ollama_url.set(event_target_value(&e))
                            />
                            <button 
                                class="btn btn-secondary"
                                on:click=move |_| {
                                    let url = new_ollama_url.get();
                                    if !url.is_empty() && url.starts_with("http") {
                                        set_ollama_urls.update(|urls| urls.push(url.clone()));
                                        set_new_ollama_url.set(String::new());
                                    }
                                }
                            >
                                "Add Instance"
                            </button>
                        </div>
                        <span class="form-hint">"Add multiple Ollama URLs for load balancing (e.g., localhost:11434, localhost:11435, localhost:11436)"</span>
                    </div>
                    
                    <div class="settings-grid" style="margin-top: 16px;">
                        <div class="form-group">
                            <label for="llm-model">"Default Model"</label>
                            <select
                                id="llm-model"
                                class="form-select"
                                on:change=move |e| set_llm_model.set(event_target_value(&e))
                            >
                                <option value="mistral" selected=move || llm_model.get() == "mistral">"Mistral"</option>
                                <option value="llama3" selected=move || llm_model.get() == "llama3">"Llama 3"</option>
                                <option value="codellama" selected=move || llm_model.get() == "codellama">"Code Llama"</option>
                                <option value="phi3" selected=move || llm_model.get() == "phi3">"Phi-3"</option>
                            </select>
                            <span class="form-hint">"Model used for agent reasoning"</span>
                        </div>
                        
                        <div class="form-group">
                            <label>"Instance Count"</label>
                            <div class="instance-count-display">
                                <span class="instance-count">{move || ollama_urls.get().len()}</span>
                                <span class="instance-label">" Ollama instance(s) configured"</span>
                            </div>
                        </div>
                    </div>
                </div>
                
                // Email Settings
                <div class="settings-section">
                    <h2 class="section-title">"Email Settings"</h2>
                    <div class="settings-grid">
                        <div class="form-group full-width">
                            <label for="mailboxes">"Mailboxes"</label>
                            <input
                                type="text"
                                id="mailboxes"
                                class="form-input"
                                value="inbox,support,hr,legal,finance,executive"
                            />
                            <span class="form-hint">"Comma-separated list of mailboxes to monitor"</span>
                        </div>
                    </div>
                </div>
                
                // Danger Zone
                <div class="settings-section danger-zone">
                    <h2 class="section-title">"Danger Zone"</h2>
                    <div class="danger-actions">
                        <div class="danger-action">
                            <div class="danger-info">
                                <h4>"Reset All Agents"</h4>
                                <p>"Restart all agents and clear their state"</p>
                            </div>
                            <button class="btn btn-danger">"Reset Agents"</button>
                        </div>
                        <div class="danger-action">
                            <div class="danger-info">
                                <h4>"Clear Task Queue"</h4>
                                <p>"Remove all pending tasks from the queue"</p>
                            </div>
                            <button class="btn btn-danger">"Clear Queue"</button>
                        </div>
                    </div>
                </div>
            </div>
            
            <div class="settings-footer">
                <button class="btn btn-secondary">"Cancel"</button>
                <button class="btn btn-primary">"Save Changes"</button>
            </div>
        </div>
    }
}
