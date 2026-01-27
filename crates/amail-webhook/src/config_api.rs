// =============================================================================
// Config API - Unified Storage (Supabase primary, JSON fallback)
// =============================================================================
// These endpoints use UnifiedStorage which automatically selects:
// - Supabase/PostgreSQL if DATABASE_URL is configured
// - JSON files in data/config/ as fallback
// =============================================================================

use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use sqlx::PgPool;

use amail_core::config_manager::{
    ConfigManager, DomainConfig, MailboxConfig, DomainSettings, DkimConfig, 
    DomainBranding, MailboxSettings, MailboxAgentConfig,
};
use crate::storage::UnifiedStorage;
use crate::stalwart::StalwartClient;

// =============================================================================
// Config State - Now uses UnifiedStorage
// =============================================================================

#[derive(Clone)]
pub struct ConfigState {
    pub config_manager: Arc<ConfigManager>,
    pub storage: Arc<UnifiedStorage>,
    pub stalwart: Option<Arc<StalwartClient>>,
}

impl ConfigState {
    pub fn new(config_manager: Arc<ConfigManager>) -> Self {
        // Create UnifiedStorage with no database (JSON only)
        let storage = Arc::new(UnifiedStorage::new(None, config_manager.clone()));
        // Try to create StalwartClient for syncing
        let stalwart = Self::create_stalwart_client();
        Self { config_manager, storage, stalwart }
    }
    
    pub fn with_database(config_manager: Arc<ConfigManager>, pool: PgPool) -> Self {
        // Create UnifiedStorage with database (Supabase primary)
        let storage = Arc::new(UnifiedStorage::new(Some(pool), config_manager.clone()));
        // Try to create StalwartClient for syncing
        let stalwart = Self::create_stalwart_client();
        Self { config_manager, storage, stalwart }
    }

    fn create_stalwart_client() -> Option<Arc<StalwartClient>> {
        let stalwart_url = std::env::var("STALWART_URL").unwrap_or_else(|_| "http://localhost:4040".to_string());
        let stalwart_key = std::env::var("STALWART_API_KEY").unwrap_or_default();
        match StalwartClient::new(&stalwart_url, &stalwart_key) {
            Ok(client) => {
                tracing::info!("StalwartClient initialized for domain/mailbox sync");
                Some(Arc::new(client))
            }
            Err(e) => {
                tracing::warn!("Failed to create StalwartClient: {} - mailboxes won't sync to mail server", e);
                None
            }
        }
    }
}

// =============================================================================
// Domain Endpoints
// =============================================================================

#[derive(Debug, Serialize)]
pub struct DomainResponse {
    pub id: String,
    pub name: String,
    pub display_name: String,
    pub enabled: bool,
    pub primary: bool,
    pub created_at: String,
    pub status: String,
    pub mx_verified: bool,
    pub spf_verified: bool,
    pub dkim_verified: bool,
    pub dmarc_verified: bool,
    pub mailbox_count: i64,
}

impl From<DomainConfig> for DomainResponse {
    fn from(d: DomainConfig) -> Self {
        Self {
            id: d.id,
            name: d.name,
            display_name: d.display_name,
            enabled: d.enabled,
            primary: d.primary,
            created_at: d.created_at.to_rfc3339(),
            status: if d.enabled { "active".to_string() } else { "disabled".to_string() },
            mx_verified: false,
            spf_verified: false,
            dkim_verified: false,
            dmarc_verified: false,
            mailbox_count: 0,
        }
    }
}

pub async fn list_domains(state: web::Data<ConfigState>) -> impl Responder {
    tracing::info!("Listing domains using {} backend", state.storage.backend_name());
    match state.storage.list_domains().await {
        Ok(domains) => {
            let mut responses: Vec<DomainResponse> = Vec::new();
            for d in domains {
                // Get actual mailbox count for this domain
                let mailbox_count = match state.storage.list_mailboxes_by_domain(&d.id).await {
                    Ok(mailboxes) => mailboxes.len() as i64,
                    Err(_) => 0,
                };
                
                responses.push(DomainResponse {
                    id: d.id,
                    name: d.name,
                    display_name: d.display_name,
                    enabled: d.enabled,
                    primary: d.primary,
                    created_at: d.created_at.to_rfc3339(),
                    status: if d.enabled { "active".to_string() } else { "disabled".to_string() },
                    mx_verified: d.mx_verified,
                    spf_verified: d.spf_verified,
                    dkim_verified: d.dkim_verified,
                    dmarc_verified: d.dmarc_verified,
                    mailbox_count,
                });
            }
            HttpResponse::Ok().json(responses)
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e
        })),
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateDomainRequest {
    pub name: String,
    pub display_name: Option<String>,
}

pub async fn create_domain(
    state: web::Data<ConfigState>,
    body: web::Json<CreateDomainRequest>,
) -> impl Responder {
    // Normalize domain name to lowercase for case-insensitive storage
    let normalized_name = body.name.trim().to_lowercase();
    tracing::info!("Creating domain '{}' using {} backend", normalized_name, state.storage.backend_name());
    
    match state.storage.create_domain(&normalized_name).await {
        Ok(created) => {
            tracing::info!("Domain '{}' created successfully", created.name);
            
            // Sync domain to Stalwart mail server
            if let Some(ref stalwart) = state.stalwart {
                if let Err(e) = stalwart.create_domain(&created.name).await {
                    tracing::warn!("Failed to sync domain to Stalwart: {} (continuing anyway)", e);
                } else {
                    tracing::info!("Domain '{}' synced to Stalwart", created.name);
                }
            }
            
            HttpResponse::Created().json(DomainResponse {
                id: created.id,
                name: created.name,
                display_name: created.display_name,
                enabled: created.enabled,
                primary: created.primary,
                created_at: created.created_at.to_rfc3339(),
                status: if created.enabled { "active".to_string() } else { "disabled".to_string() },
                mx_verified: created.mx_verified,
                spf_verified: created.spf_verified,
                dkim_verified: created.dkim_verified,
                dmarc_verified: created.dmarc_verified,
                mailbox_count: 0,
            })
        }
        Err(e) => {
            tracing::error!("Failed to create domain: {}", e);
            HttpResponse::BadRequest().json(serde_json::json!({
                "error": e
            }))
        }
    }
}

pub async fn get_domain(
    state: web::Data<ConfigState>,
    path: web::Path<String>,
) -> impl Responder {
    match state.storage.get_domain(&path).await {
        Ok(Some(domain)) => HttpResponse::Ok().json(DomainResponse {
            id: domain.id,
            name: domain.name,
            display_name: domain.display_name,
            enabled: domain.enabled,
            primary: domain.primary,
            created_at: domain.created_at.to_rfc3339(),
            status: if domain.enabled { "active".to_string() } else { "disabled".to_string() },
            mx_verified: domain.mx_verified,
            spf_verified: domain.spf_verified,
            dkim_verified: domain.dkim_verified,
            dmarc_verified: domain.dmarc_verified,
            mailbox_count: 0,
        }),
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Domain not found"
        })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e
        })),
    }
}

pub async fn delete_domain(
    state: web::Data<ConfigState>,
    path: web::Path<String>,
) -> impl Responder {
    match state.storage.delete_domain(&path).await {
        Ok(_) => HttpResponse::NoContent().finish(),
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({
            "error": e
        })),
    }
}

// =============================================================================
// Mailbox Endpoints
// =============================================================================

#[derive(Debug, Serialize)]
pub struct MailboxResponse {
    pub id: String,
    pub email: String,
    pub display_name: String,
    pub domain_id: String,
    pub mailbox_type: String,
    pub enabled: bool,
    pub created_at: String,
}

impl From<MailboxConfig> for MailboxResponse {
    fn from(m: MailboxConfig) -> Self {
        Self {
            id: m.id,
            email: m.email,
            display_name: m.display_name,
            domain_id: m.domain_id,
            mailbox_type: m.r#type,
            enabled: m.enabled,
            created_at: m.created_at.to_rfc3339(),
        }
    }
}

pub async fn list_mailboxes(state: web::Data<ConfigState>) -> impl Responder {
    match state.storage.list_mailboxes().await {
        Ok(mailboxes) => {
            let responses: Vec<MailboxResponse> = mailboxes.into_iter().map(|m| MailboxResponse {
                id: m.id,
                email: m.email,
                display_name: m.display_name,
                domain_id: m.domain_id,
                mailbox_type: "user".to_string(),
                enabled: m.enabled,
                created_at: m.created_at.to_rfc3339(),
            }).collect();
            HttpResponse::Ok().json(responses)
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({ "error": e })),
    }
}

pub async fn list_mailboxes_by_domain(
    state: web::Data<ConfigState>,
    path: web::Path<String>,
) -> impl Responder {
    match state.storage.list_mailboxes_by_domain(&path).await {
        Ok(mailboxes) => {
            let responses: Vec<MailboxResponse> = mailboxes.into_iter().map(|m| MailboxResponse {
                id: m.id,
                email: m.email,
                display_name: m.display_name,
                domain_id: m.domain_id,
                mailbox_type: "user".to_string(),
                enabled: m.enabled,
                created_at: m.created_at.to_rfc3339(),
            }).collect();
            HttpResponse::Ok().json(responses)
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({ "error": e })),
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateMailboxRequest {
    pub domain_id: String,
    pub local_part: String,
    pub display_name: Option<String>,
    pub mailbox_type: Option<String>,
    pub password: Option<String>,
}

pub async fn create_mailbox(
    state: web::Data<ConfigState>,
    body: web::Json<CreateMailboxRequest>,
) -> impl Responder {
    // Get domain to construct email
    let domain = match state.storage.get_domain(&body.domain_id).await {
        Ok(Some(d)) => d,
        Ok(None) => return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Domain not found"
        })),
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e
        })),
    };

    // Use display_name (Title Case) for email address, not name (lowercase)
    let email = format!("{}@{}", body.local_part.to_lowercase(), domain.display_name);
    
    // Title case the display name (capitalize first letter of each word)
    let display_name = body.display_name.clone().unwrap_or_else(|| body.local_part.clone());
    let display_name = display_name.split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().chain(chars).collect(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ");

    match state.storage.create_mailbox(&body.domain_id, &email, &display_name).await {
        Ok(created) => {
            // Sync mailbox to Stalwart mail server
            if let Some(ref stalwart) = state.stalwart {
                // Generate a default password for the mailbox
                let password = body.password.clone().unwrap_or_else(|| {
                    format!("{}123", body.local_part.to_lowercase())
                });
                if let Err(e) = stalwart.create_account(&created.email, &created.display_name, &password).await {
                    tracing::warn!("Failed to sync mailbox to Stalwart: {} (continuing anyway)", e);
                } else {
                    tracing::info!("Mailbox '{}' synced to Stalwart", created.email);
                }
            }
            
            HttpResponse::Created().json(MailboxResponse {
                id: created.id,
                email: created.email,
                display_name: created.display_name,
                domain_id: created.domain_id,
                mailbox_type: "user".to_string(),
                enabled: created.enabled,
                created_at: created.created_at.to_rfc3339(),
            })
        },
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({
            "error": e
        })),
    }
}

pub async fn delete_mailbox(
    state: web::Data<ConfigState>,
    path: web::Path<String>,
) -> impl Responder {
    // The mailbox ID is typically the email address or a UUID
    // We'll try to delete from Stalwart using the path as email if it contains @
    let mailbox_id = path.into_inner();
    
    match state.storage.delete_mailbox(&mailbox_id).await {
        Ok(_) => {
            // Sync deletion to Stalwart if the ID looks like an email
            if let Some(ref stalwart) = state.stalwart {
                // Try to delete - the ID might be the email or contain it
                if mailbox_id.contains('@') {
                    if let Err(e) = stalwart.delete_account(&mailbox_id).await {
                        tracing::warn!("Failed to delete mailbox from Stalwart: {}", e);
                    } else {
                        tracing::info!("Mailbox '{}' deleted from Stalwart", mailbox_id);
                    }
                }
            }
            HttpResponse::NoContent().finish()
        },
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({
            "error": e
        })),
    }
}

// =============================================================================
// DNS Endpoints
// =============================================================================

pub async fn get_dns_records(
    state: web::Data<ConfigState>,
    path: web::Path<String>,
) -> impl Responder {
    let domain_id = path.into_inner();
    
    // First check if domain exists via storage (Supabase first, JSON fallback)
    let domain_name = match state.storage.get_domain(&domain_id).await {
        Ok(Some(d)) => d.name,
        Ok(None) => {
            // Fallback to config_manager
            match state.config_manager.get_domain(&domain_id).await {
                Some(d) => d.name,
                None => {
                    return HttpResponse::NotFound().json(serde_json::json!({
                        "error": format!("Domain {} not found", domain_id)
                    }));
                }
            }
        }
        Err(_) => {
            // Fallback to config_manager on error
            match state.config_manager.get_domain(&domain_id).await {
                Some(d) => d.name,
                None => {
                    return HttpResponse::NotFound().json(serde_json::json!({
                        "error": format!("Domain {} not found", domain_id)
                    }));
                }
            }
        }
    };
    
    tracing::info!("Generating DNS records for domain: {} ({})", domain_name, domain_id);
    
    match state.config_manager.generate_dns_records(&domain_id).await {
        Ok(records) => {
            let formatted: Vec<serde_json::Value> = records.into_iter().map(|r| {
                serde_json::json!({
                    "type": r.record_type,
                    "name": r.name,
                    "value": r.value,
                    "ttl": r.ttl,
                    "priority": r.priority,
                })
            }).collect();
            HttpResponse::Ok().json(formatted)
        }
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({
            "error": e.to_string()
        })),
    }
}

pub async fn get_dns_records_text(
    state: web::Data<ConfigState>,
    path: web::Path<String>,
) -> impl Responder {
    match state.config_manager.format_dns_records(&path).await {
        Ok(text) => HttpResponse::Ok().content_type("text/plain").body(text),
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({
            "error": e.to_string()
        })),
    }
}

/// Verify DNS records for a domain using actual DNS lookups
pub async fn verify_dns_records(
    state: web::Data<ConfigState>,
    path: web::Path<String>,
) -> impl Responder {
    use hickory_resolver::TokioAsyncResolver;
    use hickory_resolver::config::{ResolverConfig, ResolverOpts};
    
    let domain_id = path.into_inner();
    
    // Check if domain exists - try storage (Supabase first) then config_manager (JSON fallback)
    let domain_name = match state.storage.get_domain(&domain_id).await {
        Ok(Some(d)) => d.name,
        Ok(None) => {
            // Fallback to config_manager
            match state.config_manager.get_domain(&domain_id).await {
                Some(d) => d.name,
                None => {
                    return HttpResponse::NotFound().json(serde_json::json!({
                        "error": format!("Domain {} not found", domain_id)
                    }));
                }
            }
        }
        Err(_) => {
            // Fallback to config_manager on error
            match state.config_manager.get_domain(&domain_id).await {
                Some(d) => d.name,
                None => {
                    return HttpResponse::NotFound().json(serde_json::json!({
                        "error": format!("Domain {} not found", domain_id)
                    }));
                }
            }
        }
    };
    tracing::info!("DNS verification requested for domain: {}", domain_name);
    
    // Create DNS resolver
    let resolver = TokioAsyncResolver::tokio(ResolverConfig::default(), ResolverOpts::default());
    
    // Get expected DNS records
    let expected_records = match state.config_manager.generate_dns_records(&domain_id).await {
        Ok(r) => r,
        Err(e) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": e.to_string()
            }));
        }
    };
    
    // Find expected A record value (server IP)
    let expected_a = expected_records.iter()
        .find(|r| r.record_type == "A")
        .map(|r| r.value.clone())
        .unwrap_or_else(|| "YOUR_SERVER_IP".to_string());
    
    // Verify A record for mail subdomain
    let mail_hostname = format!("mail.{}", domain_name);
    let a_verified = match resolver.ipv4_lookup(mail_hostname.as_str()).await {
        Ok(a_response) => {
            let found = a_response.iter().any(|ip| {
                let ip_str = ip.to_string();
                tracing::debug!("A record found for {}: {} (expected: {})", mail_hostname, ip_str, expected_a);
                // If expected is placeholder, any IP is valid
                expected_a == "YOUR_SERVER_IP" || ip_str == expected_a
            });
            if found {
                tracing::info!("A record verified for {}", mail_hostname);
            } else {
                tracing::warn!("A record mismatch for {} (expected: {})", mail_hostname, expected_a);
            }
            found
        }
        Err(e) => {
            tracing::warn!("A lookup failed for {}: {}", mail_hostname, e);
            false
        }
    };
    
    // Find expected MX value
    let expected_mx = expected_records.iter()
        .find(|r| r.record_type == "MX")
        .map(|r| r.value.clone())
        .unwrap_or_else(|| format!("mail.{}", domain_name));
    
    // Verify MX record
    let mx_verified = match resolver.mx_lookup(domain_name.as_str()).await {
        Ok(mx_response) => {
            let found = mx_response.iter().any(|mx| {
                let mx_host = mx.exchange().to_string().trim_end_matches('.').to_lowercase();
                let expected = expected_mx.to_lowercase();
                tracing::debug!("MX record found: {} (expected: {})", mx_host, expected);
                mx_host == expected || mx_host.ends_with(&format!(".{}", expected))
            });
            if found {
                tracing::info!("MX record verified for {}", domain_name);
            } else {
                tracing::warn!("MX record mismatch for {}", domain_name);
            }
            found
        }
        Err(e) => {
            tracing::warn!("MX lookup failed for {}: {}", domain_name, e);
            false
        }
    };
    
    // Verify SPF record (TXT record containing v=spf1)
    let spf_verified = match resolver.txt_lookup(domain_name.as_str()).await {
        Ok(txt_response) => {
            let found = txt_response.iter().any(|txt| {
                let record = txt.to_string();
                let is_spf = record.contains("v=spf1");
                if is_spf {
                    tracing::debug!("SPF record found: {}", record);
                }
                is_spf
            });
            if found {
                tracing::info!("SPF record verified for {}", domain_name);
            }
            found
        }
        Err(e) => {
            tracing::warn!("TXT lookup failed for {}: {}", domain_name, e);
            false
        }
    };
    
    // Verify DKIM record (TXT record at default._domainkey.domain)
    let dkim_domain = format!("default._domainkey.{}", domain_name);
    let dkim_verified = match resolver.txt_lookup(dkim_domain.as_str()).await {
        Ok(txt_response) => {
            let found = txt_response.iter().any(|txt| {
                let record = txt.to_string();
                let is_dkim = record.contains("v=DKIM1");
                if is_dkim {
                    tracing::debug!("DKIM record found: {}", record);
                }
                is_dkim
            });
            if found {
                tracing::info!("DKIM record verified for {}", domain_name);
            }
            found
        }
        Err(_) => false
    };
    
    // Verify DMARC record (TXT record at _dmarc.domain)
    let dmarc_domain = format!("_dmarc.{}", domain_name);
    let dmarc_verified = match resolver.txt_lookup(dmarc_domain.as_str()).await {
        Ok(txt_response) => {
            let found = txt_response.iter().any(|txt| {
                let record = txt.to_string();
                let is_dmarc = record.contains("v=DMARC1");
                if is_dmarc {
                    tracing::debug!("DMARC record found: {}", record);
                }
                is_dmarc
            });
            if found {
                tracing::info!("DMARC record verified for {}", domain_name);
            }
            found
        }
        Err(_) => false
    };
    
    // Save verification results to storage (Supabase first, JSON fallback)
    if let Err(e) = state.storage.update_dns_verification(
        &domain_id,
        mx_verified,
        spf_verified,
        dkim_verified,
        dmarc_verified,
    ).await {
        tracing::warn!("Failed to save DNS verification status to storage: {}", e);
        // Fallback to config_manager (JSON)
        if let Err(e2) = state.config_manager.update_dns_verification(
            &domain_id,
            mx_verified,
            spf_verified,
            dkim_verified,
            dmarc_verified,
        ).await {
            tracing::warn!("Failed to save DNS verification status to JSON: {}", e2);
        }
    }
    
    // Build message based on results
    let message = if a_verified && mx_verified && spf_verified {
        if dkim_verified && dmarc_verified {
            "All DNS records verified successfully!".to_string()
        } else {
            "Required records (A, MX, SPF) verified. Optional records (DKIM, DMARC) pending.".to_string()
        }
    } else {
        let mut missing = Vec::new();
        if !a_verified { missing.push("A (mail subdomain)"); }
        if !mx_verified { missing.push("MX"); }
        if !spf_verified { missing.push("SPF"); }
        format!("Missing required records: {}. Please add them to your DNS provider.", missing.join(", "))
    };
    
    HttpResponse::Ok().json(serde_json::json!({
        "a_verified": a_verified,
        "mx_verified": mx_verified,
        "spf_verified": spf_verified,
        "dkim_verified": dkim_verified,
        "dmarc_verified": dmarc_verified,
        "message": message
    }))
}

// =============================================================================
// Dashboard Stats (from config)
// =============================================================================

pub async fn get_config_stats(state: web::Data<ConfigState>) -> impl Responder {
    let domains = state.config_manager.get_domains().await;
    let mailboxes = state.config_manager.get_all_mailboxes().await;
    
    let active_mailboxes = mailboxes.iter().filter(|m| m.enabled).count();

    HttpResponse::Ok().json(serde_json::json!({
        "total_domains": domains.len(),
        "total_mailboxes": mailboxes.len(),
        "active_mailboxes": active_mailboxes,
    }))
}

// =============================================================================
// Workspace Rules - General rules for all AI agents
// =============================================================================

const WORKSPACE_RULES_PATH: &str = "data/config/workspace_rules.md";

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkspaceRulesResponse {
    pub content: String,
    pub last_modified: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateWorkspaceRulesRequest {
    pub content: String,
}

/// Get workspace rules
pub async fn get_workspace_rules() -> impl Responder {
    use std::fs;
    use std::path::Path;
    
    let path = Path::new(WORKSPACE_RULES_PATH);
    
    // Create default rules if file doesn't exist
    if !path.exists() {
        let default_rules = r#"# Amail Workspace Rules

These rules apply to ALL AI agents in the Amail system. Each agent will follow these guidelines in addition to their role-specific training.

## Communication Style
- Be professional and concise
- Use clear, actionable language
- Avoid jargon unless appropriate for the context

## Response Guidelines
- Always acknowledge receipt of emails
- Provide estimated response times when applicable
- Escalate urgent matters immediately

## Privacy & Security
- Never share sensitive information externally
- Verify sender identity for sensitive requests
- Log all significant actions for audit purposes

## Collaboration
- Coordinate with other agents when tasks span multiple domains
- Share relevant context when handing off tasks
- Maintain continuity in ongoing conversations

---
*Last updated: Never (default rules)*
"#;
        
        // Ensure directory exists
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = fs::write(path, default_rules);
    }
    
    match fs::read_to_string(path) {
        Ok(content) => {
            let last_modified = fs::metadata(path)
                .ok()
                .and_then(|m| m.modified().ok())
                .map(|t| {
                    let datetime: chrono::DateTime<chrono::Utc> = t.into();
                    datetime.to_rfc3339()
                });
            
            HttpResponse::Ok().json(WorkspaceRulesResponse {
                content,
                last_modified,
            })
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to read workspace rules: {}", e)
            }))
        }
    }
}

/// Update workspace rules
pub async fn update_workspace_rules(
    body: web::Json<UpdateWorkspaceRulesRequest>,
) -> impl Responder {
    use std::fs;
    use std::path::Path;
    
    let path = Path::new(WORKSPACE_RULES_PATH);
    
    // Ensure directory exists
    if let Some(parent) = path.parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to create directory: {}", e)
            }));
        }
    }
    
    match fs::write(path, &body.content) {
        Ok(_) => {
            tracing::info!("Workspace rules updated ({} bytes)", body.content.len());
            HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "message": "Workspace rules updated successfully"
            }))
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to write workspace rules: {}", e)
            }))
        }
    }
}

// =============================================================================
// Route Configuration
// =============================================================================

pub fn configure_config_routes(cfg: &mut web::ServiceConfig) {
    // Use /api/config to avoid conflict with /api/v1 from other routes
    cfg.service(
        web::scope("/api/config")
            // Domains (JSON-based)
            .route("/domains", web::get().to(list_domains))
            .route("/domains", web::post().to(create_domain))
            .route("/domains/{id}", web::get().to(get_domain))
            .route("/domains/{id}", web::delete().to(delete_domain))
            .route("/domains/{id}/mailboxes", web::get().to(list_mailboxes_by_domain))
            .route("/domains/{id}/dns", web::get().to(get_dns_records))
            .route("/domains/{id}/dns/text", web::get().to(get_dns_records_text))
            .route("/domains/{id}/dns/verify", web::post().to(verify_dns_records))
            
            // Mailboxes (JSON-based)
            .route("/mailboxes", web::get().to(list_mailboxes))
            .route("/mailboxes", web::post().to(create_mailbox))
            .route("/mailboxes/{id}", web::delete().to(delete_mailbox))
            
            // Stats
            .route("/stats", web::get().to(get_config_stats))
            
            // Workspace Rules
            .route("/workspace-rules", web::get().to(get_workspace_rules))
            .route("/workspace-rules", web::put().to(update_workspace_rules))
    );
}
