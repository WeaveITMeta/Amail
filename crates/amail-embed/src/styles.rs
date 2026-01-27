//! Embedded CSS Styles for Amail Widgets
//!
//! These styles are injected into the page when widgets are mounted.
//! They use CSS custom properties for theming.

/// Get the complete CSS for all widgets
pub fn get_widget_styles() -> &'static str {
    r#"
/* =============================================================================
   Amail Embed - Widget Styles
   ============================================================================= */

/* CSS Variables (can be overridden) */
:root {
    --amail-primary: #3b82f6;
    --amail-primary-hover: #2563eb;
    --amail-bg: #ffffff;
    --amail-bg-secondary: #f8fafc;
    --amail-text: #1e293b;
    --amail-text-muted: #64748b;
    --amail-border: #e2e8f0;
    --amail-radius: 8px;
    --amail-font: system-ui, -apple-system, sans-serif;
    --amail-success: #10b981;
    --amail-error: #ef4444;
    --amail-warning: #f59e0b;
}

/* =============================================================================
   Inbox Widget
   ============================================================================= */
.amail-inbox {
    font-family: var(--amail-font);
    background: var(--amail-bg);
    border: 1px solid var(--amail-border);
    border-radius: var(--amail-radius);
    overflow: hidden;
    max-width: 400px;
}

.amail-inbox.amail-compact {
    max-width: 320px;
}

.amail-inbox-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 16px;
    background: var(--amail-bg-secondary);
    border-bottom: 1px solid var(--amail-border);
}

.amail-inbox-title {
    display: flex;
    align-items: center;
    gap: 8px;
    font-weight: 600;
    font-size: 14px;
    color: var(--amail-text);
}

.amail-inbox-title svg {
    width: 18px;
    height: 18px;
    color: var(--amail-primary);
}

.amail-unread-badge {
    background: var(--amail-primary);
    color: white;
    font-size: 11px;
    font-weight: 600;
    padding: 2px 8px;
    border-radius: 10px;
}

.amail-inbox-list {
    max-height: 300px;
    overflow-y: auto;
}

.amail-email-item {
    padding: 12px 16px;
    border-bottom: 1px solid var(--amail-border);
    cursor: pointer;
    transition: background 0.15s;
}

.amail-email-item:hover {
    background: var(--amail-bg-secondary);
}

.amail-email-item.unread {
    background: #eff6ff;
}

.amail-email-item.unread:hover {
    background: #dbeafe;
}

.amail-email-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 4px;
}

.amail-email-from {
    font-size: 13px;
    font-weight: 500;
    color: var(--amail-text);
}

.amail-email-time {
    font-size: 11px;
    color: var(--amail-text-muted);
}

.amail-email-subject {
    font-size: 13px;
    color: var(--amail-text-muted);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
}

.amail-email-item.unread .amail-email-subject {
    color: var(--amail-text);
    font-weight: 500;
}

.amail-email-preview {
    margin-top: 8px;
    padding-top: 8px;
    border-top: 1px dashed var(--amail-border);
    font-size: 12px;
    color: var(--amail-text-muted);
    line-height: 1.5;
}

.amail-inbox-footer {
    padding: 10px 16px;
    text-align: center;
    border-top: 1px solid var(--amail-border);
}

.amail-view-all {
    font-size: 12px;
    color: var(--amail-primary);
    text-decoration: none;
}

.amail-view-all:hover {
    text-decoration: underline;
}

.amail-loading, .amail-empty {
    padding: 24px;
    text-align: center;
    color: var(--amail-text-muted);
    font-size: 13px;
}

/* =============================================================================
   Contact Form Widget
   ============================================================================= */
.amail-contact-form {
    font-family: var(--amail-font);
    background: var(--amail-bg);
    border: 1px solid var(--amail-border);
    border-radius: var(--amail-radius);
    padding: 20px;
    max-width: 480px;
}

.amail-contact-form.amail-compact {
    padding: 16px;
    max-width: 360px;
}

.amail-form-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 16px;
}

.amail-form-title {
    font-size: 18px;
    font-weight: 600;
    color: var(--amail-text);
    margin: 0;
}

.amail-ai-badge {
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: 11px;
    color: var(--amail-primary);
    background: #eff6ff;
    padding: 4px 8px;
    border-radius: 12px;
}

.amail-ai-badge svg {
    width: 14px;
    height: 14px;
}

.amail-form {
    display: flex;
    flex-direction: column;
    gap: 12px;
}

.amail-form-row {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 12px;
}

.amail-form-group {
    display: flex;
    flex-direction: column;
    gap: 4px;
}

.amail-form-group label {
    font-size: 13px;
    font-weight: 500;
    color: var(--amail-text);
}

.amail-form-group input,
.amail-form-group textarea {
    padding: 10px 12px;
    border: 1px solid var(--amail-border);
    border-radius: calc(var(--amail-radius) - 2px);
    font-size: 14px;
    font-family: inherit;
    transition: border-color 0.15s;
}

.amail-form-group input:focus,
.amail-form-group textarea:focus {
    outline: none;
    border-color: var(--amail-primary);
}

.amail-form-group textarea {
    resize: vertical;
    min-height: 80px;
}

.amail-submit-btn {
    padding: 12px 20px;
    background: var(--amail-primary);
    color: white;
    border: none;
    border-radius: calc(var(--amail-radius) - 2px);
    font-size: 14px;
    font-weight: 500;
    cursor: pointer;
    transition: background 0.15s;
}

.amail-submit-btn:hover:not(:disabled) {
    background: var(--amail-primary-hover);
}

.amail-submit-btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
}

.amail-success {
    text-align: center;
    padding: 24px;
}

.amail-success svg {
    width: 48px;
    height: 48px;
    color: var(--amail-success);
    margin-bottom: 12px;
}

.amail-success h4 {
    font-size: 18px;
    margin: 0 0 8px;
    color: var(--amail-text);
}

.amail-success p {
    font-size: 13px;
    color: var(--amail-text-muted);
    margin: 0 0 16px;
}

.amail-reset-btn {
    padding: 8px 16px;
    background: transparent;
    color: var(--amail-primary);
    border: 1px solid var(--amail-primary);
    border-radius: calc(var(--amail-radius) - 2px);
    font-size: 13px;
    cursor: pointer;
}

.amail-reset-btn:hover {
    background: #eff6ff;
}

.amail-error {
    padding: 10px;
    background: #fef2f2;
    color: var(--amail-error);
    border-radius: calc(var(--amail-radius) - 2px);
    font-size: 13px;
}

.amail-form-footer {
    margin-top: 12px;
    text-align: center;
}

.amail-powered-by {
    font-size: 11px;
    color: var(--amail-text-muted);
}

.amail-powered-by a {
    color: var(--amail-primary);
    text-decoration: none;
}

/* =============================================================================
   Status Badge Widget
   ============================================================================= */
.amail-status {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    font-family: var(--amail-font);
    font-size: 12px;
    padding: 4px 10px;
    border-radius: 12px;
    cursor: default;
    position: relative;
}

.amail-status.compact {
    padding: 0;
    background: none;
}

.amail-status.online {
    background: #ecfdf5;
    color: var(--amail-success);
}

.amail-status.offline {
    background: #fef2f2;
    color: var(--amail-error);
}

.amail-status.degraded {
    background: #fffbeb;
    color: var(--amail-warning);
}

.amail-status.unknown {
    background: var(--amail-bg-secondary);
    color: var(--amail-text-muted);
}

.amail-status-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    animation: amail-pulse 2s infinite;
}

.amail-status.online .amail-status-dot {
    background: var(--amail-success);
}

.amail-status.offline .amail-status-dot {
    background: var(--amail-error);
    animation: none;
}

.amail-status.degraded .amail-status-dot {
    background: var(--amail-warning);
}

.amail-status.unknown .amail-status-dot {
    background: var(--amail-text-muted);
    animation: none;
}

@keyframes amail-pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.5; }
}

.amail-status-label {
    font-weight: 500;
}

.amail-status-tooltip {
    display: none;
    position: absolute;
    bottom: 100%;
    left: 50%;
    transform: translateX(-50%);
    margin-bottom: 8px;
    background: var(--amail-text);
    color: white;
    padding: 12px;
    border-radius: var(--amail-radius);
    font-size: 12px;
    min-width: 200px;
    z-index: 1000;
}

.amail-status:hover .amail-status-tooltip {
    display: block;
}

.tooltip-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 8px;
    padding-bottom: 8px;
    border-bottom: 1px solid rgba(255,255,255,0.2);
}

.tooltip-title {
    font-weight: 600;
}

.tooltip-status {
    padding: 2px 6px;
    border-radius: 4px;
    font-size: 10px;
    text-transform: uppercase;
}

.tooltip-status.online { background: var(--amail-success); }
.tooltip-status.offline { background: var(--amail-error); }
.tooltip-status.degraded { background: var(--amail-warning); }

.tooltip-details {
    display: flex;
    flex-direction: column;
    gap: 4px;
}

.tooltip-row {
    display: flex;
    justify-content: space-between;
    gap: 12px;
}

.tooltip-value {
    opacity: 0.8;
    font-family: monospace;
    font-size: 11px;
}
"#
}

/// Inject styles into the document head
pub fn inject_styles() {
    use web_sys::window;
    
    if let Some(window) = window() {
        if let Some(document) = window.document() {
            if let Some(head) = document.head() {
                let style = document.create_element("style").unwrap();
                style.set_text_content(Some(get_widget_styles()));
                style.set_attribute("data-amail-embed", "true").unwrap();
                head.append_child(&style).unwrap();
            }
        }
    }
}
