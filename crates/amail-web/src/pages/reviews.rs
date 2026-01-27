//! Review Queue Page - Human review for low-confidence AI responses
//!
//! Table of Contents:
//! 1. ReviewsPage - Main review queue component
//! 2. ReviewCard - Individual review item
//! 3. ReviewModal - Full review with edit capability

use leptos::*;
use wasm_bindgen_futures::spawn_local;
use crate::api;

#[derive(Clone, Debug, PartialEq)]
pub struct ReviewItem {
    pub id: String,
    pub email_id: String,
    pub from: String,
    pub subject: String,
    pub original_preview: String,
    pub draft_response: String,
    pub confidence: u8,
    pub agent: String,
    pub created_at: String,
    pub expires_in: String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ReviewAction {
    Approve,
    Reject,
    Edit,
}

#[component]
pub fn ReviewsPage() -> impl IntoView {
    let (reviews, set_reviews) = create_signal::<Vec<ReviewItem>>(vec![]);
    let (loading, set_loading) = create_signal(true);
    let (selected_review, set_selected_review) = create_signal::<Option<ReviewItem>>(None);
    let (filter, set_filter) = create_signal("all".to_string());
    
    // Fetch reviews from API
    {
        let set_reviews = set_reviews.clone();
        let set_loading = set_loading.clone();
        
        spawn_local(async move {
            match api::fetch_reviews().await {
                Ok(review_list) => {
                    let reviews: Vec<ReviewItem> = review_list.into_iter().map(|r| ReviewItem {
                        id: r.id,
                        email_id: r.email_id.unwrap_or_default(),
                        from: r.from_name.unwrap_or_else(|| r.from_address.unwrap_or_default()),
                        subject: r.subject.unwrap_or_default(),
                        original_preview: r.email_preview.unwrap_or_default(),
                        draft_response: r.draft_response.unwrap_or_default(),
                        confidence: (r.confidence_score.unwrap_or(0.0) * 100.0) as u8,
                        agent: r.agent_role.unwrap_or_else(|| "AI Agent".to_string()),
                        created_at: r.created_at,
                        expires_in: "24h".to_string(),
                    }).collect();
                    set_reviews.set(reviews);
                }
                Err(e) => {
                    log::error!("Failed to fetch reviews: {}", e);
                }
            }
            set_loading.set(false);
        });
    }
    
    let filtered_reviews = move || {
        let f = filter.get();
        reviews.get().into_iter().filter(|r| {
            match f.as_str() {
                "low" => r.confidence < 50,
                "medium" => r.confidence >= 50 && r.confidence < 75,
                "high" => r.confidence >= 75,
                _ => true,
            }
        }).collect::<Vec<_>>()
    };
    
    let pending_count = move || reviews.get().len();
    let low_confidence_count = move || reviews.get().iter().filter(|r| r.confidence < 50).count();

    view! {
        <div class="page reviews-page">
            <div class="reviews-header">
                <div class="reviews-title-section">
                    <h1 class="page-title">"Review Queue"</h1>
                    <p class="page-subtitle">"AI responses awaiting human approval"</p>
                </div>
                
                <div class="reviews-stats">
                    <div class="review-stat">
                        <span class="stat-number">{pending_count}</span>
                        <span class="stat-label">"Pending"</span>
                    </div>
                    <div class="review-stat urgent">
                        <span class="stat-number">{low_confidence_count}</span>
                        <span class="stat-label">"Low Confidence"</span>
                    </div>
                </div>
            </div>
            
            // Filter tabs
            <div class="reviews-filters">
                <button 
                    class=move || if filter.get() == "all" { "filter-btn active" } else { "filter-btn" }
                    on:click=move |_| set_filter.set("all".to_string())
                >"All"</button>
                <button 
                    class=move || if filter.get() == "low" { "filter-btn active" } else { "filter-btn" }
                    on:click=move |_| set_filter.set("low".to_string())
                >"Low (<50%)"</button>
                <button 
                    class=move || if filter.get() == "medium" { "filter-btn active" } else { "filter-btn" }
                    on:click=move |_| set_filter.set("medium".to_string())
                >"Medium (50-75%)"</button>
                <button 
                    class=move || if filter.get() == "high" { "filter-btn active" } else { "filter-btn" }
                    on:click=move |_| set_filter.set("high".to_string())
                >"High (75%+)"</button>
            </div>
            
            // Review list
            <div class="reviews-list">
                {move || {
                    let items = filtered_reviews();
                    if items.is_empty() {
                        view! {
                            <div class="empty-state">
                                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                    <path d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"/>
                                </svg>
                                <h3>"All caught up!"</h3>
                                <p>"No reviews pending in this category."</p>
                            </div>
                        }.into_view()
                    } else {
                        items.into_iter().map(|review| {
                            let review_clone = review.clone();
                            view! {
                                <ReviewCard 
                                    review=review 
                                    on_select=move |r| set_selected_review.set(Some(r))
                                />
                            }
                        }).collect_view()
                    }
                }}
            </div>
            
            // Review modal
            <Show when=move || selected_review.get().is_some()>
                <ReviewModal 
                    review=selected_review.get().unwrap()
                    on_close=move || set_selected_review.set(None)
                    on_action=move |_action| {
                        // Handle action (approve/reject/edit)
                        set_selected_review.set(None);
                    }
                />
            </Show>
        </div>
    }
}

#[component]
fn ReviewCard(
    review: ReviewItem,
    on_select: impl Fn(ReviewItem) + 'static,
) -> impl IntoView {
    let confidence_class = if review.confidence < 50 {
        "confidence-low"
    } else if review.confidence < 75 {
        "confidence-medium"
    } else {
        "confidence-high"
    };
    
    let review_for_click = review.clone();
    
    view! {
        <div class="review-card" on:click=move |_| on_select(review_for_click.clone())>
            <div class="review-card-header">
                <div class="review-sender">
                    <div class="sender-avatar">{review.from.chars().next().unwrap_or('?').to_uppercase().to_string()}</div>
                    <div class="sender-info">
                        <span class="sender-name">{&review.from}</span>
                        <span class="sender-subject">{&review.subject}</span>
                    </div>
                </div>
                <div class=format!("confidence-badge {}", confidence_class)>
                    {review.confidence}"%"
                </div>
            </div>
            
            <div class="review-card-body">
                <div class="original-preview">
                    <span class="preview-label">"Original:"</span>
                    <p>{&review.original_preview}</p>
                </div>
                <div class="draft-preview">
                    <span class="preview-label">"AI Draft:"</span>
                    <p>{&review.draft_response}</p>
                </div>
            </div>
            
            <div class="review-card-footer">
                <div class="review-meta">
                    <span class="agent-badge">{&review.agent}</span>
                    <span class="time-badge">"Expires in "{&review.expires_in}</span>
                </div>
                <div class="review-actions">
                    <button class="action-btn approve" on:click=move |e| { e.stop_propagation(); }>
                        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M5 13l4 4L19 7"/></svg>
                    </button>
                    <button class="action-btn reject" on:click=move |e| { e.stop_propagation(); }>
                        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M6 18L18 6M6 6l12 12"/></svg>
                    </button>
                </div>
            </div>
        </div>
    }
}

#[component]
fn ReviewModal(
    review: ReviewItem,
    on_close: impl Fn() + Clone + 'static,
    on_action: impl Fn(ReviewAction) + Clone + 'static,
) -> impl IntoView {
    let (edited_response, set_edited_response) = create_signal(review.draft_response.clone());
    let (is_editing, set_is_editing) = create_signal(false);
    
    let on_close_header = on_close.clone();
    let on_close_overlay = on_close;
    let on_action_approve = on_action.clone();
    let on_action_reject = on_action.clone();
    let on_action_edit = on_action;
    
    let confidence_class = if review.confidence < 50 { "confidence-low" } else if review.confidence < 75 { "confidence-medium" } else { "confidence-high" };
    
    view! {
        <div class="modal-overlay" on:click=move |_| on_close_overlay()>
            <div class="modal review-modal" on:click=|e| e.stop_propagation()>
                <div class="modal-header">
                    <h2>"Review AI Response"</h2>
                    <button class="modal-close" on:click=move |_| on_close_header()>
                        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M6 18L18 6M6 6l12 12"/></svg>
                    </button>
                </div>
                
                <div class="modal-body">
                    <div class="review-email-info">
                        <div class="info-row">
                            <span class="info-label">"From:"</span>
                            <span class="info-value">{&review.from}</span>
                        </div>
                        <div class="info-row">
                            <span class="info-label">"Subject:"</span>
                            <span class="info-value">{&review.subject}</span>
                        </div>
                        <div class="info-row">
                            <span class="info-label">"Agent:"</span>
                            <span class="info-value agent">{&review.agent}</span>
                            <span class=format!("confidence-badge {}", confidence_class)>
                                {review.confidence}"% confidence"
                            </span>
                        </div>
                    </div>
                    
                    <div class="review-section">
                        <h3>"Original Email"</h3>
                        <div class="email-content original">{&review.original_preview}</div>
                    </div>
                    
                    <div class="review-section">
                        <div class="section-header">
                            <h3>"AI Draft Response"</h3>
                            <button class="edit-toggle" on:click=move |_| set_is_editing.update(|e| *e = !*e)>
                                {move || if is_editing.get() { "Cancel Edit" } else { "Edit Response" }}
                            </button>
                        </div>
                        
                        {move || if is_editing.get() {
                            view! {
                                <textarea 
                                    class="edit-textarea"
                                    prop:value=edited_response
                                    on:input=move |e| set_edited_response.set(event_target_value(&e))
                                />
                            }.into_view()
                        } else {
                            view! { <div class="email-content draft">{edited_response.get()}</div> }.into_view()
                        }}
                    </div>
                </div>
                
                <div class="modal-footer">
                    <button class="btn btn-reject" on:click={
                        let on_action_reject = on_action_reject.clone();
                        move |_| on_action_reject(ReviewAction::Reject)
                    }>
                        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M6 18L18 6M6 6l12 12"/></svg>
                        "Reject"
                    </button>
                    <div class="footer-right">
                        {move || if is_editing.get() {
                            let on_action_edit = on_action_edit.clone();
                            view! {
                                <button class="btn btn-edit" on:click=move |_| on_action_edit(ReviewAction::Edit)>
                                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M11 4H4a2 2 0 00-2 2v14a2 2 0 002 2h14a2 2 0 002-2v-7"/><path d="M18.5 2.5a2.121 2.121 0 013 3L12 15l-4 1 1-4 9.5-9.5z"/></svg>
                                    "Send Edited"
                                </button>
                            }.into_view()
                        } else {
                            view! {}.into_view()
                        }}
                        <button class="btn btn-approve" on:click={
                            let on_action_approve = on_action_approve.clone();
                            move |_| on_action_approve(ReviewAction::Approve)
                        }>
                            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M5 13l4 4L19 7"/></svg>
                            "Approve & Send"
                        </button>
                    </div>
                </div>
            </div>
        </div>
    }
}

// Sample data removed - now using real API data
