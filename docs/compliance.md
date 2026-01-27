# Legal Compliance Guide

Amail is designed to help you comply with email-related laws and regulations. This guide covers the major legal requirements for operating an email service.

**IMPORTANT**: This documentation is for informational purposes only and does not constitute legal advice. Consult with a qualified attorney for your specific situation.

---

## Table of Contents

1. [CSAM Reporting (18 USC 2258A)](#csam-reporting)
2. [DMCA Takedown (17 USC 512)](#dmca-takedown)
3. [CAN-SPAM Act](#can-spam-act)
4. [COPPA (Children's Privacy)](#coppa)
5. [GDPR (EU Data Protection)](#gdpr)
6. [CCPA (California Privacy)](#ccpa)
7. [Data Retention](#data-retention)
8. [Configuration](#configuration)

---

## CSAM Reporting

### Legal Requirement (18 USC 2258A - REPORT Act)

**Electronic service providers are LEGALLY REQUIRED to report apparent child sexual abuse material (CSAM) to NCMEC.**

### Key Points

- **Mandatory Reporting**: If you become aware of apparent CSAM on your service, you MUST report it to NCMEC's CyberTipline
- **Penalties**: Up to $850,000 per violation for providers with <100M users
- **Evidence Retention**: Must preserve evidence for minimum 1 year after report
- **No Searching Required**: You're not required to actively search for CSAM, but must report if discovered

### What to Report

- Child pornography (18 USC 2256)
- Child sex trafficking (18 USC 1591)
- Online enticement of minors (18 USC 2422(b))
- Misleading domain names (18 USC 2252B)

### How Amail Helps

```rust
// Amail's compliance module includes CSAM reporting structures
use amail_core::compliance::{CsamCompliance, NcmecReport};

// Create a report when CSAM is discovered
let report = csam_compliance.create_report(
    CsamIncidentType::ChildPornography,
    "Description of incident",
    "content_hash",
    vec!["sender@example.com".to_string()],
    Some("192.168.1.1".to_string()),
);
```

### NCMEC CyberTipline

- **Website**: https://report.cybertipline.org/
- **Registration**: Service providers should register with NCMEC
- **API Access**: Available for automated reporting after registration

### Configuration

```env
# CSAM Compliance
COMPLIANCE_CSAM_ENABLED=true
COMPLIANCE_NCMEC_PROVIDER_ID=your_provider_id
COMPLIANCE_LAW_ENFORCEMENT_CONTACT=legal@yourdomain.com
COMPLIANCE_EVIDENCE_RETENTION_DAYS=365
```

---

## DMCA Takedown

### Legal Requirement (17 USC 512)

The Digital Millennium Copyright Act provides "safe harbor" protection for service providers who:

1. Designate a DMCA agent with the Copyright Office
2. Implement a repeat infringer policy
3. Respond expeditiously to takedown notices
4. Provide counter-notice mechanism

### Safe Harbor Requirements

| Requirement | Description |
|-------------|-------------|
| **DMCA Agent** | Register with Copyright Office at copyright.gov |
| **Policy** | Publish repeat infringer termination policy |
| **Takedown** | Remove infringing content expeditiously |
| **Counter-Notice** | Allow users to dispute takedowns |
| **Restore** | Restore content 10-14 days after counter-notice (if no lawsuit) |

### Takedown Notice Requirements

A valid DMCA takedown notice must include:

1. Identification of copyrighted work
2. Identification of infringing material with location
3. Contact information of complainant
4. Good faith statement
5. Accuracy statement under penalty of perjury
6. Signature (physical or electronic)

### Counter-Notice Requirements

A valid counter-notice must include:

1. Identification of removed material
2. Statement of mistake or misidentification
3. Consent to federal court jurisdiction
4. Subscriber name and address
5. Signature

### How Amail Helps

```rust
use amail_core::compliance::{DmcaCompliance, DmcaTakedownNotice};

// Validate incoming takedown notice
let dmca = DmcaCompliance::default();
match dmca.validate_notice(&notice) {
    Ok(_) => { /* Process valid notice */ }
    Err(e) => { /* Reject invalid notice */ }
}
```

### Configuration

```env
# DMCA Compliance
DMCA_AGENT_NAME=John Doe
DMCA_AGENT_EMAIL=dmca@yourdomain.com
DMCA_AGENT_ADDRESS=123 Main St, City, State 12345
DMCA_REGISTRATION_NUMBER=your_registration_number
DMCA_POLICY_URL=https://yourdomain.com/dmca-policy
DMCA_COUNTER_NOTICE_WAIT_DAYS=14
DMCA_REPEAT_INFRINGER_STRIKES=3
```

### Register Your DMCA Agent

1. Go to https://www.copyright.gov/dmca-directory/
2. Create an account
3. Submit your designation
4. Pay the fee ($6 per designation)
5. Update your website with agent contact info

---

## CAN-SPAM Act

### Requirements for Commercial Email

| Requirement | Description |
|-------------|-------------|
| **Accurate Headers** | From, To, Reply-To must be accurate |
| **Non-Deceptive Subject** | Subject must reflect content |
| **Identification** | Clearly identify as advertisement |
| **Physical Address** | Include valid postal address |
| **Opt-Out** | Provide clear unsubscribe mechanism |
| **Honor Opt-Outs** | Process within 10 business days |

### Penalties

- Up to **$51,744 per email** for violations
- No exemption for small businesses or nonprofits

### How Amail Helps

Amail's spam module includes CAN-SPAM compliance:

```rust
use amail_core::spam::CanSpamCompliance;

let canspam = CanSpamCompliance::new(
    "Your Organization",
    "123 Main Street, City, State 12345",
    "https://yourdomain.com/unsubscribe?id={id}",
);

// Add required headers
canspam.add_headers(&mut headers, recipient_id);

// Add compliant footer
let footer = canspam.html_footer(recipient_id);
```

### Configuration

```env
# CAN-SPAM Compliance
CANSPAM_ORG_NAME=Your Organization
CANSPAM_PHYSICAL_ADDRESS=123 Main Street, City, State 12345
CANSPAM_UNSUBSCRIBE_URL=https://yourdomain.com/unsubscribe?id={id}
```

---

## COPPA

### Children's Online Privacy Protection Act

Applies if your service:
- Is directed at children under 13, OR
- Knowingly collects personal information from children under 13

### Requirements

1. **Parental Consent**: Get verifiable parental consent before collecting data
2. **Privacy Policy**: Post clear, prominent privacy policy
3. **Data Minimization**: Collect only necessary data
4. **Security**: Maintain reasonable security for children's data
5. **Deletion**: Delete data when no longer needed

### Amail's Approach

Amail is **NOT directed at children** and should not be used to collect data from children under 13 without parental consent.

```env
# COPPA Configuration
COPPA_DIRECTED_AT_CHILDREN=false
COPPA_MINIMUM_AGE=13
COPPA_PRIVACY_POLICY_URL=https://yourdomain.com/privacy
```

---

## GDPR

### EU General Data Protection Regulation

Applies if you:
- Are established in the EU, OR
- Offer goods/services to EU residents, OR
- Monitor behavior of EU residents

### Key Rights

| Right | Description |
|-------|-------------|
| **Access** | Users can request their data |
| **Rectification** | Users can correct inaccurate data |
| **Erasure** | "Right to be forgotten" |
| **Portability** | Export data in machine-readable format |
| **Object** | Object to certain processing |

### Requirements

1. **Lawful Basis**: Have legal basis for processing
2. **Consent**: Clear, affirmative consent for marketing
3. **Breach Notification**: Report breaches within 72 hours
4. **DPO**: Appoint Data Protection Officer if required
5. **Records**: Maintain processing records

### How Amail Helps

```rust
use amail_core::compliance::{GdprCompliance, DataSubjectRequest};

// Handle data subject requests
match request {
    DataSubjectRequest::Access { subject_email } => {
        // Export user's data
    }
    DataSubjectRequest::Erasure { subject_email, reason } => {
        // Delete user's data (if no legal hold)
    }
}
```

### Configuration

```env
# GDPR Compliance
GDPR_DPO_EMAIL=dpo@yourdomain.com
GDPR_SUPERVISORY_AUTHORITY=your_local_authority
GDPR_AUTO_DSR_ENABLED=true
```

### Data Breach Response

1. **Detect**: Identify the breach
2. **Contain**: Stop ongoing breach
3. **Assess**: Determine scope and risk
4. **Notify Authority**: Within 72 hours if risk to rights
5. **Notify Individuals**: If high risk to rights
6. **Document**: Record all breaches

---

## CCPA

### California Consumer Privacy Act

Applies if you:
- Have gross revenue >$25M, OR
- Buy/sell data of 100,000+ consumers, OR
- Derive 50%+ revenue from selling data

### Consumer Rights

- Right to know what data is collected
- Right to delete personal information
- Right to opt-out of data sales
- Right to non-discrimination

### Requirements

- "Do Not Sell My Personal Information" link
- Privacy policy updates
- Respond to requests within 45 days

---

## Data Retention

### Legal Requirements by Category

| Category | Retention Period | Reason |
|----------|-----------------|--------|
| Tax Records | 7 years | IRS requirements |
| Employment | 7 years | EEOC requirements |
| HIPAA | 6 years | Healthcare regulations |
| Financial | 5-7 years | SEC/FINRA requirements |
| CSAM Evidence | 1 year minimum | 18 USC 2258A |
| Audit Logs | 6 years | SOX compliance |

### Amail Configuration

```env
# Data Retention
RETENTION_DEFAULT_DAYS=365
RETENTION_TRANSACTIONAL_DAYS=2555  # 7 years
RETENTION_MARKETING_DAYS=365
RETENTION_LEGAL_DAYS=3650  # 10 years
RETENTION_AUTO_PURGE_ENABLED=false
```

### Legal Holds

When litigation is anticipated, you must preserve relevant data:

```rust
use amail_core::compliance::LegalHold;

let hold = LegalHold {
    id: Uuid::new_v4(),
    reason: "Pending litigation - Case #12345".to_string(),
    created_at: Utc::now(),
    created_by: "legal@company.com".to_string(),
    scope: vec!["user@example.com".to_string()],
    expires_at: None, // Indefinite until released
};
```

---

## Configuration

### Complete Compliance Configuration

Add to your `.env` file:

```env
# =============================================================================
# Legal Compliance Configuration
# =============================================================================

# Jurisdiction (affects which laws apply)
# Options: us, eu, uk, ca, au, multiple
COMPLIANCE_JURISDICTION=us

# Enable strict compliance mode
COMPLIANCE_MODE_ENABLED=true

# =============================================================================
# CSAM Reporting (18 USC 2258A)
# =============================================================================
COMPLIANCE_CSAM_ENABLED=true
COMPLIANCE_NCMEC_PROVIDER_ID=
COMPLIANCE_LAW_ENFORCEMENT_CONTACT=legal@yourdomain.com
COMPLIANCE_EVIDENCE_RETENTION_DAYS=365

# =============================================================================
# DMCA (17 USC 512)
# =============================================================================
DMCA_AGENT_NAME=Your Name
DMCA_AGENT_EMAIL=dmca@yourdomain.com
DMCA_AGENT_ADDRESS=123 Main St, City, State 12345
DMCA_REGISTRATION_NUMBER=
DMCA_POLICY_URL=https://yourdomain.com/dmca
DMCA_COUNTER_NOTICE_WAIT_DAYS=14
DMCA_REPEAT_INFRINGER_STRIKES=3

# =============================================================================
# CAN-SPAM
# =============================================================================
CANSPAM_ORG_NAME=Your Organization
CANSPAM_PHYSICAL_ADDRESS=123 Main Street, City, State 12345
CANSPAM_UNSUBSCRIBE_URL=https://yourdomain.com/unsubscribe?id={id}

# =============================================================================
# COPPA
# =============================================================================
COPPA_DIRECTED_AT_CHILDREN=false
COPPA_MINIMUM_AGE=13
COPPA_PRIVACY_POLICY_URL=https://yourdomain.com/privacy

# =============================================================================
# GDPR
# =============================================================================
GDPR_DPO_EMAIL=dpo@yourdomain.com
GDPR_SUPERVISORY_AUTHORITY=
GDPR_AUTO_DSR_ENABLED=true

# =============================================================================
# Data Retention
# =============================================================================
RETENTION_DEFAULT_DAYS=365
RETENTION_AUTO_PURGE_ENABLED=false
```

---

## Audit Logging

Amail logs all compliance-related events:

```rust
use amail_core::compliance::{ComplianceAuditLog, ComplianceEventType};

let log = ComplianceAuditLog::new(
    ComplianceEventType::DmcaTakedownReceived,
    "system",
    "Received DMCA takedown notice",
)
.with_subject("user@example.com")
.with_detail("notice_id", "12345")
.with_ip("192.168.1.1");
```

### Event Types Logged

- CSAM detection and reporting
- DMCA takedowns and counter-notices
- Data subject requests (access, deletion)
- Data breaches
- Consent records
- Policy changes

---

## Checklist

### Before Going Live

- [ ] Register DMCA agent with Copyright Office
- [ ] Publish DMCA policy on website
- [ ] Publish privacy policy
- [ ] Configure CAN-SPAM physical address
- [ ] Set up unsubscribe mechanism
- [ ] Configure data retention periods
- [ ] Set up audit logging
- [ ] Train staff on CSAM reporting
- [ ] Document data processing activities (GDPR)
- [ ] Appoint DPO if required (GDPR)

### Ongoing

- [ ] Monitor for CSAM (report immediately if found)
- [ ] Process DMCA notices expeditiously
- [ ] Honor opt-out requests within 10 days
- [ ] Respond to DSRs within required timeframes
- [ ] Review and purge data per retention policy
- [ ] Update policies as laws change
- [ ] Conduct periodic compliance audits

---

## Resources

- **NCMEC CyberTipline**: https://report.cybertipline.org/
- **Copyright Office DMCA**: https://www.copyright.gov/dmca-directory/
- **FTC CAN-SPAM Guide**: https://www.ftc.gov/business-guidance/resources/can-spam-act-compliance-guide-business
- **FTC COPPA**: https://www.ftc.gov/legal-library/browse/rules/childrens-online-privacy-protection-rule-coppa
- **GDPR Official Text**: https://gdpr-info.eu/
- **CCPA Official Text**: https://oag.ca.gov/privacy/ccpa

---

**Remember**: Compliance is not optional. Violations can result in significant fines and legal liability. When in doubt, consult with a qualified attorney.
