//! Analytics Page - KPI Dashboard with charts and summaries
//!
//! Table of Contents:
//! 1. AnalyticsPage - Main analytics dashboard
//! 2. PeriodSelector - Time period dropdown
//! 3. KpiCard - Individual metric card
//! 4. TrendIndicator - Up/down/stable indicator
//! 5. ActivityChart - Hourly activity visualization

use leptos::*;
use wasm_bindgen_futures::spawn_local;
use crate::api;

#[derive(Clone, Debug, PartialEq, Default)]
pub struct AnalyticsData {
    pub period: String,
    pub emails_received: u64,
    pub emails_sent: u64,
    pub ai_responses: u64,
    pub human_responses: u64,
    pub avg_confidence: f64,
    pub approval_rate: f64,
    pub reviews_pending: u64,
    pub reviews_approved: u64,
    pub reviews_rejected: u64,
    pub hourly_activity: [u64; 24],
    pub top_agents: Vec<AgentStat>,
    pub domain_breakdown: Vec<DomainStat>,
    // Trend data calculated from comparing periods
    pub emails_received_trend: TrendData,
    pub emails_sent_trend: TrendData,
    pub ai_responses_trend: TrendData,
    pub human_responses_trend: TrendData,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AgentStat {
    pub name: String,
    pub tasks: u64,
    pub confidence: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DomainStat {
    pub domain: String,
    pub emails: u64,
    pub percentage: f64,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct TrendData {
    pub value: f64,
    pub change: f64,
    pub trend: String, // "up", "down", "neutral"
}

impl TrendData {
    pub fn new(current: f64, previous: f64) -> Self {
        let change = if previous > 0.0 {
            ((current - previous) / previous) * 100.0
        } else if current > 0.0 {
            100.0 // New data, 100% increase from nothing
        } else {
            0.0 // Both zero
        };
        
        let trend = if change > 0.5 {
            "up".to_string()
        } else if change < -0.5 {
            "down".to_string()
        } else {
            "neutral".to_string()
        };
        
        Self { value: current, change, trend }
    }
    
    pub fn zero() -> Self {
        Self { value: 0.0, change: 0.0, trend: "neutral".to_string() }
    }
}

#[component]
pub fn AnalyticsPage() -> impl IntoView {
    let (period, set_period) = create_signal("today".to_string());
    let (data, set_data) = create_signal(AnalyticsData::default());
    let (loading, set_loading) = create_signal(true);
    
    // Fetch analytics data from API
    {
        let set_data = set_data.clone();
        let set_loading = set_loading.clone();
        
        spawn_local(async move {
            // Fetch daily metrics
            let mut analytics = AnalyticsData::default();
            
            match api::fetch_daily_metrics(Some("week")).await {
                Ok(metrics) => {
                    // Split metrics into current period (last 3 days) and previous period (earlier 4 days)
                    let mid = metrics.len() / 2;
                    let (previous, current) = if metrics.len() >= 2 {
                        (&metrics[..mid.max(1)], &metrics[mid.max(1)..])
                    } else {
                        (&metrics[..], &metrics[..])
                    };
                    
                    // Aggregate current period
                    let mut current_received: u64 = 0;
                    let mut current_sent: u64 = 0;
                    let mut current_ai: u64 = 0;
                    
                    for m in current {
                        current_received += m.emails_received as u64;
                        current_sent += m.emails_sent as u64;
                        current_ai += m.emails_auto_replied as u64;
                        analytics.reviews_approved += m.reviews_approved as u64;
                        analytics.reviews_rejected += m.reviews_rejected as u64;
                        analytics.reviews_pending += m.reviews_requested as u64;
                    }
                    
                    // Aggregate previous period for comparison
                    let mut prev_received: u64 = 0;
                    let mut prev_sent: u64 = 0;
                    let mut prev_ai: u64 = 0;
                    
                    for m in previous {
                        prev_received += m.emails_received as u64;
                        prev_sent += m.emails_sent as u64;
                        prev_ai += m.emails_auto_replied as u64;
                    }
                    
                    // Set totals
                    analytics.emails_received = current_received;
                    analytics.emails_sent = current_sent;
                    analytics.ai_responses = current_ai;
                    
                    // Calculate trends
                    analytics.emails_received_trend = TrendData::new(current_received as f64, prev_received as f64);
                    analytics.emails_sent_trend = TrendData::new(current_sent as f64, prev_sent as f64);
                    analytics.ai_responses_trend = TrendData::new(current_ai as f64, prev_ai as f64);
                    analytics.human_responses_trend = TrendData::zero(); // No human response tracking yet
                    
                    // Calculate approval rate
                    let total_reviews = analytics.reviews_approved + analytics.reviews_rejected;
                    if total_reviews > 0 {
                        analytics.approval_rate = (analytics.reviews_approved as f64 / total_reviews as f64) * 100.0;
                    }
                }
                Err(e) => {
                    log::warn!("Failed to fetch daily metrics: {}", e);
                }
            }
            
            // Fetch hourly metrics for activity chart
            match api::fetch_hourly_metrics(Some("today")).await {
                Ok(hourly) => {
                    for h in hourly {
                        // Parse hour from timestamp and update activity
                        if let Some(hour_num) = h.hour.split('T').nth(1).and_then(|t| t.split(':').next()).and_then(|h| h.parse::<usize>().ok()) {
                            if hour_num < 24 {
                                analytics.hourly_activity[hour_num] = h.emails_received as u64;
                            }
                        }
                    }
                }
                Err(e) => {
                    log::warn!("Failed to fetch hourly metrics: {}", e);
                }
            }
            
            // Fetch agents for top agents list
            match api::fetch_agents().await {
                Ok(agents) => {
                    analytics.top_agents = agents.into_iter().take(5).map(|a| AgentStat {
                        name: a.role,
                        tasks: a.tasks_processed as u64,
                        confidence: 75.0, // Default confidence
                    }).collect();
                }
                Err(e) => {
                    log::warn!("Failed to fetch agents: {}", e);
                }
            }
            
            // Fetch domains for breakdown
            match api::fetch_domains().await {
                Ok(domains) => {
                    let total: i64 = domains.iter().map(|d| d.mailbox_count).sum();
                    analytics.domain_breakdown = domains.into_iter().map(|d| {
                        let percentage = if total > 0 { (d.mailbox_count as f64 / total as f64) * 100.0 } else { 0.0 };
                        DomainStat {
                            domain: d.name,
                            emails: d.mailbox_count as u64,
                            percentage,
                        }
                    }).collect();
                }
                Err(e) => {
                    log::warn!("Failed to fetch domains: {}", e);
                }
            }
            
            set_data.set(analytics);
            set_loading.set(false);
        });
    }
    
    let period_label = move || match period.get().as_str() {
        "today" => "Today",
        "yesterday" => "Yesterday",
        "week" => "This Week",
        "month" => "This Month",
        "year" => "This Year",
        _ => "All Time",
    };

    view! {
        <div class="page analytics-page">
            <div class="analytics-header">
                <div class="analytics-title-section">
                    <h1 class="page-title">"Analytics"</h1>
                    <p class="page-subtitle">"AI mail server performance at a glance"</p>
                </div>
                
                // Period selector
                <div class="period-selector">
                    <select 
                        class="period-select"
                        on:change=move |e| set_period.set(event_target_value(&e))
                    >
                        <option value="today" selected>"Today"</option>
                        <option value="yesterday">"Yesterday"</option>
                        <option value="week">"This Week"</option>
                        <option value="month">"This Month"</option>
                        <option value="year">"This Year"</option>
                        <option value="all">"All Time"</option>
                    </select>
                </div>
            </div>
            
            // Summary highlights
            <div class="highlights-banner">
                <div class="highlight-item">
                    <span class="highlight-icon">
                        <svg viewBox="0 0 24 24" fill="currentColor"><path d="M20 4H4c-1.1 0-2 .9-2 2v12c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V6c0-1.1-.9-2-2-2zm0 4l-8 5-8-5V6l8 5 8-5v2z"/></svg>
                    </span>
                    <span class="highlight-value">{move || data.get().emails_received}</span>
                    <span class="highlight-label">"emails received"</span>
                </div>
                <div class="highlight-item">
                    <span class="highlight-icon">
                        <svg viewBox="0 0 24 24" fill="currentColor"><path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm0 3c1.66 0 3 1.34 3 3s-1.34 3-3 3-3-1.34-3-3 1.34-3 3-3zm0 14.2c-2.5 0-4.71-1.28-6-3.22.03-1.99 4-3.08 6-3.08 1.99 0 5.97 1.09 6 3.08-1.29 1.94-3.5 3.22-6 3.22z"/></svg>
                    </span>
                    <span class="highlight-value">{move || data.get().ai_responses}</span>
                    <span class="highlight-label">"AI responses"</span>
                </div>
                <div class="highlight-item">
                    <span class="highlight-icon">
                        <svg viewBox="0 0 24 24" fill="currentColor"><path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z"/></svg>
                    </span>
                    <span class="highlight-value">{move || format!("{:.0}%", data.get().approval_rate)}</span>
                    <span class="highlight-label">"approval rate"</span>
                </div>
                <div class="highlight-item">
                    <span class="highlight-icon">
                        <svg viewBox="0 0 24 24" fill="currentColor"><path d="M19 3H5c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2V5c0-1.1-.9-2-2-2zM9 17H7v-7h2v7zm4 0h-2V7h2v10zm4 0h-2v-4h2v4z"/></svg>
                    </span>
                    <span class="highlight-value">{move || format!("{:.1}%", data.get().avg_confidence)}</span>
                    <span class="highlight-label">"avg confidence"</span>
                </div>
            </div>
            
            // KPI Cards Grid
            <div class="kpi-grid">
                <KpiCard 
                    title="Emails Received"
                    value=move || data.get().emails_received.to_string()
                    icon="inbox"
                    trend=Signal::derive(move || data.get().emails_received_trend.clone())
                    color="blue"
                />
                <KpiCard 
                    title="Emails Sent"
                    value=move || data.get().emails_sent.to_string()
                    icon="send"
                    trend=Signal::derive(move || data.get().emails_sent_trend.clone())
                    color="green"
                />
                <KpiCard 
                    title="AI Responses"
                    value=move || data.get().ai_responses.to_string()
                    icon="robot"
                    trend=Signal::derive(move || data.get().ai_responses_trend.clone())
                    color="purple"
                />
                <KpiCard 
                    title="Human Responses"
                    value=move || data.get().human_responses.to_string()
                    icon="user"
                    trend=Signal::derive(move || data.get().human_responses_trend.clone())
                    color="orange"
                />
            </div>
            
            // Charts row
            <div class="charts-row">
                // Hourly activity
                <div class="chart-card">
                    <h3>"Hourly Activity"</h3>
                    <div class="hourly-chart">
                        {move || {
                            let activity = data.get().hourly_activity;
                            let max_val = activity.iter().max().copied().unwrap_or(1).max(1);
                            activity.iter().enumerate().map(|(hour, &count)| {
                                let height = (count as f64 / max_val as f64 * 100.0) as u32;
                                let is_peak = count == max_val && count > 0;
                                view! {
                                    <div class="hour-bar-container" title=format!("{}:00 - {} emails", hour, count)>
                                        <div 
                                            class=if is_peak { "hour-bar peak" } else { "hour-bar" }
                                            style=format!("height: {}%", height.max(2))
                                        ></div>
                                        <span class="hour-label">{if hour % 4 == 0 { format!("{}", hour) } else { String::new() }}</span>
                                    </div>
                                }
                            }).collect_view()
                        }}
                    </div>
                </div>
                
                // Review breakdown
                <div class="chart-card">
                    <h3>"Review Outcomes"</h3>
                    <div class="review-breakdown">
                        <div class="breakdown-item approved">
                            <div class="breakdown-bar" style=move || format!("width: {}%", 
                                if data.get().reviews_approved + data.get().reviews_rejected + data.get().reviews_pending > 0 {
                                    data.get().reviews_approved as f64 / (data.get().reviews_approved + data.get().reviews_rejected + data.get().reviews_pending) as f64 * 100.0
                                } else { 0.0 }
                            )></div>
                            <span class="breakdown-label">"Approved"</span>
                            <span class="breakdown-value">{move || data.get().reviews_approved}</span>
                        </div>
                        <div class="breakdown-item rejected">
                            <div class="breakdown-bar" style=move || format!("width: {}%",
                                if data.get().reviews_approved + data.get().reviews_rejected + data.get().reviews_pending > 0 {
                                    data.get().reviews_rejected as f64 / (data.get().reviews_approved + data.get().reviews_rejected + data.get().reviews_pending) as f64 * 100.0
                                } else { 0.0 }
                            )></div>
                            <span class="breakdown-label">"Rejected"</span>
                            <span class="breakdown-value">{move || data.get().reviews_rejected}</span>
                        </div>
                        <div class="breakdown-item pending">
                            <div class="breakdown-bar" style=move || format!("width: {}%",
                                if data.get().reviews_approved + data.get().reviews_rejected + data.get().reviews_pending > 0 {
                                    data.get().reviews_pending as f64 / (data.get().reviews_approved + data.get().reviews_rejected + data.get().reviews_pending) as f64 * 100.0
                                } else { 0.0 }
                            )></div>
                            <span class="breakdown-label">"Pending"</span>
                            <span class="breakdown-value">{move || data.get().reviews_pending}</span>
                        </div>
                    </div>
                </div>
            </div>
            
            // Bottom row - Agent performance & Domain breakdown
            <div class="analytics-bottom-row">
                // Top agents
                <div class="analytics-card">
                    <h3>"Top AI Agents"</h3>
                    <div class="agent-list">
                        {move || data.get().top_agents.iter().map(|agent| {
                            let confidence_class = if agent.confidence >= 75.0 { "high" } else if agent.confidence >= 50.0 { "medium" } else { "low" };
                            view! {
                                <div class="agent-row">
                                    <div class="agent-info">
                                        <span class="agent-name">{&agent.name}</span>
                                        <span class="agent-tasks">{agent.tasks}" tasks"</span>
                                    </div>
                                    <div class=format!("agent-confidence {}", confidence_class)>
                                        {format!("{:.0}%", agent.confidence)}
                                    </div>
                                </div>
                            }
                        }).collect_view()}
                    </div>
                </div>
                
                // Domain breakdown
                <div class="analytics-card">
                    <h3>"Emails by Domain"</h3>
                    <div class="domain-list">
                        {move || data.get().domain_breakdown.iter().map(|domain| {
                            view! {
                                <div class="domain-row">
                                    <div class="domain-info">
                                        <span class="domain-name">{&domain.domain}</span>
                                        <div class="domain-bar-bg">
                                            <div class="domain-bar" style=format!("width: {}%", domain.percentage)></div>
                                        </div>
                                    </div>
                                    <span class="domain-count">{domain.emails}</span>
                                </div>
                            }
                        }).collect_view()}
                    </div>
                </div>
            </div>
        </div>
    }
}

#[component]
fn KpiCard(
    title: &'static str,
    value: impl Fn() -> String + 'static,
    icon: &'static str,
    #[prop(into)] trend: Signal<TrendData>,
    color: &'static str,
) -> impl IntoView {
    let icon_svg = match icon {
        "inbox" => view! { <svg viewBox="0 0 24 24" fill="currentColor"><path d="M20 4H4c-1.1 0-2 .9-2 2v12c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V6c0-1.1-.9-2-2-2zm0 4l-8 5-8-5V6l8 5 8-5v2z"/></svg> }.into_view(),
        "send" => view! { <svg viewBox="0 0 24 24" fill="currentColor"><path d="M2.01 21L23 12 2.01 3 2 10l15 2-15 2z"/></svg> }.into_view(),
        "robot" => view! { <svg viewBox="0 0 24 24" fill="currentColor"><path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm0 3c1.66 0 3 1.34 3 3s-1.34 3-3 3-3-1.34-3-3 1.34-3 3-3zm0 14.2c-2.5 0-4.71-1.28-6-3.22.03-1.99 4-3.08 6-3.08 1.99 0 5.97 1.09 6 3.08-1.29 1.94-3.5 3.22-6 3.22z"/></svg> }.into_view(),
        "user" => view! { <svg viewBox="0 0 24 24" fill="currentColor"><path d="M12 12c2.21 0 4-1.79 4-4s-1.79-4-4-4-4 1.79-4 4 1.79 4 4 4zm0 2c-2.67 0-8 1.34-8 4v2h16v-2c0-2.66-5.33-4-8-4z"/></svg> }.into_view(),
        _ => view! { <svg viewBox="0 0 24 24" fill="currentColor"><path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2z"/></svg> }.into_view(),
    };
    
    view! {
        <div class=format!("kpi-card kpi-{}", color)>
            <div class="kpi-header">
                <span class="kpi-icon">{icon_svg}</span>
                {move || {
                    let t = trend.get();
                    let trend_class = match t.trend.as_str() {
                        "up" => "trend-up",
                        "down" => "trend-down",
                        _ => "trend-neutral",
                    };
                    let trend_icon = match t.trend.as_str() {
                        "up" => "↑",
                        "down" => "↓",
                        _ => "→",
                    };
                    view! {
                        <div class=format!("kpi-trend {}", trend_class)>
                            <span class="trend-icon">{trend_icon}</span>
                            <span class="trend-value">{format!("{:+.1}%", t.change)}</span>
                        </div>
                    }
                }}
            </div>
            <div class="kpi-value">{value}</div>
            <div class="kpi-title">{title}</div>
        </div>
    }
}

// Sample data removed - now using real API data
