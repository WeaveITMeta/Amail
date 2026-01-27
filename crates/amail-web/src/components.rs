// =============================================================================
// Dashboard UI Components
// =============================================================================
// Table of Contents:
// 1. Header component
// 2. Sidebar component
// 3. AgentsGrid component
// 4. AgentCard component
// 5. TaskList component
// 6. RightPanel component
// 7. Statistics components
// =============================================================================

use leptos::*;
use crate::state::{AgentState, AgentStatus, DashboardState, TaskPriority, TaskState, DashboardEvent};

// =============================================================================
// 1. Header Component
// =============================================================================

#[component]
pub fn Header(_connected: ReadSignal<bool>) -> impl IntoView {
    let state = use_context::<DashboardState>().expect("DashboardState not provided");
    let ws_connected = state.ws_connected;
    
    view! {
        <header class="header">
            <div class="logo">
                <svg class="logo-icon" viewBox="0 0 24 24" fill="currentColor">
                    <path d="M20 4H4c-1.1 0-2 .9-2 2v12c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V6c0-1.1-.9-2-2-2zm0 4l-8 5-8-5V6l8 5 8-5v2z"/>
                </svg>
                "Amail Dashboard"
            </div>
            
            <div class=move || {
                let base = "connection-status";
                if ws_connected.get() {
                    format!("{} connected", base)
                } else {
                    format!("{} disconnected", base)
                }
            }>
                <span class=move || {
                    if ws_connected.get() { "status-dot pulse" } else { "status-dot" }
                }></span>
                {move || if ws_connected.get() { "Connected" } else { "Disconnected" }}
            </div>
        </header>
    }
}

// =============================================================================
// 2. Sidebar Component
// =============================================================================

#[component]
pub fn Sidebar() -> impl IntoView {
    let state = use_context::<DashboardState>().expect("DashboardState not provided");
    let stats = state.stats;
    
    view! {
        <aside class="sidebar">
            <div class="stats-panel">
                <div class="stats-section">
                    <h3 class="stats-section-title">"System Overview"</h3>
                    <div class="big-stat">
                        <span class="big-stat-value">
                            {move || stats.get().total_agents}
                        </span>
                        <span class="big-stat-unit">"agents"</span>
                    </div>
                </div>
                
                <div class="stats-section">
                    <h3 class="stats-section-title">"Performance"</h3>
                    <div class="stat-row">
                        <span class="stat-row-label">"Tasks/min"</span>
                        <span class="stat-row-value">
                            {move || format!("{:.1}", stats.get().tasks_per_minute)}
                        </span>
                    </div>
                    <div class="stat-row">
                        <span class="stat-row-label">"Avg Duration"</span>
                        <span class="stat-row-value">
                            {move || format!("{}ms", stats.get().avg_task_duration_ms)}
                        </span>
                    </div>
                    <div class="stat-row">
                        <span class="stat-row-label">"Total Processed"</span>
                        <span class="stat-row-value">
                            {move || stats.get().total_tasks_processed}
                        </span>
                    </div>
                </div>
                
                <div class="stats-section">
                    <h3 class="stats-section-title">"Email Activity"</h3>
                    <div class="stat-row">
                        <span class="stat-row-label">"Received"</span>
                        <span class="stat-row-value">
                            {move || stats.get().emails_received}
                        </span>
                    </div>
                    <div class="stat-row">
                        <span class="stat-row-label">"Sent"</span>
                        <span class="stat-row-value">
                            {move || stats.get().emails_sent}
                        </span>
                    </div>
                </div>
                
                <div class="stats-section">
                    <h3 class="stats-section-title">"Integrations"</h3>
                    <div class="stat-row">
                        <span class="stat-row-label">"LLM Calls"</span>
                        <span class="stat-row-value">
                            {move || stats.get().llm_calls}
                        </span>
                    </div>
                    <div class="stat-row">
                        <span class="stat-row-label">"MCP Calls"</span>
                        <span class="stat-row-value">
                            {move || stats.get().mcp_calls}
                        </span>
                    </div>
                </div>
            </div>
        </aside>
    }
}

// =============================================================================
// 3. AgentsGrid Component
// =============================================================================

#[component]
pub fn AgentsGrid() -> impl IntoView {
    let state = use_context::<DashboardState>().expect("DashboardState not provided");
    let agents = state.agents;
    
    let agents_list = move || {
        let mut agents_vec: Vec<_> = agents.get().values().cloned().collect();
        agents_vec.sort_by(|a, b| a.name.cmp(&b.name));
        agents_vec
    };
    
    view! {
        <div class="agents-grid">
            <For
                each=agents_list
                key=|agent| agent.id.clone()
                children=move |agent: AgentState| {
                    view! { <AgentCard agent=agent/> }
                }
            />
            
            {move || {
                if agents.get().is_empty() {
                    Some(view! {
                        <div class="agent-card" style="opacity: 0.5; text-align: center;">
                            <p style="color: var(--text-secondary);">
                                "No agents connected"
                            </p>
                            <p style="color: var(--text-muted); font-size: 0.75rem;">
                                "Waiting for WebSocket connection..."
                            </p>
                        </div>
                    })
                } else {
                    None
                }
            }}
        </div>
    }
}

// =============================================================================
// 4. AgentCard Component
// =============================================================================

#[component]
pub fn AgentCard(agent: AgentState) -> impl IntoView {
    let status_class = match agent.status {
        AgentStatus::Idle => "agent-status idle",
        AgentStatus::Processing => "agent-status processing",
        AgentStatus::Error => "agent-status error",
        AgentStatus::Starting => "agent-status idle",
        AgentStatus::Stopping => "agent-status idle",
    };
    
    let agent_name = agent.name.clone();
    let agent_role = agent.role.clone();
    let status_display = agent.status.display();
    let current_task = agent.current_task.clone();
    
    view! {
        <div class="agent-card fade-in">
            <div class="agent-card-header">
                <div>
                    <div class="agent-name">{agent_name}</div>
                    <div class="agent-role">{agent_role}</div>
                </div>
                <span class=status_class>{status_display}</span>
            </div>
            
            {current_task.map(|task| {
                view! {
                    <div style="font-size: 0.75rem; color: var(--accent-blue); margin-bottom: var(--spacing-sm);">
                        "Working on: " {task}
                    </div>
                }
            })}
            
            <div class="agent-stats">
                <div class="stat">
                    <div class="stat-value">{agent.active_tasks}</div>
                    <div class="stat-label">"Active"</div>
                </div>
                <div class="stat">
                    <div class="stat-value">{agent.completed_tasks}</div>
                    <div class="stat-label">"Done"</div>
                </div>
                <div class="stat">
                    <div class="stat-value">{agent.failed_tasks}</div>
                    <div class="stat-label">"Failed"</div>
                </div>
            </div>
        </div>
    }
}

// =============================================================================
// 5. TaskList Component
// =============================================================================

#[component]
pub fn TaskList() -> impl IntoView {
    let state = use_context::<DashboardState>().expect("DashboardState not provided");
    let tasks = state.tasks;
    
    let tasks_list = move || {
        tasks.get().into_iter().take(10).collect::<Vec<_>>()
    };
    
    view! {
        <div class="task-list">
            <div class="task-list-header">
                <span class="task-list-title">"Recent Tasks"</span>
                <span class="task-count">{move || tasks.get().len()}</span>
            </div>
            
            <For
                each=tasks_list
                key=|task| task.id.clone()
                children=move |task: TaskState| {
                    view! { <TaskItem task=task/> }
                }
            />
            
            {move || {
                if tasks.get().is_empty() {
                    Some(view! {
                        <div class="task-item" style="justify-content: center; color: var(--text-muted);">
                            "No tasks yet"
                        </div>
                    })
                } else {
                    None
                }
            }}
        </div>
    }
}

#[component]
fn TaskItem(task: TaskState) -> impl IntoView {
    let priority_class = match task.priority {
        TaskPriority::Critical => "task-priority critical",
        TaskPriority::High => "task-priority high",
        TaskPriority::Normal => "task-priority normal",
        TaskPriority::Low => "task-priority low",
    };
    
    let task_type = task.task_type.clone();
    let description = task.description.clone();
    let created_at = task.created_at.clone();
    
    view! {
        <div class="task-item fade-in">
            <div class=priority_class></div>
            <div class="task-info">
                <div class="task-type">{task_type}</div>
                <div class="task-description">{description}</div>
            </div>
            <div class="task-time">{created_at}</div>
        </div>
    }
}

// =============================================================================
// 6. RightPanel Component
// =============================================================================

#[component]
pub fn RightPanel() -> impl IntoView {
    let state = use_context::<DashboardState>().expect("DashboardState not provided");
    let events = state.events;
    
    let events_list = move || {
        events.get().into_iter().take(10).collect::<Vec<_>>()
    };
    
    view! {
        <aside class="right-panel">
            <TaskList/>
            
            <div class="stats-section" style="margin-top: var(--spacing-lg);">
                <h3 class="stats-section-title">"Recent Events"</h3>
                
                <For
                    each=events_list
                    key=|event| event.id.clone()
                    children=move |event: DashboardEvent| {
                        let msg = event.message.clone();
                        let ts = event.timestamp.clone();
                        view! {
                            <div class="stat-row fade-in">
                                <span class="stat-row-label" style="font-size: 0.75rem;">
                                    {msg}
                                </span>
                                <span class="stat-row-value" style="font-size: 0.7rem; color: var(--text-muted);">
                                    {ts}
                                </span>
                            </div>
                        }
                    }
                />
                
                {move || {
                    if events.get().is_empty() {
                        Some(view! {
                            <div style="color: var(--text-muted); font-size: 0.75rem; text-align: center; padding: var(--spacing-md);">
                                "No events yet"
                            </div>
                        })
                    } else {
                        None
                    }
                }}
            </div>
        </aside>
    }
}
