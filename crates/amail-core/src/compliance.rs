// =============================================================================
// Legal Compliance Module
// =============================================================================
// Table of Contents:
// 1. CSAM Detection & NCMEC Reporting (18 USC 2258A)
// 2. DMCA Takedown Handling (17 USC 512)
// 3. COPPA Compliance (Children's Online Privacy Protection)
// 4. GDPR Compliance (EU Data Protection)
// 5. Data Retention & Right to be Forgotten
// 6. Compliance Audit Logging
// =============================================================================

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// =============================================================================
// 1. CSAM Detection & NCMEC Reporting (18 USC 2258A)
// =============================================================================
// CRITICAL: Electronic service providers MUST report apparent CSAM to NCMEC
// Failure to report: Up to $850,000 per violation (providers <100M users)
// Must retain evidence for 1 year after report

/// CSAM (Child Sexual Abuse Material) detection and reporting
/// Required by 18 USC 2258A - REPORT Act (2024)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CsamCompliance {
    /// NCMEC CyberTipline API endpoint
    pub ncmec_api_url: String,
    /// Provider registration ID with NCMEC
    pub provider_id: Option<String>,
    /// Enable hash-based detection (PhotoDNA, etc.)
    pub hash_detection_enabled: bool,
    /// Retention period for reported content (minimum 1 year per law)
    pub evidence_retention_days: u32,
    /// Contact email for law enforcement
    pub law_enforcement_contact: String,
}

impl Default for CsamCompliance {
    fn default() -> Self {
        Self {
            ncmec_api_url: "https://report.cybertipline.org/ispws/".to_string(),
            provider_id: None,
            hash_detection_enabled: true,
            evidence_retention_days: 365, // Minimum required by law
            law_enforcement_contact: String::new(),
        }
    }
}

/// NCMEC CyberTipline report structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NcmecReport {
    pub report_id: Uuid,
    pub incident_type: CsamIncidentType,
    pub reported_at: DateTime<Utc>,
    /// Information about the uploader/sender
    pub reporter_info: ReporterInfo,
    /// Description of the incident
    pub incident_summary: String,
    /// Hash of the content (for evidence)
    pub content_hash: String,
    /// IP address if available
    pub ip_address: Option<String>,
    /// Email addresses involved
    pub email_addresses: Vec<String>,
    /// Status of the report
    pub status: ReportStatus,
    /// Evidence file paths (retained for 1 year)
    pub evidence_paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CsamIncidentType {
    /// Apparent child pornography (18 USC 2256)
    ChildPornography,
    /// Child sex trafficking (18 USC 1591)
    ChildSexTrafficking,
    /// Online enticement (18 USC 2422(b))
    OnlineEnticement,
    /// Misleading domain name (18 USC 2252B)
    MisleadingDomain,
    /// Other CSAM-related
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReporterInfo {
    pub provider_name: String,
    pub contact_name: String,
    pub contact_email: String,
    pub contact_phone: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ReportStatus {
    Pending,
    Submitted,
    Acknowledged,
    UnderInvestigation,
    Closed,
    Failed,
}

impl CsamCompliance {
    /// Check if content matches known CSAM hashes
    /// In production, integrate with PhotoDNA or similar service
    pub fn check_content_hash(&self, content_hash: &str) -> bool {
        // PLACEHOLDER: In production, check against NCMEC hash database
        // This requires registration with NCMEC and access to their hash sharing program
        // https://www.missingkids.org/theissues/csam
        false
    }

    /// Create a report for submission to NCMEC CyberTipline
    pub fn create_report(
        &self,
        incident_type: CsamIncidentType,
        summary: &str,
        content_hash: &str,
        email_addresses: Vec<String>,
        ip_address: Option<String>,
    ) -> NcmecReport {
        NcmecReport {
            report_id: Uuid::new_v4(),
            incident_type,
            reported_at: Utc::now(),
            reporter_info: ReporterInfo {
                provider_name: "Amail".to_string(),
                contact_name: String::new(),
                contact_email: self.law_enforcement_contact.clone(),
                contact_phone: None,
            },
            incident_summary: summary.to_string(),
            content_hash: content_hash.to_string(),
            ip_address,
            email_addresses,
            status: ReportStatus::Pending,
            evidence_paths: Vec::new(),
        }
    }

    /// Submit report to NCMEC CyberTipline
    /// Returns report confirmation number
    pub async fn submit_report(&self, report: &mut NcmecReport) -> Result<String, ComplianceError> {
        // IMPORTANT: This is a placeholder. Real implementation requires:
        // 1. Registration with NCMEC as an Electronic Service Provider
        // 2. API credentials from NCMEC
        // 3. Secure transmission of report data
        // 4. Evidence preservation for minimum 1 year
        
        // For now, log the report and return pending status
        tracing::error!(
            "CSAM REPORT REQUIRED - Report ID: {}, Type: {:?}",
            report.report_id,
            report.incident_type
        );
        
        report.status = ReportStatus::Pending;
        
        Err(ComplianceError::NcmecNotConfigured(
            "NCMEC API credentials not configured. Manual report required at https://report.cybertipline.org/".to_string()
        ))
    }
}

// =============================================================================
// 2. DMCA Takedown Handling (17 USC 512)
// =============================================================================
// Safe harbor requirements:
// - Designate DMCA agent with Copyright Office
// - Implement repeat infringer policy
// - Respond expeditiously to takedown notices
// - Provide counter-notice mechanism

/// DMCA compliance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DmcaCompliance {
    /// Designated DMCA agent name
    pub agent_name: String,
    /// Agent email address
    pub agent_email: String,
    /// Agent physical address
    pub agent_address: String,
    /// Copyright Office registration number (if registered)
    pub registration_number: Option<String>,
    /// URL where DMCA policy is published
    pub policy_url: String,
    /// Days to wait before restoring content after counter-notice (10-14 required)
    pub counter_notice_wait_days: u32,
    /// Number of strikes before account termination
    pub repeat_infringer_strikes: u32,
}

impl Default for DmcaCompliance {
    fn default() -> Self {
        Self {
            agent_name: String::new(),
            agent_email: String::new(),
            agent_address: String::new(),
            registration_number: None,
            policy_url: String::new(),
            counter_notice_wait_days: 14, // Maximum allowed
            repeat_infringer_strikes: 3,
        }
    }
}

/// DMCA takedown notice
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DmcaTakedownNotice {
    pub id: Uuid,
    pub received_at: DateTime<Utc>,
    /// Claimant information
    pub claimant_name: String,
    pub claimant_email: String,
    pub claimant_address: Option<String>,
    /// Copyrighted work identification
    pub copyrighted_work_description: String,
    pub copyrighted_work_url: Option<String>,
    /// Infringing material identification
    pub infringing_material_description: String,
    pub infringing_material_location: String,
    /// Required statements
    pub good_faith_statement: bool,
    pub accuracy_statement: bool,
    pub authorization_statement: bool,
    /// Signature (physical or electronic)
    pub signature: String,
    /// Processing status
    pub status: DmcaStatus,
    /// User who uploaded the content
    pub affected_user_email: Option<String>,
    /// When content was removed
    pub removed_at: Option<DateTime<Utc>>,
    /// Counter-notice if filed
    pub counter_notice: Option<DmcaCounterNotice>,
}

/// DMCA counter-notice
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DmcaCounterNotice {
    pub id: Uuid,
    pub received_at: DateTime<Utc>,
    pub subscriber_name: String,
    pub subscriber_email: String,
    pub subscriber_address: String,
    /// Identification of removed material
    pub material_description: String,
    pub original_location: String,
    /// Required statements
    pub mistake_or_misidentification_statement: bool,
    pub consent_to_jurisdiction: bool,
    pub signature: String,
    /// When content should be restored (10-14 business days)
    pub restore_date: DateTime<Utc>,
    pub status: DmcaStatus,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum DmcaStatus {
    Received,
    UnderReview,
    ContentRemoved,
    CounterNoticeFiled,
    ContentRestored,
    CourtActionFiled,
    Resolved,
    Invalid,
}

impl DmcaCompliance {
    /// Validate a takedown notice has required elements
    pub fn validate_notice(&self, notice: &DmcaTakedownNotice) -> Result<(), ComplianceError> {
        let mut missing = Vec::new();

        if notice.claimant_name.is_empty() {
            missing.push("claimant name");
        }
        if notice.claimant_email.is_empty() {
            missing.push("claimant contact information");
        }
        if notice.copyrighted_work_description.is_empty() {
            missing.push("identification of copyrighted work");
        }
        if notice.infringing_material_location.is_empty() {
            missing.push("location of infringing material");
        }
        if !notice.good_faith_statement {
            missing.push("good faith statement");
        }
        if !notice.accuracy_statement {
            missing.push("accuracy statement under penalty of perjury");
        }
        if notice.signature.is_empty() {
            missing.push("signature");
        }

        if missing.is_empty() {
            Ok(())
        } else {
            Err(ComplianceError::InvalidDmcaNotice(format!(
                "Missing required elements: {}",
                missing.join(", ")
            )))
        }
    }

    /// Validate a counter-notice has required elements
    pub fn validate_counter_notice(&self, notice: &DmcaCounterNotice) -> Result<(), ComplianceError> {
        let mut missing = Vec::new();

        if notice.subscriber_name.is_empty() {
            missing.push("subscriber name");
        }
        if notice.subscriber_address.is_empty() {
            missing.push("subscriber address");
        }
        if notice.material_description.is_empty() {
            missing.push("identification of removed material");
        }
        if !notice.mistake_or_misidentification_statement {
            missing.push("statement of mistake or misidentification");
        }
        if !notice.consent_to_jurisdiction {
            missing.push("consent to federal court jurisdiction");
        }
        if notice.signature.is_empty() {
            missing.push("signature");
        }

        if missing.is_empty() {
            Ok(())
        } else {
            Err(ComplianceError::InvalidDmcaCounterNotice(format!(
                "Missing required elements: {}",
                missing.join(", ")
            )))
        }
    }

    /// Calculate restore date (10-14 business days from counter-notice)
    pub fn calculate_restore_date(&self, counter_notice_date: DateTime<Utc>) -> DateTime<Utc> {
        // Add the configured wait days (default 14)
        counter_notice_date + chrono::Duration::days(self.counter_notice_wait_days as i64)
    }
}

// =============================================================================
// 3. COPPA Compliance (Children's Online Privacy Protection)
// =============================================================================
// Applies if service is directed at children under 13 or knowingly collects
// their personal information. Requires verifiable parental consent.

/// COPPA compliance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoppaCompliance {
    /// Is service directed at children under 13?
    pub directed_at_children: bool,
    /// Minimum age requirement
    pub minimum_age: u8,
    /// Parental consent mechanism URL
    pub parental_consent_url: Option<String>,
    /// Privacy policy URL (must be prominent)
    pub privacy_policy_url: String,
    /// Data collected from children (must be minimized)
    pub data_collected: Vec<String>,
}

impl Default for CoppaCompliance {
    fn default() -> Self {
        Self {
            directed_at_children: false,
            minimum_age: 13, // COPPA threshold
            parental_consent_url: None,
            privacy_policy_url: String::new(),
            data_collected: Vec::new(),
        }
    }
}

impl CoppaCompliance {
    /// Check if parental consent is required for a user
    pub fn requires_parental_consent(&self, user_age: Option<u8>) -> bool {
        match user_age {
            Some(age) => age < self.minimum_age,
            None => self.directed_at_children,
        }
    }
}

// =============================================================================
// 4. GDPR Compliance (EU Data Protection)
// =============================================================================
// Applies to EU residents. Key requirements:
// - Lawful basis for processing
// - Data minimization
// - Right to access, rectification, erasure
// - Data breach notification (72 hours)
// - Data Protection Impact Assessments

/// GDPR compliance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GdprCompliance {
    /// Data Protection Officer contact
    pub dpo_email: Option<String>,
    /// Supervisory authority for complaints
    pub supervisory_authority: Option<String>,
    /// Data retention periods by category
    pub retention_periods: HashMap<String, u32>,
    /// Lawful bases for processing
    pub lawful_bases: Vec<LawfulBasis>,
    /// Enable automatic data subject request handling
    pub auto_dsr_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LawfulBasis {
    Consent,
    Contract,
    LegalObligation,
    VitalInterests,
    PublicTask,
    LegitimateInterests(String),
}

impl Default for GdprCompliance {
    fn default() -> Self {
        let mut retention = HashMap::new();
        retention.insert("email_content".to_string(), 365); // 1 year default
        retention.insert("email_metadata".to_string(), 730); // 2 years
        retention.insert("audit_logs".to_string(), 2190); // 6 years (legal requirement)
        retention.insert("consent_records".to_string(), 2555); // 7 years

        Self {
            dpo_email: None,
            supervisory_authority: None,
            retention_periods: retention,
            lawful_bases: vec![LawfulBasis::Contract, LawfulBasis::LegalObligation],
            auto_dsr_enabled: true,
        }
    }
}

/// Data Subject Request types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataSubjectRequest {
    /// Right to access (Article 15)
    Access { subject_email: String },
    /// Right to rectification (Article 16)
    Rectification { subject_email: String, corrections: HashMap<String, String> },
    /// Right to erasure / Right to be forgotten (Article 17)
    Erasure { subject_email: String, reason: ErasureReason },
    /// Right to data portability (Article 20)
    Portability { subject_email: String, format: String },
    /// Right to object (Article 21)
    Objection { subject_email: String, processing_type: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErasureReason {
    NoLongerNecessary,
    ConsentWithdrawn,
    ObjectionToProcessing,
    UnlawfulProcessing,
    LegalObligation,
    ChildData,
}

/// Data breach notification record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataBreachNotification {
    pub id: Uuid,
    pub detected_at: DateTime<Utc>,
    pub nature_of_breach: String,
    pub categories_affected: Vec<String>,
    pub approximate_records_affected: u64,
    pub likely_consequences: String,
    pub measures_taken: String,
    /// Must notify supervisory authority within 72 hours
    pub authority_notified_at: Option<DateTime<Utc>>,
    /// Must notify affected individuals if high risk
    pub individuals_notified_at: Option<DateTime<Utc>>,
    pub status: BreachStatus,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum BreachStatus {
    Detected,
    UnderInvestigation,
    AuthorityNotified,
    IndividualsNotified,
    Resolved,
}

impl GdprCompliance {
    /// Check if breach notification deadline is approaching (72 hours)
    pub fn breach_notification_deadline(&self, detected_at: DateTime<Utc>) -> DateTime<Utc> {
        detected_at + chrono::Duration::hours(72)
    }

    /// Check if a data subject request should be auto-processed
    pub fn should_auto_process(&self, request: &DataSubjectRequest) -> bool {
        self.auto_dsr_enabled
    }
}

// =============================================================================
// 5. Data Retention & Right to be Forgotten
// =============================================================================

/// Data retention policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionPolicy {
    /// Default retention period in days
    pub default_retention_days: u32,
    /// Category-specific retention periods
    pub category_retention: HashMap<String, u32>,
    /// Legal hold - data that cannot be deleted
    pub legal_holds: Vec<LegalHold>,
    /// Enable automatic purging of expired data
    pub auto_purge_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegalHold {
    pub id: Uuid,
    pub reason: String,
    pub created_at: DateTime<Utc>,
    pub created_by: String,
    /// Affected email addresses or domains
    pub scope: Vec<String>,
    /// Hold expires at (None = indefinite)
    pub expires_at: Option<DateTime<Utc>>,
}

impl Default for RetentionPolicy {
    fn default() -> Self {
        let mut categories = HashMap::new();
        categories.insert("transactional".to_string(), 2555); // 7 years (tax records)
        categories.insert("marketing".to_string(), 365); // 1 year
        categories.insert("support".to_string(), 730); // 2 years
        categories.insert("legal".to_string(), 3650); // 10 years

        Self {
            default_retention_days: 365,
            category_retention: categories,
            legal_holds: Vec::new(),
            auto_purge_enabled: false, // Disabled by default for safety
        }
    }
}

// =============================================================================
// 6. Compliance Audit Logging
// =============================================================================

/// Compliance audit log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceAuditLog {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub event_type: ComplianceEventType,
    pub actor: String,
    pub subject: Option<String>,
    pub action: String,
    pub details: HashMap<String, String>,
    pub ip_address: Option<String>,
    pub success: bool,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComplianceEventType {
    // CSAM
    CsamDetection,
    CsamReport,
    // DMCA
    DmcaTakedownReceived,
    DmcaContentRemoved,
    DmcaCounterNoticeReceived,
    DmcaContentRestored,
    // GDPR
    DataSubjectRequest,
    DataBreach,
    ConsentRecorded,
    ConsentWithdrawn,
    // General
    DataAccess,
    DataExport,
    DataDeletion,
    PolicyChange,
    AuditAccess,
}

impl ComplianceAuditLog {
    pub fn new(event_type: ComplianceEventType, actor: &str, action: &str) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            event_type,
            actor: actor.to_string(),
            subject: None,
            action: action.to_string(),
            details: HashMap::new(),
            ip_address: None,
            success: true,
            error_message: None,
        }
    }

    pub fn with_subject(mut self, subject: &str) -> Self {
        self.subject = Some(subject.to_string());
        self
    }

    pub fn with_detail(mut self, key: &str, value: &str) -> Self {
        self.details.insert(key.to_string(), value.to_string());
        self
    }

    pub fn with_ip(mut self, ip: &str) -> Self {
        self.ip_address = Some(ip.to_string());
        self
    }

    pub fn failed(mut self, error: &str) -> Self {
        self.success = false;
        self.error_message = Some(error.to_string());
        self
    }
}

// =============================================================================
// Master Compliance Configuration
// =============================================================================

/// Master compliance configuration combining all requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceConfig {
    pub csam: CsamCompliance,
    pub dmca: DmcaCompliance,
    pub coppa: CoppaCompliance,
    pub gdpr: GdprCompliance,
    pub retention: RetentionPolicy,
    /// Enable compliance mode (stricter logging, content scanning)
    pub compliance_mode_enabled: bool,
    /// Jurisdiction (affects which laws apply)
    pub jurisdiction: Jurisdiction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Jurisdiction {
    /// United States - CAN-SPAM, DMCA, COPPA, CSAM reporting
    UnitedStates,
    /// European Union - GDPR, ePrivacy
    EuropeanUnion,
    /// United Kingdom - UK GDPR, DPA 2018
    UnitedKingdom,
    /// Canada - PIPEDA, CASL
    Canada,
    /// Australia - Privacy Act, Spam Act
    Australia,
    /// Multiple jurisdictions
    Multiple(Vec<String>),
    /// Custom/Other
    Other(String),
}

impl Default for ComplianceConfig {
    fn default() -> Self {
        Self {
            csam: CsamCompliance::default(),
            dmca: DmcaCompliance::default(),
            coppa: CoppaCompliance::default(),
            gdpr: GdprCompliance::default(),
            retention: RetentionPolicy::default(),
            compliance_mode_enabled: true,
            jurisdiction: Jurisdiction::UnitedStates,
        }
    }
}

impl ComplianceConfig {
    /// Check which laws apply based on jurisdiction
    pub fn applicable_laws(&self) -> Vec<&'static str> {
        match &self.jurisdiction {
            Jurisdiction::UnitedStates => vec![
                "CAN-SPAM Act",
                "DMCA (17 USC 512)",
                "COPPA",
                "18 USC 2258A (CSAM Reporting)",
                "ECPA",
            ],
            Jurisdiction::EuropeanUnion => vec![
                "GDPR",
                "ePrivacy Directive",
                "Digital Services Act",
            ],
            Jurisdiction::UnitedKingdom => vec![
                "UK GDPR",
                "Data Protection Act 2018",
                "PECR",
            ],
            Jurisdiction::Canada => vec![
                "PIPEDA",
                "CASL",
            ],
            Jurisdiction::Australia => vec![
                "Privacy Act 1988",
                "Spam Act 2003",
            ],
            Jurisdiction::Multiple(_) | Jurisdiction::Other(_) => vec![
                "Multiple jurisdictions - consult legal counsel",
            ],
        }
    }
}

// =============================================================================
// Compliance Errors
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComplianceError {
    /// NCMEC API not configured for CSAM reporting
    NcmecNotConfigured(String),
    /// Invalid DMCA takedown notice
    InvalidDmcaNotice(String),
    /// Invalid DMCA counter-notice
    InvalidDmcaCounterNotice(String),
    /// GDPR violation
    GdprViolation(String),
    /// COPPA violation
    CoppaViolation(String),
    /// Data retention violation
    RetentionViolation(String),
    /// Legal hold prevents deletion
    LegalHoldActive(String),
    /// General compliance error
    Other(String),
}

impl std::fmt::Display for ComplianceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NcmecNotConfigured(msg) => write!(f, "NCMEC not configured: {}", msg),
            Self::InvalidDmcaNotice(msg) => write!(f, "Invalid DMCA notice: {}", msg),
            Self::InvalidDmcaCounterNotice(msg) => write!(f, "Invalid DMCA counter-notice: {}", msg),
            Self::GdprViolation(msg) => write!(f, "GDPR violation: {}", msg),
            Self::CoppaViolation(msg) => write!(f, "COPPA violation: {}", msg),
            Self::RetentionViolation(msg) => write!(f, "Retention violation: {}", msg),
            Self::LegalHoldActive(msg) => write!(f, "Legal hold active: {}", msg),
            Self::Other(msg) => write!(f, "Compliance error: {}", msg),
        }
    }
}

impl std::error::Error for ComplianceError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dmca_notice_validation() {
        let dmca = DmcaCompliance::default();
        
        let valid_notice = DmcaTakedownNotice {
            id: Uuid::new_v4(),
            received_at: Utc::now(),
            claimant_name: "John Doe".to_string(),
            claimant_email: "john@example.com".to_string(),
            claimant_address: Some("123 Main St".to_string()),
            copyrighted_work_description: "My copyrighted photo".to_string(),
            copyrighted_work_url: Some("https://example.com/original".to_string()),
            infringing_material_description: "Unauthorized copy".to_string(),
            infringing_material_location: "https://target.com/infringing".to_string(),
            good_faith_statement: true,
            accuracy_statement: true,
            authorization_statement: true,
            signature: "John Doe".to_string(),
            status: DmcaStatus::Received,
            affected_user_email: None,
            removed_at: None,
            counter_notice: None,
        };

        assert!(dmca.validate_notice(&valid_notice).is_ok());
    }

    #[test]
    fn test_dmca_notice_validation_missing_fields() {
        let dmca = DmcaCompliance::default();
        
        let invalid_notice = DmcaTakedownNotice {
            id: Uuid::new_v4(),
            received_at: Utc::now(),
            claimant_name: String::new(), // Missing
            claimant_email: String::new(), // Missing
            claimant_address: None,
            copyrighted_work_description: String::new(), // Missing
            copyrighted_work_url: None,
            infringing_material_description: String::new(),
            infringing_material_location: String::new(), // Missing
            good_faith_statement: false, // Missing
            accuracy_statement: false, // Missing
            authorization_statement: false,
            signature: String::new(), // Missing
            status: DmcaStatus::Received,
            affected_user_email: None,
            removed_at: None,
            counter_notice: None,
        };

        assert!(dmca.validate_notice(&invalid_notice).is_err());
    }

    #[test]
    fn test_coppa_parental_consent() {
        let coppa = CoppaCompliance::default();
        
        assert!(coppa.requires_parental_consent(Some(10)));
        assert!(coppa.requires_parental_consent(Some(12)));
        assert!(!coppa.requires_parental_consent(Some(13)));
        assert!(!coppa.requires_parental_consent(Some(18)));
    }

    #[test]
    fn test_jurisdiction_laws() {
        let us_config = ComplianceConfig {
            jurisdiction: Jurisdiction::UnitedStates,
            ..Default::default()
        };
        
        let laws = us_config.applicable_laws();
        assert!(laws.contains(&"CAN-SPAM Act"));
        assert!(laws.contains(&"DMCA (17 USC 512)"));
        assert!(laws.contains(&"18 USC 2258A (CSAM Reporting)"));
    }
}
