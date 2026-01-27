# Amail Embed

Embeddable Leptos components for integrating Amail functionality into external applications.

## Features

- **Inbox Widget** - Mini email inbox for external sites
- **Contact Form** - AI-powered contact form with smart routing
- **Status Badge** - Real-time server status indicator

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
amail-embed = { git = "https://github.com/your-repo/amail", features = ["full"] }
```

## Usage in Leptos Apps

```rust
use leptos::*;
use amail_embed::{AmailInbox, AmailContactForm, AmailStatusBadge};

#[component]
fn MyApp() -> impl IntoView {
    view! {
        // Mini inbox widget
        <AmailInbox 
            api_url="https://mail.example.com" 
            mailbox="support"
            max_items=5
        />
        
        // AI-powered contact form
        <AmailContactForm 
            api_url="https://mail.example.com" 
            to="contact@example.com"
            title="Get in Touch"
        />
        
        // Server status badge
        <AmailStatusBadge 
            api_url="https://mail.example.com"
        />
    }
}
```

## JavaScript/WASM Usage

For non-Leptos apps, you can use the WASM bundle directly:

```html
<script type="module">
    import init, { mount_inbox, mount_contact_form, mount_status_badge } from './amail_embed.js';
    
    await init();
    
    // Mount widgets to DOM elements
    mount_inbox('inbox-container', 'https://mail.example.com', 'support');
    mount_contact_form('contact-container', 'https://mail.example.com', 'contact@example.com');
    mount_status_badge('status-container', 'https://mail.example.com');
</script>

<div id="inbox-container"></div>
<div id="contact-container"></div>
<div id="status-container"></div>
```

## Theming

Override CSS variables to customize the look:

```css
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
}
```

## Widget Props

### AmailInbox

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `api_url` | String | required | Amail server URL |
| `mailbox` | String | required | Mailbox name |
| `max_items` | usize | 5 | Max emails to show |
| `show_header` | bool | true | Show header bar |
| `compact` | bool | false | Compact mode |

### AmailContactForm

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `api_url` | String | required | Amail server URL |
| `to` | String | required | Recipient email |
| `title` | String | "Contact Us" | Form title |
| `show_ai_badge` | bool | true | Show AI badge |
| `compact` | bool | false | Compact mode |

### AmailStatusBadge

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `api_url` | String | required | Amail server URL |
| `show_label` | bool | true | Show status text |
| `compact` | bool | false | Dot only mode |

## Building

```bash
# Build WASM bundle
cd crates/amail-embed
trunk build --release

# Or with wasm-pack for npm
wasm-pack build --target web
```

## License

MIT - Part of the Amail project (100% Open Source, FREE forever)
