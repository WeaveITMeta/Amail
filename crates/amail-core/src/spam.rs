// =============================================================================
// SPAM Detection, Auto-Reply Prevention, and CAN-SPAM Compliance
// =============================================================================
// Table of Contents:
// 1. SpamScore - Spam scoring and classification
// 2. SpamFilter - Configurable spam filtering rules
// 3. SpamIndicators - Individual spam signals
// 4. AutoReplyDetector - Detect machine-generated emails (prevent reply loops)
// 5. CAN-SPAM Compliance - Outbound email compliance
// =============================================================================

use crate::email::{EmailMessage, EmailMetadata};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

// =============================================================================
// 1. Spam Score and Classification
// =============================================================================

/// Spam classification result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpamResult {
    /// Total spam score (0-100, higher = more likely spam)
    pub score: f32,
    /// Classification
    pub classification: SpamClassification,
    /// Individual indicators that contributed to score
    pub indicators: Vec<SpamIndicator>,
    /// Whether email should be delivered
    pub should_deliver: bool,
    /// Recommended folder
    pub folder: SpamFolder,
}

/// Spam classification levels
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum SpamClassification {
    /// Definitely not spam (score 0-20)
    Ham,
    /// Probably not spam (score 21-40)
    LikelyHam,
    /// Uncertain (score 41-60)
    Uncertain,
    /// Probably spam (score 61-80)
    LikelySpam,
    /// Definitely spam (score 81-100)
    Spam,
}

impl SpamClassification {
    pub fn from_score(score: f32) -> Self {
        match score as u32 {
            0..=20 => Self::Ham,
            21..=40 => Self::LikelyHam,
            41..=60 => Self::Uncertain,
            61..=80 => Self::LikelySpam,
            _ => Self::Spam,
        }
    }
}

/// Recommended folder for email
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum SpamFolder {
    Inbox,
    Spam,
    Quarantine,
    Reject,
}

// =============================================================================
// 2. Spam Indicators
// =============================================================================

/// Individual spam indicator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpamIndicator {
    /// Indicator name
    pub name: String,
    /// Score contribution (positive = spam, negative = ham)
    pub score: f32,
    /// Description
    pub description: String,
}

impl SpamIndicator {
    pub fn new(name: impl Into<String>, score: f32, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            score,
            description: description.into(),
        }
    }
}

// =============================================================================
// 3. Spam Filter
// =============================================================================

/// Configurable spam filter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpamFilter {
    /// Spam threshold (emails above this go to spam)
    pub spam_threshold: f32,
    /// Quarantine threshold (emails above this are quarantined)
    pub quarantine_threshold: f32,
    /// Reject threshold (emails above this are rejected)
    pub reject_threshold: f32,
    /// Blocked sender domains
    pub blocked_domains: HashSet<String>,
    /// Blocked sender addresses
    pub blocked_addresses: HashSet<String>,
    /// Allowed sender domains (whitelist)
    pub allowed_domains: HashSet<String>,
    /// Allowed sender addresses (whitelist)
    pub allowed_addresses: HashSet<String>,
    /// Spam keywords with scores
    pub spam_keywords: Vec<(String, f32)>,
    /// Check authentication (SPF, DKIM, DMARC)
    pub check_authentication: bool,
    /// Check for suspicious URLs
    pub check_urls: bool,
    /// Check for excessive capitalization
    pub check_caps: bool,
}

impl Default for SpamFilter {
    fn default() -> Self {
        Self {
            spam_threshold: 50.0,
            quarantine_threshold: 80.0,
            reject_threshold: 95.0,
            blocked_domains: HashSet::new(),
            blocked_addresses: HashSet::new(),
            allowed_domains: HashSet::new(),
            allowed_addresses: HashSet::new(),
            spam_keywords: default_spam_keywords(),
            check_authentication: true,
            check_urls: true,
            check_caps: true,
        }
    }
}

impl SpamFilter {
    /// Analyze an email for spam
    pub fn analyze(&self, email: &EmailMessage) -> SpamResult {
        let mut indicators = Vec::new();
        let mut total_score: f32 = 0.0;

        // Check whitelist first
        if self.is_whitelisted(&email.metadata) {
            return SpamResult {
                score: 0.0,
                classification: SpamClassification::Ham,
                indicators: vec![SpamIndicator::new(
                    "whitelist",
                    -100.0,
                    "Sender is whitelisted",
                )],
                should_deliver: true,
                folder: SpamFolder::Inbox,
            };
        }

        // Check blacklist
        if self.is_blacklisted(&email.metadata) {
            return SpamResult {
                score: 100.0,
                classification: SpamClassification::Spam,
                indicators: vec![SpamIndicator::new(
                    "blacklist",
                    100.0,
                    "Sender is blacklisted",
                )],
                should_deliver: false,
                folder: SpamFolder::Reject,
            };
        }

        // Check authentication
        if self.check_authentication {
            let auth_indicators = self.check_email_authentication(&email.metadata);
            for ind in auth_indicators {
                total_score += ind.score;
                indicators.push(ind);
            }
        }

        // Check subject
        let subject_indicators = self.check_subject(&email.metadata.subject);
        for ind in subject_indicators {
            total_score += ind.score;
            indicators.push(ind);
        }

        // Check body
        if let Some(body) = email.body_text() {
            let body_indicators = self.check_body(body);
            for ind in body_indicators {
                total_score += ind.score;
                indicators.push(ind);
            }
        }

        // Check URLs
        if self.check_urls {
            if let Some(body) = email.body_text() {
                let url_indicators = self.check_urls_in_text(body);
                for ind in url_indicators {
                    total_score += ind.score;
                    indicators.push(ind);
                }
            }
        }

        // Check sender patterns
        let sender_indicators = self.check_sender(&email.metadata);
        for ind in sender_indicators {
            total_score += ind.score;
            indicators.push(ind);
        }

        // Clamp score to 0-100
        total_score = total_score.clamp(0.0, 100.0);

        let classification = SpamClassification::from_score(total_score);
        let folder = self.determine_folder(total_score);
        let should_deliver = folder != SpamFolder::Reject;

        SpamResult {
            score: total_score,
            classification,
            indicators,
            should_deliver,
            folder,
        }
    }

    /// Check if sender is whitelisted
    fn is_whitelisted(&self, metadata: &EmailMetadata) -> bool {
        let address = metadata.from.address.to_lowercase();
        if self.allowed_addresses.contains(&address) {
            return true;
        }
        if let Some(domain) = metadata.from.domain() {
            if self.allowed_domains.contains(&domain.to_lowercase()) {
                return true;
            }
        }
        false
    }

    /// Check if sender is blacklisted
    fn is_blacklisted(&self, metadata: &EmailMetadata) -> bool {
        let address = metadata.from.address.to_lowercase();
        if self.blocked_addresses.contains(&address) {
            return true;
        }
        if let Some(domain) = metadata.from.domain() {
            if self.blocked_domains.contains(&domain.to_lowercase()) {
                return true;
            }
        }
        false
    }

    /// Check email authentication results
    fn check_email_authentication(&self, metadata: &EmailMetadata) -> Vec<SpamIndicator> {
        let mut indicators = Vec::new();

        // SPF check
        match metadata.spf_result.as_deref() {
            Some("pass") => indicators.push(SpamIndicator::new("spf_pass", -5.0, "SPF authentication passed")),
            Some("fail") | Some("softfail") => indicators.push(SpamIndicator::new("spf_fail", 15.0, "SPF authentication failed")),
            Some("none") | None => indicators.push(SpamIndicator::new("spf_none", 5.0, "No SPF record")),
            _ => {}
        }

        // DKIM check
        match metadata.dkim_result.as_deref() {
            Some("pass") => indicators.push(SpamIndicator::new("dkim_pass", -5.0, "DKIM signature valid")),
            Some("fail") => indicators.push(SpamIndicator::new("dkim_fail", 15.0, "DKIM signature invalid")),
            Some("none") | None => indicators.push(SpamIndicator::new("dkim_none", 5.0, "No DKIM signature")),
            _ => {}
        }

        // DMARC check
        match metadata.dmarc_result.as_deref() {
            Some("pass") => indicators.push(SpamIndicator::new("dmarc_pass", -10.0, "DMARC policy passed")),
            Some("fail") => indicators.push(SpamIndicator::new("dmarc_fail", 20.0, "DMARC policy failed")),
            Some("none") | None => indicators.push(SpamIndicator::new("dmarc_none", 3.0, "No DMARC policy")),
            _ => {}
        }

        indicators
    }

    /// Check subject line for spam indicators
    fn check_subject(&self, subject: &str) -> Vec<SpamIndicator> {
        let mut indicators = Vec::new();
        let subject_lower = subject.to_lowercase();

        // Check spam keywords in subject
        for (keyword, score) in &self.spam_keywords {
            if subject_lower.contains(&keyword.to_lowercase()) {
                indicators.push(SpamIndicator::new(
                    format!("keyword_{}", keyword),
                    *score,
                    format!("Subject contains spam keyword: {}", keyword),
                ));
            }
        }

        // Check excessive caps
        if self.check_caps {
            let caps_ratio = subject.chars().filter(|c| c.is_uppercase()).count() as f32
                / subject.len().max(1) as f32;
            if caps_ratio > 0.5 && subject.len() > 10 {
                indicators.push(SpamIndicator::new(
                    "excessive_caps_subject",
                    10.0,
                    "Subject has excessive capitalization",
                ));
            }
        }

        // Check for RE:/FW: spam (fake replies)
        if subject_lower.starts_with("re:") || subject_lower.starts_with("fw:") {
            // This alone isn't spam, but combined with no references header it's suspicious
            // (handled elsewhere)
        }

        // Empty subject
        if subject.trim().is_empty() {
            indicators.push(SpamIndicator::new(
                "empty_subject",
                5.0,
                "Email has empty subject",
            ));
        }

        indicators
    }

    /// Check body for spam indicators
    fn check_body(&self, body: &str) -> Vec<SpamIndicator> {
        let mut indicators = Vec::new();
        let body_lower = body.to_lowercase();

        // Check spam keywords
        for (keyword, score) in &self.spam_keywords {
            if body_lower.contains(&keyword.to_lowercase()) {
                indicators.push(SpamIndicator::new(
                    format!("body_keyword_{}", keyword),
                    *score * 0.5, // Body keywords weighted less than subject
                    format!("Body contains spam keyword: {}", keyword),
                ));
            }
        }

        // Check for excessive caps in body
        if self.check_caps && body.len() > 100 {
            let caps_ratio = body.chars().filter(|c| c.is_uppercase()).count() as f32
                / body.len() as f32;
            if caps_ratio > 0.3 {
                indicators.push(SpamIndicator::new(
                    "excessive_caps_body",
                    8.0,
                    "Body has excessive capitalization",
                ));
            }
        }

        // Check for common spam phrases
        let spam_phrases = [
            ("click here", 5.0),
            ("act now", 8.0),
            ("limited time", 6.0),
            ("congratulations", 5.0),
            ("you have won", 15.0),
            ("claim your prize", 15.0),
            ("nigerian prince", 25.0),
            ("wire transfer", 10.0),
            ("bank account", 5.0),
            ("social security", 8.0),
            ("verify your account", 10.0),
            ("suspended account", 10.0),
            ("unusual activity", 8.0),
            ("confirm your identity", 8.0),
            ("unsubscribe", -3.0), // Legitimate emails have unsubscribe
        ];

        for (phrase, score) in spam_phrases {
            if body_lower.contains(phrase) {
                indicators.push(SpamIndicator::new(
                    format!("phrase_{}", phrase.replace(' ', "_")),
                    score,
                    format!("Body contains phrase: {}", phrase),
                ));
            }
        }

        indicators
    }

    /// Check URLs in text
    fn check_urls_in_text(&self, text: &str) -> Vec<SpamIndicator> {
        let mut indicators = Vec::new();

        // Simple URL detection
        let url_count = text.matches("http://").count() + text.matches("https://").count();

        if url_count > 10 {
            indicators.push(SpamIndicator::new(
                "many_urls",
                10.0,
                format!("Email contains {} URLs", url_count),
            ));
        }

        // Check for URL shorteners (often used in spam)
        let shorteners = ["bit.ly", "tinyurl", "goo.gl", "t.co", "ow.ly", "is.gd"];
        for shortener in shorteners {
            if text.to_lowercase().contains(shortener) {
                indicators.push(SpamIndicator::new(
                    "url_shortener",
                    8.0,
                    format!("Contains URL shortener: {}", shortener),
                ));
                break;
            }
        }

        // Check for suspicious TLDs
        let suspicious_tlds = [".xyz", ".top", ".click", ".loan", ".work", ".gq", ".ml", ".cf", ".tk"];
        for tld in suspicious_tlds {
            if text.to_lowercase().contains(tld) {
                indicators.push(SpamIndicator::new(
                    "suspicious_tld",
                    5.0,
                    format!("Contains suspicious TLD: {}", tld),
                ));
                break;
            }
        }

        indicators
    }

    /// Check sender patterns
    fn check_sender(&self, metadata: &EmailMetadata) -> Vec<SpamIndicator> {
        let mut indicators = Vec::new();
        let address = &metadata.from.address;

        // Check for random-looking sender
        if let Some(local) = metadata.from.local_part() {
            // Check for excessive numbers in local part
            let digit_ratio = local.chars().filter(|c| c.is_numeric()).count() as f32
                / local.len().max(1) as f32;
            if digit_ratio > 0.5 && local.len() > 5 {
                indicators.push(SpamIndicator::new(
                    "random_sender",
                    10.0,
                    "Sender address looks randomly generated",
                ));
            }
        }

        // Check for display name spoofing
        if let Some(name) = &metadata.from.name {
            // Name contains email address (potential spoofing)
            if name.contains('@') {
                indicators.push(SpamIndicator::new(
                    "name_spoofing",
                    15.0,
                    "Display name contains email address (potential spoofing)",
                ));
            }
        }

        // Check for mismatched reply-to
        if let Some(reply_to) = &metadata.reply_to {
            if reply_to.domain() != metadata.from.domain() {
                indicators.push(SpamIndicator::new(
                    "mismatched_reply_to",
                    8.0,
                    "Reply-To domain differs from From domain",
                ));
            }
        }

        indicators
    }

    /// Determine folder based on score
    fn determine_folder(&self, score: f32) -> SpamFolder {
        if score >= self.reject_threshold {
            SpamFolder::Reject
        } else if score >= self.quarantine_threshold {
            SpamFolder::Quarantine
        } else if score >= self.spam_threshold {
            SpamFolder::Spam
        } else {
            SpamFolder::Inbox
        }
    }
}

/// Default spam keywords with scores
fn default_spam_keywords() -> Vec<(String, f32)> {
    vec![
        ("viagra".to_string(), 15.0),
        ("cialis".to_string(), 15.0),
        ("lottery".to_string(), 12.0),
        ("winner".to_string(), 8.0),
        ("free money".to_string(), 15.0),
        ("make money fast".to_string(), 15.0),
        ("work from home".to_string(), 5.0),
        ("earn extra cash".to_string(), 8.0),
        ("double your".to_string(), 10.0),
        ("million dollars".to_string(), 12.0),
        ("cash bonus".to_string(), 8.0),
        ("credit card".to_string(), 3.0),
        ("no credit check".to_string(), 10.0),
        ("refinance".to_string(), 5.0),
        ("mortgage rates".to_string(), 5.0),
        ("weight loss".to_string(), 8.0),
        ("diet pill".to_string(), 10.0),
        ("enlargement".to_string(), 15.0),
        ("casino".to_string(), 10.0),
        ("gambling".to_string(), 8.0),
        ("adult content".to_string(), 12.0),
        ("xxx".to_string(), 15.0),
    ]
}

// =============================================================================
// 4. Auto-Reply Detection (Prevent Reply Loops)
// =============================================================================

/// Detects machine-generated emails to prevent auto-reply loops
/// 
/// Checks for:
/// - Auto-Submitted header (RFC 3834)
/// - Precedence header (bulk, junk, list)
/// - X-Auto-Response-Suppress header
/// - Common automated sender patterns (noreply, mailer-daemon, etc.)
/// - List-Unsubscribe header (mailing lists)
#[derive(Debug, Clone, Default)]
pub struct AutoReplyDetector;

/// Result of auto-reply detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoReplyResult {
    /// Whether this email is machine-generated
    pub is_automated: bool,
    /// Whether we should auto-reply to this email
    pub should_auto_reply: bool,
    /// Reasons why this was detected as automated
    pub reasons: Vec<String>,
}

impl AutoReplyDetector {
    /// Check if an email is machine-generated and should not receive auto-replies
    pub fn analyze(&self, email: &EmailMessage) -> AutoReplyResult {
        let mut reasons = Vec::new();
        let mut is_automated = false;

        let headers = &email.metadata.custom_headers;

        // RFC 3834: Auto-Submitted header
        if let Some(auto_submitted) = headers.get("Auto-Submitted").or(headers.get("auto-submitted")) {
            let value = auto_submitted.to_lowercase();
            if value != "no" {
                is_automated = true;
                reasons.push(format!("Auto-Submitted: {}", auto_submitted));
            }
        }

        // Precedence header (bulk, junk, list, auto_reply)
        if let Some(precedence) = headers.get("Precedence").or(headers.get("precedence")) {
            let value = precedence.to_lowercase();
            if value == "bulk" || value == "junk" || value == "list" || value == "auto_reply" {
                is_automated = true;
                reasons.push(format!("Precedence: {}", precedence));
            }
        }

        // X-Auto-Response-Suppress header (Microsoft)
        if headers.contains_key("X-Auto-Response-Suppress") || headers.contains_key("x-auto-response-suppress") {
            is_automated = true;
            reasons.push("X-Auto-Response-Suppress header present".to_string());
        }

        // X-Autoreply header
        if headers.contains_key("X-Autoreply") || headers.contains_key("x-autoreply") {
            is_automated = true;
            reasons.push("X-Autoreply header present".to_string());
        }

        // X-Autorespond header
        if headers.contains_key("X-Autorespond") || headers.contains_key("x-autorespond") {
            is_automated = true;
            reasons.push("X-Autorespond header present".to_string());
        }

        // List-Id or List-Unsubscribe (mailing list)
        if headers.contains_key("List-Id") || headers.contains_key("list-id") ||
           headers.contains_key("List-Unsubscribe") || headers.contains_key("list-unsubscribe") {
            is_automated = true;
            reasons.push("Mailing list headers present".to_string());
        }

        // Check sender address patterns
        let from_address = email.metadata.from.address.to_lowercase();
        let from_local = email.metadata.from.local_part().unwrap_or("").to_lowercase();

        // Common automated sender patterns
        let automated_patterns = [
            "noreply",
            "no-reply",
            "no_reply",
            "donotreply",
            "do-not-reply",
            "do_not_reply",
            "mailer-daemon",
            "mailerdaemon",
            "postmaster",
            "bounce",
            "bounces",
            "notifications",
            "notification",
            "alert",
            "alerts",
            "system",
            "automated",
            "auto",
            "robot",
            "bot",
            "daemon",
            "admin@",
            "support@", // Often automated ticket systems
        ];

        for pattern in automated_patterns {
            if from_local.contains(pattern) || from_address.contains(pattern) {
                is_automated = true;
                reasons.push(format!("Sender matches automated pattern: {}", pattern));
                break;
            }
        }

        // Check for common automated sender domains
        let automated_domains = [
            "googlemail.com", // Google notifications
            "facebookmail.com",
            "linkedin.com",
            "twitter.com",
            "x.com",
            "github.com",
            "amazonses.com",
            "sendgrid.net",
            "mailchimp.com",
            "mailgun.org",
            "postmarkapp.com",
            "sparkpostmail.com",
        ];

        if let Some(domain) = email.metadata.from.domain() {
            let domain_lower = domain.to_lowercase();
            for automated_domain in automated_domains {
                if domain_lower.ends_with(automated_domain) {
                    is_automated = true;
                    reasons.push(format!("Sender from automated service: {}", automated_domain));
                    break;
                }
            }
        }

        // Check subject for auto-reply indicators
        let subject_lower = email.metadata.subject.to_lowercase();
        let auto_subject_patterns = [
            "auto:",
            "automatic reply",
            "autoreply",
            "auto-reply",
            "out of office",
            "ooo:",
            "vacation reply",
            "away message",
            "delivery status",
            "delivery notification",
            "undeliverable",
            "returned mail",
            "mail delivery failed",
            "failure notice",
            "read receipt",
            "delivery receipt",
        ];

        for pattern in auto_subject_patterns {
            if subject_lower.contains(pattern) {
                is_automated = true;
                reasons.push(format!("Subject indicates auto-reply: {}", pattern));
                break;
            }
        }

        // Check for Return-Path indicating bounce
        if let Some(return_path) = headers.get("Return-Path").or(headers.get("return-path")) {
            let rp_lower = return_path.to_lowercase();
            if rp_lower.contains("<>") || rp_lower.contains("mailer-daemon") || rp_lower.contains("postmaster") {
                is_automated = true;
                reasons.push("Return-Path indicates bounce message".to_string());
            }
        }

        // Check Content-Type for delivery status notifications
        if let Some(content_type) = headers.get("Content-Type").or(headers.get("content-type")) {
            if content_type.contains("multipart/report") || content_type.contains("delivery-status") {
                is_automated = true;
                reasons.push("Content-Type indicates delivery status notification".to_string());
            }
        }

        AutoReplyResult {
            is_automated,
            should_auto_reply: !is_automated,
            reasons,
        }
    }

    /// Quick check - returns true if we should NOT auto-reply
    pub fn is_automated(&self, email: &EmailMessage) -> bool {
        self.analyze(email).is_automated
    }

    /// Quick check - returns true if it's safe to auto-reply
    pub fn can_auto_reply(&self, email: &EmailMessage) -> bool {
        !self.is_automated(email)
    }
}

// =============================================================================
// 5. CAN-SPAM Compliance
// =============================================================================

/// CAN-SPAM compliance checker and helper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanSpamCompliance {
    /// Physical postal address (required by CAN-SPAM)
    pub physical_address: String,
    /// Organization name
    pub organization_name: String,
    /// Unsubscribe URL template
    pub unsubscribe_url_template: String,
    /// Unsubscribe email address
    pub unsubscribe_email: Option<String>,
}

impl CanSpamCompliance {
    /// Create new CAN-SPAM compliance config
    pub fn new(
        organization_name: impl Into<String>,
        physical_address: impl Into<String>,
        unsubscribe_url_template: impl Into<String>,
    ) -> Self {
        Self {
            organization_name: organization_name.into(),
            physical_address: physical_address.into(),
            unsubscribe_url_template: unsubscribe_url_template.into(),
            unsubscribe_email: None,
        }
    }

    /// Generate List-Unsubscribe header value
    pub fn unsubscribe_header(&self, recipient_id: &str) -> String {
        let url = self.unsubscribe_url_template.replace("{id}", recipient_id);
        if let Some(email) = &self.unsubscribe_email {
            format!("<{}>, <mailto:{}>", url, email)
        } else {
            format!("<{}>", url)
        }
    }

    /// Generate List-Unsubscribe-Post header (one-click unsubscribe)
    pub fn unsubscribe_post_header() -> &'static str {
        "List-Unsubscribe=One-Click"
    }

    /// Generate CAN-SPAM compliant footer for HTML emails
    pub fn html_footer(&self, recipient_id: &str) -> String {
        let unsubscribe_url = self.unsubscribe_url_template.replace("{id}", recipient_id);
        format!(
            r#"<div style="margin-top: 20px; padding-top: 20px; border-top: 1px solid #ccc; font-size: 12px; color: #666;">
    <p>{}</p>
    <p>{}</p>
    <p>
        <a href="{}" style="color: #666;">Unsubscribe</a> | 
        <a href="{}" style="color: #666;">Update Preferences</a>
    </p>
</div>"#,
            self.organization_name,
            self.physical_address,
            unsubscribe_url,
            unsubscribe_url.replace("unsubscribe", "preferences")
        )
    }

    /// Generate CAN-SPAM compliant footer for plain text emails
    pub fn text_footer(&self, recipient_id: &str) -> String {
        let unsubscribe_url = self.unsubscribe_url_template.replace("{id}", recipient_id);
        format!(
            r#"
---
{}
{}

To unsubscribe: {}
To update preferences: {}
"#,
            self.organization_name,
            self.physical_address,
            unsubscribe_url,
            unsubscribe_url.replace("unsubscribe", "preferences")
        )
    }

    /// Check if outgoing email is CAN-SPAM compliant
    pub fn check_compliance(&self, email: &EmailMessage) -> CanSpamCheckResult {
        let mut issues = Vec::new();
        let mut is_compliant = true;

        // Check for unsubscribe link in body
        let has_unsubscribe = email.body_text()
            .map(|b| b.to_lowercase().contains("unsubscribe"))
            .unwrap_or(false);
        
        if !has_unsubscribe {
            issues.push("Missing unsubscribe link/instructions".to_string());
            is_compliant = false;
        }

        // Check for physical address
        let has_address = email.body_text()
            .map(|b| {
                // Simple heuristic: look for address-like patterns
                b.contains(&self.physical_address) ||
                (b.contains("Street") || b.contains("Ave") || b.contains("Blvd") || 
                 b.contains("Suite") || b.contains("Floor"))
            })
            .unwrap_or(false);

        if !has_address {
            issues.push("Missing physical postal address".to_string());
            is_compliant = false;
        }

        // Check subject line isn't deceptive
        let subject = &email.metadata.subject;
        if subject.to_lowercase().starts_with("re:") && email.metadata.in_reply_to.is_none() {
            issues.push("Deceptive subject line (fake reply)".to_string());
            is_compliant = false;
        }
        if subject.to_lowercase().starts_with("fw:") && email.metadata.references.is_empty() {
            issues.push("Deceptive subject line (fake forward)".to_string());
            is_compliant = false;
        }

        // Check From header is accurate
        // (This is harder to verify programmatically, so we just note it)

        CanSpamCheckResult {
            is_compliant,
            issues,
        }
    }

    /// Add CAN-SPAM required headers to email
    pub fn add_headers(&self, headers: &mut std::collections::HashMap<String, String>, recipient_id: &str) {
        headers.insert(
            "List-Unsubscribe".to_string(),
            self.unsubscribe_header(recipient_id),
        );
        headers.insert(
            "List-Unsubscribe-Post".to_string(),
            Self::unsubscribe_post_header().to_string(),
        );
        headers.insert(
            "Precedence".to_string(),
            "bulk".to_string(),
        );
    }
}

/// Result of CAN-SPAM compliance check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanSpamCheckResult {
    /// Whether email is compliant
    pub is_compliant: bool,
    /// List of compliance issues
    pub issues: Vec<String>,
}

impl Default for CanSpamCompliance {
    fn default() -> Self {
        Self {
            organization_name: "Your Organization".to_string(),
            physical_address: "123 Main Street, City, State 12345".to_string(),
            unsubscribe_url_template: "https://example.com/unsubscribe?id={id}".to_string(),
            unsubscribe_email: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spam_classification() {
        assert_eq!(SpamClassification::from_score(10.0), SpamClassification::Ham);
        assert_eq!(SpamClassification::from_score(30.0), SpamClassification::LikelyHam);
        assert_eq!(SpamClassification::from_score(50.0), SpamClassification::Uncertain);
        assert_eq!(SpamClassification::from_score(70.0), SpamClassification::LikelySpam);
        assert_eq!(SpamClassification::from_score(90.0), SpamClassification::Spam);
    }

    #[test]
    fn test_spam_filter_whitelist() {
        let mut filter = SpamFilter::default();
        filter.allowed_addresses.insert("trusted@example.com".to_string());

        let email = create_test_email("trusted@example.com", "Test Subject", "Test body");
        let result = filter.analyze(&email);

        assert_eq!(result.classification, SpamClassification::Ham);
        assert!(result.should_deliver);
    }

    #[test]
    fn test_spam_filter_blacklist() {
        let mut filter = SpamFilter::default();
        filter.blocked_domains.insert("spammer.com".to_string());

        let email = create_test_email("bad@spammer.com", "Test Subject", "Test body");
        let result = filter.analyze(&email);

        assert_eq!(result.classification, SpamClassification::Spam);
        assert!(!result.should_deliver);
    }

    #[test]
    fn test_spam_keywords() {
        let filter = SpamFilter::default();
        let email = create_test_email(
            "sender@example.com",
            "You have won the lottery!",
            "Congratulations! You have won a million dollars. Click here to claim your prize.",
        );
        let result = filter.analyze(&email);

        assert!(result.score > 30.0);
        assert!(result.indicators.len() > 0);
    }

    fn create_test_email(from: &str, subject: &str, body: &str) -> EmailMessage {
        use crate::email::{EmailAddress, EmailMetadata, EmailPriority};
        use chrono::Utc;
        use uuid::Uuid;

        EmailMessage {
            metadata: EmailMetadata {
                message_id: "test@example.com".to_string(),
                tracking_id: Uuid::new_v4(),
                from: EmailAddress::new(from),
                to: vec![EmailAddress::new("recipient@example.com")],
                cc: vec![],
                bcc: vec![],
                reply_to: None,
                subject: subject.to_string(),
                received_at: Utc::now(),
                sent_at: None,
                in_reply_to: None,
                references: vec![],
                spf_result: None,
                dkim_result: None,
                dmarc_result: None,
                priority: EmailPriority::Normal,
                custom_headers: std::collections::HashMap::new(),
            },
            text_body: Some(body.to_string()),
            html_body: None,
            attachments: vec![],
            raw: None,
        }
    }
}
