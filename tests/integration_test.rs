// =============================================================================
// Integration Tests - Email Processing Pipeline
// =============================================================================

use amail_core::{
    email::{EmailAddress, EmailMessage, EmailMetadata, EmailPriority},
    task::TaskPriority,
};
use amail_agents::agents::receptionist::{EmailCategory, ReceptionistAgent};
use chrono::Utc;
use uuid::Uuid;

/// Create a test email
fn create_test_email(subject: &str, body: &str, from: &str) -> EmailMessage {
    EmailMessage {
        metadata: EmailMetadata {
            message_id: format!("test-{}", Uuid::new_v4()),
            tracking_id: Uuid::new_v4(),
            from: EmailAddress::new(from),
            to: vec![EmailAddress::new("agent@example.com")],
            cc: Vec::new(),
            bcc: Vec::new(),
            reply_to: None,
            subject: subject.to_string(),
            received_at: Utc::now(),
            sent_at: None,
            in_reply_to: None,
            references: Vec::new(),
            spf_result: None,
            dkim_result: None,
            dmarc_result: None,
            priority: EmailPriority::Normal,
            custom_headers: std::collections::HashMap::new(),
        },
        text_body: Some(body.to_string()),
        html_body: None,
        attachments: Vec::new(),
        raw: None,
    }
}

#[test]
fn test_legal_email_classification() {
    let email = create_test_email(
        "Contract Review Request",
        "Please review the attached contract for our new vendor agreement.",
        "legal@company.com",
    );

    let classification = ReceptionistAgent::classify_email(&email);

    assert_eq!(classification.category, EmailCategory::Legal);
    assert_eq!(classification.priority, TaskPriority::High);
    assert!(classification.requires_human_review);
}

#[test]
fn test_financial_email_classification() {
    let email = create_test_email(
        "Invoice #12345 - Payment Due",
        "Please find attached the invoice for services rendered.",
        "billing@vendor.com",
    );

    let classification = ReceptionistAgent::classify_email(&email);

    assert_eq!(classification.category, EmailCategory::Financial);
}

#[test]
fn test_urgent_email_priority() {
    let email = create_test_email(
        "URGENT: Server Down",
        "The production server is not responding.",
        "ops@company.com",
    );

    let classification = ReceptionistAgent::classify_email(&email);

    assert_eq!(classification.priority, TaskPriority::Urgent);
    assert!(classification.requires_human_review);
}

#[test]
fn test_support_email_classification() {
    let email = create_test_email(
        "Help with account access",
        "I'm having trouble logging into my account.",
        "customer@example.com",
    );

    let classification = ReceptionistAgent::classify_email(&email);

    assert_eq!(classification.category, EmailCategory::Support);
}

#[test]
fn test_scheduling_email_classification() {
    let email = create_test_email(
        "Meeting Request: Q4 Planning",
        "Can we schedule a meeting to discuss Q4 goals?",
        "manager@company.com",
    );

    let classification = ReceptionistAgent::classify_email(&email);

    assert_eq!(classification.category, EmailCategory::Scheduling);
}

#[test]
fn test_hr_email_classification() {
    let email = create_test_email(
        "New Employee Onboarding",
        "We have a new hire starting next week.",
        "hr@company.com",
    );

    let classification = ReceptionistAgent::classify_email(&email);

    assert_eq!(classification.category, EmailCategory::HumanResources);
}

#[test]
fn test_security_email_classification() {
    let email = create_test_email(
        "Security Alert: Suspicious Access",
        "We detected unusual login activity on your account.",
        "security@company.com",
    );

    let classification = ReceptionistAgent::classify_email(&email);

    assert_eq!(classification.category, EmailCategory::Security);
    assert_eq!(classification.priority, TaskPriority::High);
}

#[test]
fn test_general_email_classification() {
    let email = create_test_email(
        "Hello",
        "Just wanted to say hi!",
        "friend@example.com",
    );

    let classification = ReceptionistAgent::classify_email(&email);

    assert_eq!(classification.category, EmailCategory::General);
    assert_eq!(classification.priority, TaskPriority::Normal);
}

#[test]
fn test_task_creation_from_email() {
    let email = create_test_email(
        "Contract Review Request",
        "Please review the attached contract.",
        "legal@company.com",
    );

    let classification = ReceptionistAgent::classify_email(&email);
    let task = ReceptionistAgent::create_task(email, &classification);

    assert!(task.description.contains("Legal"));
    assert!(task.description.contains("Contract Review Request"));
    assert_eq!(task.priority, TaskPriority::High);
}
