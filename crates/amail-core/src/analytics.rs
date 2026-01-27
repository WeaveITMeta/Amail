//! Analytics & KPI Tracking - Metrics collection and reporting
//!
//! Table of Contents:
//! 1. KpiTracker - Collects and stores metrics
//! 2. MetricsPeriod - Time periods for aggregation
//! 3. EmailMetrics - Email-related KPIs
//! 4. AgentMetrics - Agent performance KPIs
//! 5. SummaryReport - Generated summary reports

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc, Duration, Datelike, Timelike};

/// Time period for metrics aggregation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MetricsPeriod {
    Today,
    Yesterday,
    ThisWeek,
    LastWeek,
    ThisMonth,
    LastMonth,
    ThisYear,
    LastYear,
    AllTime,
}

impl MetricsPeriod {
    pub fn label(&self) -> &'static str {
        match self {
            MetricsPeriod::Today => "Today",
            MetricsPeriod::Yesterday => "Yesterday",
            MetricsPeriod::ThisWeek => "This Week",
            MetricsPeriod::LastWeek => "Last Week",
            MetricsPeriod::ThisMonth => "This Month",
            MetricsPeriod::LastMonth => "Last Month",
            MetricsPeriod::ThisYear => "This Year",
            MetricsPeriod::LastYear => "Last Year",
            MetricsPeriod::AllTime => "All Time",
        }
    }
}

/// Email-related metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EmailMetrics {
    pub received: u64,
    pub sent: u64,
    pub ai_responses: u64,
    pub human_responses: u64,
    pub pending_review: u64,
    pub avg_response_time_secs: f64,
    pub by_domain: HashMap<String, u64>,
    pub by_hour: [u64; 24],
}

/// Agent performance metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AgentMetrics {
    pub tasks_completed: u64,
    pub tasks_delegated: u64,
    pub avg_confidence: f64,
    pub high_confidence_count: u64,  // >= 75%
    pub medium_confidence_count: u64, // 50-75%
    pub low_confidence_count: u64,   // < 50%
    pub by_agent: HashMap<String, AgentStats>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AgentStats {
    pub name: String,
    pub tasks_completed: u64,
    pub avg_confidence: f64,
    pub avg_response_time_secs: f64,
}

/// Review metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReviewMetrics {
    pub total_reviews: u64,
    pub approved: u64,
    pub rejected: u64,
    pub edited: u64,
    pub expired: u64,
    pub avg_review_time_secs: f64,
    pub approval_rate: f64,
}

/// Complete metrics snapshot for a period
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MetricsSnapshot {
    pub period: String,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub email: EmailMetrics,
    pub agent: AgentMetrics,
    pub review: ReviewMetrics,
}

/// Summary report for delivery (email/Telegram)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummaryReport {
    pub period: MetricsPeriod,
    pub generated_at: DateTime<Utc>,
    pub metrics: MetricsSnapshot,
    pub highlights: Vec<String>,
    pub trends: Vec<TrendItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendItem {
    pub metric: String,
    pub current: f64,
    pub previous: f64,
    pub change_percent: f64,
    pub trend: Trend,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum Trend {
    Up,
    Down,
    Stable,
}

/// KPI Tracker - Collects and aggregates metrics
#[derive(Clone)]
pub struct KpiTracker {
    /// Raw event log for aggregation
    events: Arc<RwLock<Vec<MetricEvent>>>,
    /// Cached snapshots by period
    snapshots: Arc<RwLock<HashMap<MetricsPeriod, MetricsSnapshot>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricEvent {
    pub timestamp: DateTime<Utc>,
    pub event_type: MetricEventType,
    pub domain: Option<String>,
    pub agent: Option<String>,
    pub value: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetricEventType {
    EmailReceived,
    EmailSent,
    AiResponse { confidence: u8 },
    HumanResponse,
    ReviewCreated,
    ReviewApproved,
    ReviewRejected,
    ReviewEdited,
    ReviewExpired,
    TaskCompleted { agent: String },
    ResponseTime { seconds: f64 },
}

impl KpiTracker {
    pub fn new() -> Self {
        Self {
            events: Arc::new(RwLock::new(Vec::new())),
            snapshots: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Record a metric event
    pub async fn record(&self, event_type: MetricEventType, domain: Option<String>, agent: Option<String>) {
        let event = MetricEvent {
            timestamp: Utc::now(),
            event_type,
            domain,
            agent,
            value: 1.0,
        };
        self.events.write().await.push(event);
    }

    /// Record email received
    pub async fn record_email_received(&self, domain: &str) {
        self.record(MetricEventType::EmailReceived, Some(domain.to_string()), None).await;
    }

    /// Record email sent
    pub async fn record_email_sent(&self, domain: &str) {
        self.record(MetricEventType::EmailSent, Some(domain.to_string()), None).await;
    }

    /// Record AI response with confidence
    pub async fn record_ai_response(&self, domain: &str, agent: &str, confidence: u8) {
        self.record(
            MetricEventType::AiResponse { confidence },
            Some(domain.to_string()),
            Some(agent.to_string()),
        ).await;
    }

    /// Record review outcome
    pub async fn record_review(&self, outcome: &str) {
        let event_type = match outcome {
            "approved" => MetricEventType::ReviewApproved,
            "rejected" => MetricEventType::ReviewRejected,
            "edited" => MetricEventType::ReviewEdited,
            "expired" => MetricEventType::ReviewExpired,
            _ => MetricEventType::ReviewCreated,
        };
        self.record(event_type, None, None).await;
    }

    /// Get metrics for a specific period
    pub async fn get_metrics(&self, period: MetricsPeriod) -> MetricsSnapshot {
        let events = self.events.read().await;
        let now = Utc::now();
        
        let (start, end) = self.period_range(period, now);
        
        let filtered: Vec<_> = events.iter()
            .filter(|e| e.timestamp >= start && e.timestamp < end)
            .collect();

        let mut snapshot = MetricsSnapshot {
            period: period.label().to_string(),
            start_time: Some(start),
            end_time: Some(end),
            ..Default::default()
        };

        // Aggregate email metrics
        for event in &filtered {
            match &event.event_type {
                MetricEventType::EmailReceived => {
                    snapshot.email.received += 1;
                    if let Some(domain) = &event.domain {
                        *snapshot.email.by_domain.entry(domain.clone()).or_insert(0) += 1;
                    }
                    let hour = event.timestamp.hour() as usize;
                    snapshot.email.by_hour[hour] += 1;
                }
                MetricEventType::EmailSent => {
                    snapshot.email.sent += 1;
                }
                MetricEventType::AiResponse { confidence } => {
                    snapshot.email.ai_responses += 1;
                    snapshot.agent.tasks_completed += 1;
                    
                    if *confidence >= 75 {
                        snapshot.agent.high_confidence_count += 1;
                    } else if *confidence >= 50 {
                        snapshot.agent.medium_confidence_count += 1;
                    } else {
                        snapshot.agent.low_confidence_count += 1;
                    }
                    
                    if let Some(agent) = &event.agent {
                        let stats = snapshot.agent.by_agent
                            .entry(agent.clone())
                            .or_insert_with(|| AgentStats { name: agent.clone(), ..Default::default() });
                        stats.tasks_completed += 1;
                    }
                }
                MetricEventType::HumanResponse => {
                    snapshot.email.human_responses += 1;
                }
                MetricEventType::ReviewCreated => {
                    snapshot.review.total_reviews += 1;
                }
                MetricEventType::ReviewApproved => {
                    snapshot.review.approved += 1;
                }
                MetricEventType::ReviewRejected => {
                    snapshot.review.rejected += 1;
                }
                MetricEventType::ReviewEdited => {
                    snapshot.review.edited += 1;
                }
                MetricEventType::ReviewExpired => {
                    snapshot.review.expired += 1;
                }
                _ => {}
            }
        }

        // Calculate rates
        let total_responses = snapshot.email.ai_responses + snapshot.email.human_responses;
        if total_responses > 0 {
            snapshot.agent.avg_confidence = 
                (snapshot.agent.high_confidence_count as f64 * 87.5 +
                 snapshot.agent.medium_confidence_count as f64 * 62.5 +
                 snapshot.agent.low_confidence_count as f64 * 25.0) / 
                snapshot.agent.tasks_completed.max(1) as f64;
        }

        let total_reviews = snapshot.review.approved + snapshot.review.rejected + 
                           snapshot.review.edited + snapshot.review.expired;
        if total_reviews > 0 {
            snapshot.review.approval_rate = 
                (snapshot.review.approved + snapshot.review.edited) as f64 / total_reviews as f64 * 100.0;
        }

        snapshot
    }

    /// Generate a summary report
    pub async fn generate_report(&self, period: MetricsPeriod) -> SummaryReport {
        let metrics = self.get_metrics(period).await;
        let previous_period = self.previous_period(period);
        let previous_metrics = self.get_metrics(previous_period).await;

        let mut highlights = Vec::new();
        let mut trends = Vec::new();

        // Generate highlights
        if metrics.email.received > 0 {
            highlights.push(format!("📬 {} emails received", metrics.email.received));
        }
        if metrics.email.ai_responses > 0 {
            highlights.push(format!("🤖 {} AI responses sent", metrics.email.ai_responses));
        }
        if metrics.review.approved > 0 {
            highlights.push(format!("✅ {} reviews approved", metrics.review.approved));
        }
        if metrics.agent.avg_confidence > 0.0 {
            highlights.push(format!("📊 {:.1}% average confidence", metrics.agent.avg_confidence));
        }

        // Calculate trends
        if previous_metrics.email.received > 0 {
            let change = ((metrics.email.received as f64 - previous_metrics.email.received as f64) 
                / previous_metrics.email.received as f64) * 100.0;
            trends.push(TrendItem {
                metric: "Emails Received".to_string(),
                current: metrics.email.received as f64,
                previous: previous_metrics.email.received as f64,
                change_percent: change,
                trend: if change > 5.0 { Trend::Up } else if change < -5.0 { Trend::Down } else { Trend::Stable },
            });
        }

        if previous_metrics.email.ai_responses > 0 {
            let change = ((metrics.email.ai_responses as f64 - previous_metrics.email.ai_responses as f64) 
                / previous_metrics.email.ai_responses as f64) * 100.0;
            trends.push(TrendItem {
                metric: "AI Responses".to_string(),
                current: metrics.email.ai_responses as f64,
                previous: previous_metrics.email.ai_responses as f64,
                change_percent: change,
                trend: if change > 5.0 { Trend::Up } else if change < -5.0 { Trend::Down } else { Trend::Stable },
            });
        }

        SummaryReport {
            period,
            generated_at: Utc::now(),
            metrics,
            highlights,
            trends,
        }
    }

    /// Get time range for a period
    fn period_range(&self, period: MetricsPeriod, now: DateTime<Utc>) -> (DateTime<Utc>, DateTime<Utc>) {
        let today_start = now.date_naive().and_hms_opt(0, 0, 0).unwrap();
        let today_start = DateTime::<Utc>::from_naive_utc_and_offset(today_start, Utc);
        
        match period {
            MetricsPeriod::Today => (today_start, now),
            MetricsPeriod::Yesterday => (today_start - Duration::days(1), today_start),
            MetricsPeriod::ThisWeek => {
                let days_since_monday = now.weekday().num_days_from_monday() as i64;
                let week_start = today_start - Duration::days(days_since_monday);
                (week_start, now)
            }
            MetricsPeriod::LastWeek => {
                let days_since_monday = now.weekday().num_days_from_monday() as i64;
                let this_week_start = today_start - Duration::days(days_since_monday);
                let last_week_start = this_week_start - Duration::days(7);
                (last_week_start, this_week_start)
            }
            MetricsPeriod::ThisMonth => {
                let month_start = now.date_naive().with_day(1).unwrap().and_hms_opt(0, 0, 0).unwrap();
                let month_start = DateTime::<Utc>::from_naive_utc_and_offset(month_start, Utc);
                (month_start, now)
            }
            MetricsPeriod::LastMonth => {
                let this_month_start = now.date_naive().with_day(1).unwrap().and_hms_opt(0, 0, 0).unwrap();
                let this_month_start = DateTime::<Utc>::from_naive_utc_and_offset(this_month_start, Utc);
                let last_month_start = (this_month_start - Duration::days(1)).date_naive().with_day(1).unwrap().and_hms_opt(0, 0, 0).unwrap();
                let last_month_start = DateTime::<Utc>::from_naive_utc_and_offset(last_month_start, Utc);
                (last_month_start, this_month_start)
            }
            MetricsPeriod::ThisYear => {
                let year_start = now.date_naive().with_month(1).unwrap().with_day(1).unwrap().and_hms_opt(0, 0, 0).unwrap();
                let year_start = DateTime::<Utc>::from_naive_utc_and_offset(year_start, Utc);
                (year_start, now)
            }
            MetricsPeriod::LastYear => {
                let this_year_start = now.date_naive().with_month(1).unwrap().with_day(1).unwrap().and_hms_opt(0, 0, 0).unwrap();
                let this_year_start = DateTime::<Utc>::from_naive_utc_and_offset(this_year_start, Utc);
                let last_year_start = (this_year_start - Duration::days(1)).date_naive().with_month(1).unwrap().with_day(1).unwrap().and_hms_opt(0, 0, 0).unwrap();
                let last_year_start = DateTime::<Utc>::from_naive_utc_and_offset(last_year_start, Utc);
                (last_year_start, this_year_start)
            }
            MetricsPeriod::AllTime => {
                let epoch = DateTime::<Utc>::from_naive_utc_and_offset(
                    chrono::NaiveDateTime::from_timestamp_opt(0, 0).unwrap(),
                    Utc
                );
                (epoch, now)
            }
        }
    }

    /// Get the previous period for comparison
    fn previous_period(&self, period: MetricsPeriod) -> MetricsPeriod {
        match period {
            MetricsPeriod::Today => MetricsPeriod::Yesterday,
            MetricsPeriod::ThisWeek => MetricsPeriod::LastWeek,
            MetricsPeriod::ThisMonth => MetricsPeriod::LastMonth,
            MetricsPeriod::ThisYear => MetricsPeriod::LastYear,
            _ => MetricsPeriod::AllTime,
        }
    }

    /// Format report for Telegram
    pub fn format_telegram_report(&self, report: &SummaryReport) -> String {
        let mut text = format!("📊 *{} Summary*\n\n", report.period.label());
        
        for highlight in &report.highlights {
            text.push_str(&format!("{}\n", highlight));
        }
        
        if !report.trends.is_empty() {
            text.push_str("\n*Trends:*\n");
            for trend in &report.trends {
                let arrow = match trend.trend {
                    Trend::Up => "📈",
                    Trend::Down => "📉",
                    Trend::Stable => "➡️",
                };
                text.push_str(&format!("{} {}: {:.0} ({:+.1}%)\n", 
                    arrow, trend.metric, trend.current, trend.change_percent));
            }
        }
        
        text
    }
}

impl Default for KpiTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_record_and_get_metrics() {
        let tracker = KpiTracker::new();
        
        tracker.record_email_received("example.com").await;
        tracker.record_email_received("example.com").await;
        tracker.record_ai_response("example.com", "Receptionist", 85).await;
        
        let metrics = tracker.get_metrics(MetricsPeriod::Today).await;
        
        assert_eq!(metrics.email.received, 2);
        assert_eq!(metrics.email.ai_responses, 1);
        assert_eq!(metrics.agent.high_confidence_count, 1);
    }
}
