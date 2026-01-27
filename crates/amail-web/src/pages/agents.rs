//! Agents Page - Agent monitoring and management

use leptos::*;
use crate::state::{DashboardState, AgentState};

#[component]
pub fn AgentsPage() -> impl IntoView {
    let state = use_context::<DashboardState>().expect("DashboardState not provided");
    let agents = state.agents;

    view! {
        <div class="page agents-page">
            <div class="page-header">
                <h1 class="page-title">"Agents"</h1>
                <div class="page-actions">
                    <button class="btn btn-secondary">
                        <svg viewBox="0 0 24 24" fill="currentColor" class="btn-icon">
                            <path d="M17.65 6.35C16.2 4.9 14.21 4 12 4c-4.42 0-7.99 3.58-7.99 8s3.57 8 7.99 8c3.73 0 6.84-2.55 7.73-6h-2.08c-.82 2.33-3.04 4-5.65 4-3.31 0-6-2.69-6-6s2.69-6 6-6c1.66 0 3.14.69 4.22 1.78L13 11h7V4l-2.35 2.35z"/>
                        </svg>
                        "Refresh"
                    </button>
                </div>
            </div>
            
            {move || {
                let agent_list = agents.get();
                if agent_list.is_empty() {
                    view! {
                        <div class="empty-state">
                            <div class="empty-state-icon">
                                <svg viewBox="0 0 24 24" fill="currentColor">
                                    <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm0 3c1.66 0 3 1.34 3 3s-1.34 3-3 3-3-1.34-3-3 1.34-3 3-3zm0 14.2c-2.5 0-4.71-1.28-6-3.22.03-1.99 4-3.08 6-3.08 1.99 0 5.97 1.09 6 3.08-1.29 1.94-3.5 3.22-6 3.22z"/>
                                </svg>
                            </div>
                            <h3 class="empty-state-title">"No Agents Running"</h3>
                            <p class="empty-state-description">
                                "AI agents will appear here once the server is started and connected. "
                                "Agents handle email triage, responses, scheduling, and more."
                            </p>
                            <div class="empty-state-hints">
                                <div class="hint-item">
                                    <svg viewBox="0 0 24 24" fill="currentColor" class="hint-icon">
                                        <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-6h2v6zm0-8h-2V7h2v2z"/>
                                    </svg>
                                    <span>"Make sure the Amail server is running"</span>
                                </div>
                                <div class="hint-item">
                                    <svg viewBox="0 0 24 24" fill="currentColor" class="hint-icon">
                                        <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-6h2v6zm0-8h-2V7h2v2z"/>
                                    </svg>
                                    <span>"Check the WebSocket connection status above"</span>
                                </div>
                                <div class="hint-item">
                                    <svg viewBox="0 0 24 24" fill="currentColor" class="hint-icon">
                                        <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-6h2v6zm0-8h-2V7h2v2z"/>
                                    </svg>
                                    <span>"Agents start automatically at 8am or on first email"</span>
                                </div>
                            </div>
                        </div>
                    }.into_view()
                } else {
                    view! {
                        <div class="agents-grid">
                            <For
                                each=move || agents.get().into_iter()
                                key=|(id, _)| id.clone()
                                children=move |(id, agent)| {
                                    view! { <AgentDetailCard agent=agent/> }
                                }
                            />
                        </div>
                    }.into_view()
                }
            }}
        </div>
    }
}

#[component]
fn AgentDetailCard(agent: AgentState) -> impl IntoView {
    let status_class = match agent.status.as_str() {
        "idle" => "status-idle",
        "busy" => "status-busy", 
        "error" => "status-error",
        _ => "status-unknown",
    };

    view! {
        <div class="agent-detail-card">
            <div class="agent-card-header">
                <div class=format!("agent-avatar {}", status_class)>
                    {agent.name.chars().next().unwrap_or('?')}
                </div>
                <div class="agent-card-info">
                    <h3 class="agent-card-name">{&agent.name}</h3>
                    <span class="agent-card-role">{&agent.role}</span>
                </div>
                <div class=format!("agent-status-badge {}", status_class)>
                    {agent.status.to_string()}
                </div>
            </div>
            
            <div class="agent-card-stats">
                <div class="agent-stat">
                    <span class="agent-stat-value">{agent.completed_tasks}</span>
                    <span class="agent-stat-label">"Completed"</span>
                </div>
                <div class="agent-stat">
                    <span class="agent-stat-value">{agent.failed_tasks}</span>
                    <span class="agent-stat-label">"Failed"</span>
                </div>
                <div class="agent-stat">
                    <span class="agent-stat-value">{agent.active_tasks}</span>
                    <span class="agent-stat-label">"Active"</span>
                </div>
            </div>
            
            <div class="agent-card-footer">
                <span class="agent-id">"ID: "{agent.id.chars().take(8).collect::<String>()}"..."</span>
                <button class="btn btn-sm btn-ghost">"View Logs"</button>
            </div>
        </div>
    }
}
