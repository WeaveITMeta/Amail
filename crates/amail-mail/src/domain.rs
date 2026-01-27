//! Domain management for multi-tenant email
//!
//! Handles domain registration, DNS verification, and tenant isolation.

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, FromRow};
use uuid::Uuid;
use tracing::{info, debug};

// =============================================================================
// Data Types
// =============================================================================

/// Tenant (organization/customer)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Tenant {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub is_active: bool,
    pub plan: String,
    pub created_at: DateTime<Utc>,
}

/// Domain associated with a tenant
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Domain {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub domain: String,
    pub is_primary: bool,
    pub is_verified: bool,
    pub verification_token: String,
    pub mx_verified: bool,
    pub spf_verified: bool,
    pub dkim_verified: bool,
    pub dmarc_verified: bool,
    pub created_at: DateTime<Utc>,
    pub verified_at: Option<DateTime<Utc>>,
}

/// DNS record for domain verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsRecord {
    pub record_type: String, // MX, TXT, CNAME
    pub name: String,
    pub value: String,
    pub priority: Option<u16>,
    pub purpose: String, // verification, mx, spf, dkim, dmarc
}

/// Domain verification status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationStatus {
    pub domain: String,
    pub mx_verified: bool,
    pub spf_verified: bool,
    pub dkim_verified: bool,
    pub dmarc_verified: bool,
    pub all_verified: bool,
    pub required_records: Vec<DnsRecord>,
    pub missing_records: Vec<DnsRecord>,
}

// =============================================================================
// Domain Manager
// =============================================================================

/// Manages domains and tenants
pub struct DomainManager {
    pool: PgPool,
    mail_hostname: String,
    dkim_selector: String,
}

impl DomainManager {
    /// Create a new domain manager
    pub fn new(pool: PgPool, mail_hostname: &str, dkim_selector: &str) -> Self {
        Self {
            pool,
            mail_hostname: mail_hostname.to_string(),
            dkim_selector: dkim_selector.to_string(),
        }
    }

    /// Initialize database schema
    pub async fn init_schema(&self) -> Result<()> {
        sqlx::query(DOMAIN_SCHEMA)
            .execute(&self.pool)
            .await?;
        info!("Domain management schema initialized");
        Ok(())
    }

    // =========================================================================
    // Tenant Operations
    // =========================================================================

    /// Create a new tenant
    pub async fn create_tenant(&self, name: &str, slug: &str, plan: &str) -> Result<Tenant> {
        let id = Uuid::new_v4();
        
        let tenant = sqlx::query_as::<_, Tenant>(
            r#"
            INSERT INTO mail_tenants (id, name, slug, plan)
            VALUES ($1, $2, $3, $4)
            RETURNING *
            "#
        )
        .bind(id)
        .bind(name)
        .bind(slug)
        .bind(plan)
        .fetch_one(&self.pool)
        .await?;

        info!("Created tenant: {} ({})", name, id);
        Ok(tenant)
    }

    /// Get tenant by ID
    pub async fn get_tenant(&self, id: Uuid) -> Result<Option<Tenant>> {
        let tenant = sqlx::query_as::<_, Tenant>(
            "SELECT * FROM mail_tenants WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(tenant)
    }

    /// Get tenant by slug
    pub async fn get_tenant_by_slug(&self, slug: &str) -> Result<Option<Tenant>> {
        let tenant = sqlx::query_as::<_, Tenant>(
            "SELECT * FROM mail_tenants WHERE slug = $1"
        )
        .bind(slug)
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(tenant)
    }

    /// List all tenants
    pub async fn list_tenants(&self) -> Result<Vec<Tenant>> {
        let tenants = sqlx::query_as::<_, Tenant>(
            "SELECT * FROM mail_tenants ORDER BY name"
        )
        .fetch_all(&self.pool)
        .await?;
        
        Ok(tenants)
    }

    // =========================================================================
    // Domain Operations
    // =========================================================================

    /// Add a domain to a tenant
    pub async fn add_domain(&self, tenant_id: Uuid, domain: &str, is_primary: bool) -> Result<Domain> {
        let id = Uuid::new_v4();
        let verification_token = format!("amail-verify-{}", Uuid::new_v4().to_string().replace("-", "")[..16].to_string());
        
        let domain_record = sqlx::query_as::<_, Domain>(
            r#"
            INSERT INTO mail_domains (id, tenant_id, domain, is_primary, verification_token)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
            "#
        )
        .bind(id)
        .bind(tenant_id)
        .bind(domain)
        .bind(is_primary)
        .bind(&verification_token)
        .fetch_one(&self.pool)
        .await?;

        info!("Added domain {} for tenant {}", domain, tenant_id);
        Ok(domain_record)
    }

    /// Get domain by name
    pub async fn get_domain(&self, domain: &str) -> Result<Option<Domain>> {
        let domain_record = sqlx::query_as::<_, Domain>(
            "SELECT * FROM mail_domains WHERE domain = $1"
        )
        .bind(domain)
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(domain_record)
    }

    /// List domains for a tenant
    pub async fn list_domains(&self, tenant_id: Uuid) -> Result<Vec<Domain>> {
        let domains = sqlx::query_as::<_, Domain>(
            "SELECT * FROM mail_domains WHERE tenant_id = $1 ORDER BY is_primary DESC, domain"
        )
        .bind(tenant_id)
        .fetch_all(&self.pool)
        .await?;
        
        Ok(domains)
    }

    /// Get required DNS records for a domain
    pub fn get_required_dns_records(&self, domain: &str, verification_token: &str) -> Vec<DnsRecord> {
        vec![
            // MX record - route mail to Amail
            DnsRecord {
                record_type: "MX".to_string(),
                name: domain.to_string(),
                value: format!("{}.", self.mail_hostname),
                priority: Some(10),
                purpose: "mx".to_string(),
            },
            // TXT record for domain verification
            DnsRecord {
                record_type: "TXT".to_string(),
                name: format!("_amail.{}", domain),
                value: verification_token.to_string(),
                priority: None,
                purpose: "verification".to_string(),
            },
            // SPF record
            DnsRecord {
                record_type: "TXT".to_string(),
                name: domain.to_string(),
                value: format!("v=spf1 include:{} ~all", self.mail_hostname),
                priority: None,
                purpose: "spf".to_string(),
            },
            // DKIM record
            DnsRecord {
                record_type: "TXT".to_string(),
                name: format!("{}._domainkey.{}", self.dkim_selector, domain),
                value: "v=DKIM1; k=rsa; p=<YOUR_DKIM_PUBLIC_KEY>".to_string(),
                priority: None,
                purpose: "dkim".to_string(),
            },
            // DMARC record
            DnsRecord {
                record_type: "TXT".to_string(),
                name: format!("_dmarc.{}", domain),
                value: format!("v=DMARC1; p=quarantine; rua=mailto:dmarc@{}", self.mail_hostname),
                priority: None,
                purpose: "dmarc".to_string(),
            },
        ]
    }

    /// Verify domain DNS records
    pub async fn verify_domain(&self, domain: &str) -> Result<VerificationStatus> {
        use hickory_resolver::TokioAsyncResolver;
        use hickory_resolver::config::*;

        let domain_record = self.get_domain(domain).await?
            .ok_or_else(|| anyhow::anyhow!("Domain not found: {}", domain))?;

        let resolver = TokioAsyncResolver::tokio(
            ResolverConfig::default(),
            ResolverOpts::default(),
        );

        let required_records = self.get_required_dns_records(domain, &domain_record.verification_token);
        let mut missing_records = Vec::new();

        // Check MX records
        let mx_verified = match resolver.mx_lookup(domain).await {
            Ok(mx) => mx.iter().any(|r| r.exchange().to_string().contains(&self.mail_hostname)),
            Err(_) => false,
        };

        // Check TXT records for verification token
        let verification_name = format!("_amail.{}", domain);
        let txt_verified = match resolver.txt_lookup(&verification_name).await {
            Ok(txt) => txt.iter().any(|r| {
                r.iter().any(|data| {
                    String::from_utf8_lossy(data).contains(&domain_record.verification_token)
                })
            }),
            Err(_) => false,
        };

        // Check SPF
        let spf_verified = match resolver.txt_lookup(domain).await {
            Ok(txt) => txt.iter().any(|r| {
                r.iter().any(|data| {
                    let s = String::from_utf8_lossy(data);
                    s.starts_with("v=spf1") && s.contains(&self.mail_hostname)
                })
            }),
            Err(_) => false,
        };

        // Check DKIM
        let dkim_name = format!("{}._domainkey.{}", self.dkim_selector, domain);
        let dkim_verified = match resolver.txt_lookup(&dkim_name).await {
            Ok(txt) => txt.iter().any(|r| {
                r.iter().any(|data| String::from_utf8_lossy(data).starts_with("v=DKIM1"))
            }),
            Err(_) => false,
        };

        // Check DMARC
        let dmarc_name = format!("_dmarc.{}", domain);
        let dmarc_verified = match resolver.txt_lookup(&dmarc_name).await {
            Ok(txt) => txt.iter().any(|r| {
                r.iter().any(|data| String::from_utf8_lossy(data).starts_with("v=DMARC1"))
            }),
            Err(_) => false,
        };

        // Build missing records list
        if !mx_verified {
            missing_records.push(required_records.iter().find(|r| r.purpose == "mx").unwrap().clone());
        }
        if !txt_verified {
            missing_records.push(required_records.iter().find(|r| r.purpose == "verification").unwrap().clone());
        }
        if !spf_verified {
            missing_records.push(required_records.iter().find(|r| r.purpose == "spf").unwrap().clone());
        }
        if !dkim_verified {
            missing_records.push(required_records.iter().find(|r| r.purpose == "dkim").unwrap().clone());
        }
        if !dmarc_verified {
            missing_records.push(required_records.iter().find(|r| r.purpose == "dmarc").unwrap().clone());
        }

        let all_verified = mx_verified && txt_verified && spf_verified && dkim_verified && dmarc_verified;

        // Update database
        sqlx::query(
            r#"
            UPDATE mail_domains SET
                is_verified = $2,
                mx_verified = $3,
                spf_verified = $4,
                dkim_verified = $5,
                dmarc_verified = $6,
                verified_at = CASE WHEN $2 THEN NOW() ELSE verified_at END
            WHERE domain = $1
            "#
        )
        .bind(domain)
        .bind(txt_verified) // is_verified based on TXT token
        .bind(mx_verified)
        .bind(spf_verified)
        .bind(dkim_verified)
        .bind(dmarc_verified)
        .execute(&self.pool)
        .await?;

        debug!("Domain {} verification: MX={}, SPF={}, DKIM={}, DMARC={}", 
               domain, mx_verified, spf_verified, dkim_verified, dmarc_verified);

        Ok(VerificationStatus {
            domain: domain.to_string(),
            mx_verified,
            spf_verified,
            dkim_verified,
            dmarc_verified,
            all_verified,
            required_records,
            missing_records,
        })
    }

    /// Find tenant by domain
    pub async fn find_tenant_by_domain(&self, domain: &str) -> Result<Option<Uuid>> {
        let result = sqlx::query_scalar::<_, Uuid>(
            "SELECT tenant_id FROM mail_domains WHERE domain = $1 AND is_verified = true"
        )
        .bind(domain)
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(result)
    }
}

// =============================================================================
// Database Schema
// =============================================================================

const DOMAIN_SCHEMA: &str = r#"
-- Tenants table
CREATE TABLE IF NOT EXISTS mail_tenants (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    slug TEXT NOT NULL UNIQUE,
    is_active BOOLEAN NOT NULL DEFAULT true,
    plan TEXT NOT NULL DEFAULT 'free',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_mail_tenants_slug ON mail_tenants(slug);

-- Domains table
CREATE TABLE IF NOT EXISTS mail_domains (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL REFERENCES mail_tenants(id),
    domain TEXT NOT NULL UNIQUE,
    is_primary BOOLEAN NOT NULL DEFAULT false,
    is_verified BOOLEAN NOT NULL DEFAULT false,
    verification_token TEXT NOT NULL,
    mx_verified BOOLEAN NOT NULL DEFAULT false,
    spf_verified BOOLEAN NOT NULL DEFAULT false,
    dkim_verified BOOLEAN NOT NULL DEFAULT false,
    dmarc_verified BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    verified_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_mail_domains_tenant ON mail_domains(tenant_id);
CREATE INDEX IF NOT EXISTS idx_mail_domains_domain ON mail_domains(domain);
"#;
