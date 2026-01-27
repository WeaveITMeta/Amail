//! Workspace Rules Page - General rules for all AI agents
//!
//! This page allows editing the workspace_rules.md file that applies to all agents.

use leptos::*;
use wasm_bindgen_futures::spawn_local;
use crate::api;

#[component]
pub fn WorkspaceRulesPage() -> impl IntoView {
    // State
    let (rules_content, set_rules_content) = create_signal(String::new());
    let (loading, set_loading) = create_signal(true);
    let (saving, set_saving) = create_signal(false);
    let (error, set_error) = create_signal::<Option<String>>(None);
    let (success, set_success) = create_signal::<Option<String>>(None);
    let (last_modified, set_last_modified) = create_signal::<Option<String>>(None);
    let (has_changes, set_has_changes) = create_signal(false);
    let (original_content, set_original_content) = create_signal(String::new());
    
    // Fetch rules on mount
    {
        let set_rules_content = set_rules_content.clone();
        let set_original_content = set_original_content.clone();
        let set_loading = set_loading.clone();
        let set_error = set_error.clone();
        let set_last_modified = set_last_modified.clone();
        
        spawn_local(async move {
            match api::fetch_workspace_rules().await {
                Ok(response) => {
                    set_rules_content.set(response.content.clone());
                    set_original_content.set(response.content);
                    set_last_modified.set(response.last_modified);
                }
                Err(e) => {
                    set_error.set(Some(format!("Failed to load workspace rules: {}", e)));
                }
            }
            set_loading.set(false);
        });
    }
    
    // Track changes
    let on_content_change = move |e: ev::Event| {
        let value = event_target_value(&e);
        set_rules_content.set(value.clone());
        set_has_changes.set(value != original_content.get());
        set_error.set(None);
        set_success.set(None);
    };
    
    // Save handler
    let save_rules = move |_| {
        set_saving.set(true);
        set_error.set(None);
        set_success.set(None);
        
        let content = rules_content.get();
        let set_saving = set_saving.clone();
        let set_error = set_error.clone();
        let set_success = set_success.clone();
        let set_has_changes = set_has_changes.clone();
        let set_original_content = set_original_content.clone();
        
        spawn_local(async move {
            match api::update_workspace_rules(&content).await {
                Ok(_) => {
                    set_success.set(Some("Workspace rules saved successfully!".to_string()));
                    set_original_content.set(content);
                    set_has_changes.set(false);
                }
                Err(e) => {
                    set_error.set(Some(format!("Failed to save: {}", e)));
                }
            }
            set_saving.set(false);
        });
    };
    
    // Reset handler
    let reset_changes = move |_| {
        set_rules_content.set(original_content.get());
        set_has_changes.set(false);
        set_error.set(None);
        set_success.set(None);
    };

    view! {
        <div class="page workspace-rules-page">
            <div class="page-header">
                <div class="page-header-content">
                    <h1 class="page-title">
                        <svg viewBox="0 0 24 24" fill="currentColor" class="page-icon">
                            <path d="M14 2H6c-1.1 0-1.99.9-1.99 2L4 20c0 1.1.89 2 1.99 2H18c1.1 0 2-.9 2-2V8l-6-6zm2 16H8v-2h8v2zm0-4H8v-2h8v2zm-3-5V3.5L18.5 9H13z"/>
                        </svg>
                        "Workspace Rules"
                    </h1>
                    <p class="page-description">
                        "Define general rules that apply to ALL AI agents. These guidelines supplement each agent's role-specific training."
                    </p>
                </div>
                <div class="page-actions">
                    <Show when=move || has_changes.get()>
                        <button 
                            class="btn btn-secondary"
                            on:click=reset_changes
                            disabled=move || saving.get()
                        >
                            "Discard Changes"
                        </button>
                    </Show>
                    <button 
                        class="btn btn-primary"
                        on:click=save_rules
                        disabled=move || saving.get() || !has_changes.get()
                    >
                        <svg viewBox="0 0 24 24" fill="currentColor" class="btn-icon">
                            <path d="M17 3H5c-1.11 0-2 .9-2 2v14c0 1.1.89 2 2 2h14c1.1 0 2-.9 2-2V7l-4-4zm-5 16c-1.66 0-3-1.34-3-3s1.34-3 3-3 3 1.34 3 3-1.34 3-3 3zm3-10H5V5h10v4z"/>
                        </svg>
                        {move || if saving.get() { "Saving..." } else { "Save Rules" }}
                    </button>
                </div>
            </div>
            
            // Status messages
            <Show when=move || error.get().is_some()>
                <div class="alert alert-error">
                    <svg viewBox="0 0 24 24" fill="currentColor" class="alert-icon">
                        <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-2h2v2zm0-4h-2V7h2v6z"/>
                    </svg>
                    {move || error.get().unwrap_or_default()}
                </div>
            </Show>
            
            <Show when=move || success.get().is_some()>
                <div class="alert alert-success">
                    <svg viewBox="0 0 24 24" fill="currentColor" class="alert-icon">
                        <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z"/>
                    </svg>
                    {move || success.get().unwrap_or_default()}
                </div>
            </Show>
            
            // Loading state
            <Show when=move || loading.get()>
                <div class="loading-state">
                    <p>"Loading workspace rules..."</p>
                </div>
            </Show>
            
            // Editor
            <Show when=move || !loading.get()>
                <div class="rules-editor-container">
                    <div class="editor-info">
                        <span class="editor-label">"Markdown Editor"</span>
                        <Show when=move || last_modified.get().is_some()>
                            <span class="last-modified">
                                "Last modified: "{move || last_modified.get().unwrap_or_default()}
                            </span>
                        </Show>
                        <Show when=move || has_changes.get()>
                            <span class="unsaved-badge">"Unsaved changes"</span>
                        </Show>
                    </div>
                    <textarea
                        class="rules-editor"
                        prop:value=move || rules_content.get()
                        on:input=on_content_change
                        disabled=move || saving.get()
                        placeholder="# Workspace Rules\n\nDefine rules for your AI agents here..."
                        spellcheck="false"
                    />
                    <div class="editor-footer">
                        <span class="char-count">
                            {move || rules_content.get().len()}" characters"
                        </span>
                        <span class="editor-hint">
                            "Use Markdown formatting. Changes apply to all agents on save."
                        </span>
                    </div>
                </div>
            </Show>
            
            // Help section
            <div class="rules-help">
                <h3>"Markdown Tips"</h3>
                <div class="help-grid">
                    <div class="help-item">
                        <code>"# Heading"</code>
                        <span>"Main section"</span>
                    </div>
                    <div class="help-item">
                        <code>"## Subheading"</code>
                        <span>"Subsection"</span>
                    </div>
                    <div class="help-item">
                        <code>"- Item"</code>
                        <span>"Bullet list"</span>
                    </div>
                    <div class="help-item">
                        <code>"**bold**"</code>
                        <span>"Bold text"</span>
                    </div>
                    <div class="help-item">
                        <code>"*italic*"</code>
                        <span>"Italic text"</span>
                    </div>
                    <div class="help-item">
                        <code>"`code`"</code>
                        <span>"Inline code"</span>
                    </div>
                </div>
            </div>
        </div>
    }
}
