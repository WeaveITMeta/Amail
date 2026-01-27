// =============================================================================
// Unified Storage Layer - Supabase Primary, JSON Fallback
// =============================================================================
// Table of Contents:
// 1. StorageBackend trait - Common interface for storage
// 2. SupabaseStorage - Database-backed storage (primary)
// 3. JsonStorage - File-based storage (fallback)
// 4. UnifiedStorage - Automatic failover between backends
// =============================================================================

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use tracing::{info, warn};

use amail_core::config_manager::{ConfigManager, DomainConfig, MailboxConfig};

// =============================================================================
// 1. Domain and Mailbox Types
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredDomain {
    pub id: String,
    pub name: String,
    pub display_name: String,
    pub enabled: bool,
    pub primary: bool,
    pub mx_verified: bool,
    pub spf_verified: bool,
    pub dkim_verified: bool,
    pub dmarc_verified: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredMailbox {
    pub id: String,
    pub domain_id: String,
    pub email: String,
    pub display_name: String,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

// =============================================================================
// 2. Storage Backend Trait
// =============================================================================

#[async_trait]
pub trait StorageBackend: Send + Sync {
    async fn list_domains(&self) -> Result<Vec<StoredDomain>, String>;
    async fn get_domain(&self, id: &str) -> Result<Option<StoredDomain>, String>;
    async fn create_domain(&self, name: &str) -> Result<StoredDomain, String>;
    async fn delete_domain(&self, id: &str) -> Result<(), String>;
    async fn update_dns_verification(&self, id: &str, mx: bool, spf: bool, dkim: bool, dmarc: bool) -> Result<(), String>;
    
    async fn list_mailboxes(&self) -> Result<Vec<StoredMailbox>, String>;
    async fn list_mailboxes_by_domain(&self, domain_id: &str) -> Result<Vec<StoredMailbox>, String>;
    async fn create_mailbox(&self, domain_id: &str, email: &str, display_name: &str) -> Result<StoredMailbox, String>;
    async fn delete_mailbox(&self, id: &str) -> Result<(), String>;
    
    fn backend_name(&self) -> &'static str;
}

// =============================================================================
// 3. Supabase/PostgreSQL Storage
// =============================================================================

pub struct SupabaseStorage {
    pool: PgPool,
}

impl SupabaseStorage {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl StorageBackend for SupabaseStorage {
    async fn list_domains(&self) -> Result<Vec<StoredDomain>, String> {
        let rows: Vec<(String, String, String, bool, bool, bool, bool, bool, bool, DateTime<Utc>)> = 
            sqlx::query_as(
                r#"SELECT id, name, display_name, enabled, "primary", 
                   mx_verified, spf_verified, dkim_verified, dmarc_verified, created_at
                   FROM domains ORDER BY created_at DESC"#
            )
            .fetch_all(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        
        Ok(rows.into_iter().map(|r| StoredDomain {
            id: r.0,
            name: r.1,
            display_name: r.2,
            enabled: r.3,
            primary: r.4,
            mx_verified: r.5,
            spf_verified: r.6,
            dkim_verified: r.7,
            dmarc_verified: r.8,
            created_at: r.9,
        }).collect())
    }
    
    async fn get_domain(&self, id: &str) -> Result<Option<StoredDomain>, String> {
        let row: Option<(String, String, String, bool, bool, bool, bool, bool, bool, DateTime<Utc>)> = 
            sqlx::query_as(
                r#"SELECT id, name, display_name, enabled, "primary",
                   mx_verified, spf_verified, dkim_verified, dmarc_verified, created_at
                   FROM domains WHERE LOWER(id) = LOWER($1)"#
            )
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        
        Ok(row.map(|r| StoredDomain {
            id: r.0,
            name: r.1,
            display_name: r.2,
            enabled: r.3,
            primary: r.4,
            mx_verified: r.5,
            spf_verified: r.6,
            dkim_verified: r.7,
            dmarc_verified: r.8,
            created_at: r.9,
        }))
    }
    
    async fn create_domain(&self, name: &str) -> Result<StoredDomain, String> {
        // Store ID in lowercase for case-insensitive lookups
        let id = name.to_lowercase().replace('.', "-");
        // Store name in lowercase, display_name preserves original casing
        let name_lower = name.to_lowercase();
        let now = Utc::now();
        
        sqlx::query(
            r#"INSERT INTO domains (id, name, display_name, enabled, "primary", 
                                mx_verified, spf_verified, dkim_verified, dmarc_verified, created_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)"#
        )
        .bind(&id)
        .bind(&name_lower)
        .bind(name)
        .bind(true)
        .bind(false)
        .bind(false)
        .bind(false)
        .bind(false)
        .bind(false)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;
        
        Ok(StoredDomain {
            id,
            name: name_lower,
            display_name: name.to_string(),
            enabled: true,
            primary: false,
            mx_verified: false,
            spf_verified: false,
            dkim_verified: false,
            dmarc_verified: false,
            created_at: now,
        })
    }
    
    async fn delete_domain(&self, id: &str) -> Result<(), String> {
        sqlx::query("DELETE FROM domains WHERE LOWER(id) = LOWER($1)")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }
    
    async fn update_dns_verification(&self, id: &str, mx: bool, spf: bool, dkim: bool, dmarc: bool) -> Result<(), String> {
        sqlx::query(
            "UPDATE domains SET mx_verified = $1, spf_verified = $2, dkim_verified = $3, dmarc_verified = $4 WHERE LOWER(id) = LOWER($5)"
        )
        .bind(mx)
        .bind(spf)
        .bind(dkim)
        .bind(dmarc)
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;
        Ok(())
    }
    
    async fn list_mailboxes(&self) -> Result<Vec<StoredMailbox>, String> {
        let rows: Vec<(String, String, String, String, bool, DateTime<Utc>)> = 
            sqlx::query_as(
                "SELECT id, domain_id, email, display_name, enabled, created_at FROM mailboxes ORDER BY created_at DESC"
            )
            .fetch_all(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        
        Ok(rows.into_iter().map(|r| StoredMailbox {
            id: r.0,
            domain_id: r.1,
            email: r.2,
            display_name: r.3,
            enabled: r.4,
            created_at: r.5,
        }).collect())
    }
    
    async fn list_mailboxes_by_domain(&self, domain_id: &str) -> Result<Vec<StoredMailbox>, String> {
        let rows: Vec<(String, String, String, String, bool, DateTime<Utc>)> = 
            sqlx::query_as(
                "SELECT id, domain_id, email, display_name, enabled, created_at FROM mailboxes WHERE LOWER(domain_id) = LOWER($1)"
            )
            .bind(domain_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        
        Ok(rows.into_iter().map(|r| StoredMailbox {
            id: r.0,
            domain_id: r.1,
            email: r.2,
            display_name: r.3,
            enabled: r.4,
            created_at: r.5,
        }).collect())
    }
    
    async fn create_mailbox(&self, domain_id: &str, email: &str, display_name: &str) -> Result<StoredMailbox, String> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();
        
        sqlx::query(
            "INSERT INTO mailboxes (id, domain_id, email, display_name, enabled, created_at) VALUES ($1, $2, $3, $4, $5, $6)"
        )
        .bind(&id)
        .bind(domain_id)
        .bind(email)
        .bind(display_name)
        .bind(true)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;
        
        Ok(StoredMailbox {
            id,
            domain_id: domain_id.to_string(),
            email: email.to_string(),
            display_name: display_name.to_string(),
            enabled: true,
            created_at: now,
        })
    }
    
    async fn delete_mailbox(&self, id: &str) -> Result<(), String> {
        sqlx::query("DELETE FROM mailboxes WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }
    
    fn backend_name(&self) -> &'static str {
        "Supabase"
    }
}

// =============================================================================
// 4. JSON File Storage (Fallback)
// =============================================================================

pub struct JsonStorage {
    config_manager: Arc<ConfigManager>,
}

impl JsonStorage {
    pub fn new(config_manager: Arc<ConfigManager>) -> Self {
        Self { config_manager }
    }
}

#[async_trait]
impl StorageBackend for JsonStorage {
    async fn list_domains(&self) -> Result<Vec<StoredDomain>, String> {
        // Reload from disk to ensure we have latest data
        if let Err(e) = self.config_manager.load_domains().await {
            warn!("Failed to reload domains: {}", e);
        }
        let domains = self.config_manager.get_domains().await;
        info!("JsonStorage: Found {} domains", domains.len());
        Ok(domains.into_iter().map(|d| StoredDomain {
            id: d.id,
            name: d.name,
            display_name: d.display_name,
            enabled: d.enabled,
            primary: d.primary,
            mx_verified: d.mx_verified,
            spf_verified: d.spf_verified,
            dkim_verified: d.dkim_verified,
            dmarc_verified: d.dmarc_verified,
            created_at: d.created_at,
        }).collect())
    }
    
    async fn get_domain(&self, id: &str) -> Result<Option<StoredDomain>, String> {
        let domain = self.config_manager.get_domain(id).await;
        Ok(domain.map(|d| StoredDomain {
            id: d.id,
            name: d.name,
            display_name: d.display_name,
            enabled: d.enabled,
            primary: d.primary,
            mx_verified: d.mx_verified,
            spf_verified: d.spf_verified,
            dkim_verified: d.dkim_verified,
            dmarc_verified: d.dmarc_verified,
            created_at: d.created_at,
        }))
    }
    
    async fn create_domain(&self, name: &str) -> Result<StoredDomain, String> {
        // Store ID and name in lowercase, display_name preserves original casing
        let id = name.to_lowercase().replace('.', "-");
        let name_lower = name.to_lowercase();
        
        let domain_config = DomainConfig {
            id,
            name: name_lower.clone(),
            display_name: name.to_string(),
            enabled: true,
            primary: false,
            settings: Default::default(),
            dkim: Default::default(),
            branding: Default::default(),
            created_at: Utc::now(),
            mx_verified: false,
            spf_verified: false,
            dkim_verified: false,
            dmarc_verified: false,
            last_verified: None,
        };
        
        let domain = self.config_manager.add_domain(domain_config).await
            .map_err(|e| e.to_string())?;
        
        Ok(StoredDomain {
            id: domain.id,
            name: domain.name,
            display_name: domain.display_name,
            enabled: domain.enabled,
            primary: domain.primary,
            mx_verified: domain.mx_verified,
            spf_verified: domain.spf_verified,
            dkim_verified: domain.dkim_verified,
            dmarc_verified: domain.dmarc_verified,
            created_at: domain.created_at,
        })
    }
    
    async fn delete_domain(&self, id: &str) -> Result<(), String> {
        self.config_manager.delete_domain(id).await
            .map_err(|e| e.to_string())
    }
    
    async fn list_mailboxes(&self) -> Result<Vec<StoredMailbox>, String> {
        let mailboxes = self.config_manager.get_all_mailboxes().await;
        Ok(mailboxes.into_iter().map(|m| StoredMailbox {
            id: m.id,
            domain_id: m.domain_id,
            email: m.email,
            display_name: m.display_name,
            enabled: m.enabled,
            created_at: m.created_at,
        }).collect())
    }
    
    async fn list_mailboxes_by_domain(&self, domain_id: &str) -> Result<Vec<StoredMailbox>, String> {
        let mailboxes = self.config_manager.get_mailboxes(domain_id).await;
        Ok(mailboxes.into_iter().map(|m| StoredMailbox {
            id: m.id,
            domain_id: m.domain_id,
            email: m.email,
            display_name: m.display_name,
            enabled: m.enabled,
            created_at: m.created_at,
        }).collect())
    }
    
    async fn create_mailbox(&self, domain_id: &str, email: &str, display_name: &str) -> Result<StoredMailbox, String> {
        // Create a MailboxConfig
        let mailbox_config = MailboxConfig {
            id: String::new(), // Will be generated
            domain_id: domain_id.to_string(),
            email: email.to_string(),
            display_name: display_name.to_string(),
            enabled: true,
            quota_mb: 1024,
            settings: Default::default(),
            aliases: vec![],
            ai_agent: Default::default(),
            created_at: Utc::now(),
            r#type: "user".to_string(),
            password_hash: None,
        };
        
        let mailbox = self.config_manager.add_mailbox(mailbox_config).await
            .map_err(|e| e.to_string())?;
        
        Ok(StoredMailbox {
            id: mailbox.id,
            domain_id: mailbox.domain_id,
            email: mailbox.email,
            display_name: mailbox.display_name,
            enabled: mailbox.enabled,
            created_at: mailbox.created_at,
        })
    }
    
    async fn delete_mailbox(&self, id: &str) -> Result<(), String> {
        self.config_manager.delete_mailbox(id).await
            .map_err(|e| e.to_string())
    }
    
    async fn update_dns_verification(&self, id: &str, mx: bool, spf: bool, dkim: bool, dmarc: bool) -> Result<(), String> {
        self.config_manager.update_dns_verification(id, mx, spf, dkim, dmarc).await
            .map_err(|e| e.to_string())?;
        Ok(())
    }
    
    fn backend_name(&self) -> &'static str {
        "JSON"
    }
}

// =============================================================================
// 5. Unified Storage - Supabase Primary, JSON Fallback
// =============================================================================

pub struct UnifiedStorage {
    primary: Option<Arc<dyn StorageBackend>>,
    fallback: Arc<dyn StorageBackend>,
}

impl UnifiedStorage {
    pub fn new(db_pool: Option<PgPool>, config_manager: Arc<ConfigManager>) -> Self {
        let fallback = Arc::new(JsonStorage::new(config_manager));
        
        let primary = db_pool.map(|pool| {
            Arc::new(SupabaseStorage::new(pool)) as Arc<dyn StorageBackend>
        });
        
        if primary.is_some() {
            info!("UnifiedStorage: Using Supabase as primary, JSON as fallback");
        } else {
            info!("UnifiedStorage: Using JSON storage (no database configured)");
        }
        
        Self { primary, fallback }
    }
    
    fn get_backend(&self) -> &Arc<dyn StorageBackend> {
        self.primary.as_ref().unwrap_or(&self.fallback)
    }
    
    pub fn backend_name(&self) -> &'static str {
        self.get_backend().backend_name()
    }
    
    pub async fn list_domains(&self) -> Result<Vec<StoredDomain>, String> {
        self.get_backend().list_domains().await
    }
    
    pub async fn get_domain(&self, id: &str) -> Result<Option<StoredDomain>, String> {
        self.get_backend().get_domain(id).await
    }
    
    pub async fn create_domain(&self, name: &str) -> Result<StoredDomain, String> {
        self.get_backend().create_domain(name).await
    }
    
    pub async fn delete_domain(&self, id: &str) -> Result<(), String> {
        self.get_backend().delete_domain(id).await
    }
    
    pub async fn list_mailboxes(&self) -> Result<Vec<StoredMailbox>, String> {
        self.get_backend().list_mailboxes().await
    }
    
    pub async fn list_mailboxes_by_domain(&self, domain_id: &str) -> Result<Vec<StoredMailbox>, String> {
        self.get_backend().list_mailboxes_by_domain(domain_id).await
    }
    
    pub async fn create_mailbox(&self, domain_id: &str, email: &str, display_name: &str) -> Result<StoredMailbox, String> {
        self.get_backend().create_mailbox(domain_id, email, display_name).await
    }
    
    pub async fn delete_mailbox(&self, id: &str) -> Result<(), String> {
        self.get_backend().delete_mailbox(id).await
    }
    
    pub async fn update_dns_verification(&self, id: &str, mx: bool, spf: bool, dkim: bool, dmarc: bool) -> Result<(), String> {
        self.get_backend().update_dns_verification(id, mx, spf, dkim, dmarc).await
    }
}
