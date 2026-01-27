-- =============================================================================
-- Amail Database Schema - Initial Migration for Supabase
-- =============================================================================
-- This is the consolidated initial migration containing all tables
-- Run: supabase db push
-- =============================================================================

-- =============================================================================
-- 1. Core Tables: Domains and Mailboxes
-- =============================================================================

CREATE TABLE IF NOT EXISTS "public"."domains" (
    "id" TEXT PRIMARY KEY,
    "name" TEXT NOT NULL UNIQUE,
    "display_name" TEXT NOT NULL,
    "enabled" BOOLEAN NOT NULL DEFAULT TRUE,
    "primary" BOOLEAN NOT NULL DEFAULT FALSE,
    "mx_verified" BOOLEAN NOT NULL DEFAULT FALSE,
    "spf_verified" BOOLEAN NOT NULL DEFAULT FALSE,
    "dkim_verified" BOOLEAN NOT NULL DEFAULT FALSE,
    "dmarc_verified" BOOLEAN NOT NULL DEFAULT FALSE,
    "created_at" TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS "public"."mailboxes" (
    "id" TEXT PRIMARY KEY,
    "domain_id" TEXT NOT NULL REFERENCES "public"."domains"("id") ON DELETE CASCADE,
    "email" TEXT NOT NULL UNIQUE,
    "display_name" TEXT NOT NULL,
    "enabled" BOOLEAN NOT NULL DEFAULT TRUE,
    "created_at" TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS "idx_domains_name" ON "public"."domains"("name");
CREATE INDEX IF NOT EXISTS "idx_domains_enabled" ON "public"."domains"("enabled");
CREATE INDEX IF NOT EXISTS "idx_mailboxes_domain_id" ON "public"."mailboxes"("domain_id");
CREATE INDEX IF NOT EXISTS "idx_mailboxes_email" ON "public"."mailboxes"("email");

-- =============================================================================
-- 2. Emails Table
-- =============================================================================

CREATE TABLE IF NOT EXISTS "public"."emails" (
    "message_id" TEXT PRIMARY KEY,
    "tracking_id" UUID NOT NULL DEFAULT gen_random_uuid(),
    "mailbox_id" TEXT REFERENCES "public"."mailboxes"("id"),
    "from_address" TEXT NOT NULL,
    "from_name" TEXT,
    "to_addresses" TEXT[] NOT NULL DEFAULT '{}',
    "cc_addresses" TEXT[] NOT NULL DEFAULT '{}',
    "subject" TEXT NOT NULL,
    "text_body" TEXT,
    "html_body" TEXT,
    "folder" TEXT NOT NULL DEFAULT 'inbox',
    "is_read" BOOLEAN NOT NULL DEFAULT FALSE,
    "is_starred" BOOLEAN NOT NULL DEFAULT FALSE,
    "is_archived" BOOLEAN NOT NULL DEFAULT FALSE,
    "is_spam" BOOLEAN NOT NULL DEFAULT FALSE,
    "is_trash" BOOLEAN NOT NULL DEFAULT FALSE,
    "labels" TEXT[] NOT NULL DEFAULT '{}',
    "thread_id" TEXT,
    "in_reply_to" TEXT,
    "references_ids" TEXT[] NOT NULL DEFAULT '{}',
    "priority" TEXT NOT NULL DEFAULT 'normal',
    "received_at" TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    "created_at" TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS "idx_emails_received_at" ON "public"."emails"("received_at" DESC);
CREATE INDEX IF NOT EXISTS "idx_emails_from_address" ON "public"."emails"("from_address");
CREATE INDEX IF NOT EXISTS "idx_emails_mailbox_id" ON "public"."emails"("mailbox_id");
CREATE INDEX IF NOT EXISTS "idx_emails_folder" ON "public"."emails"("folder");
CREATE INDEX IF NOT EXISTS "idx_emails_is_read" ON "public"."emails"("is_read");
CREATE INDEX IF NOT EXISTS "idx_emails_thread_id" ON "public"."emails"("thread_id");

-- =============================================================================
-- 3. Email Attachments
-- =============================================================================

CREATE TABLE IF NOT EXISTS "public"."email_attachments" (
    "id" TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    "email_id" TEXT NOT NULL REFERENCES "public"."emails"("message_id") ON DELETE CASCADE,
    "filename" TEXT NOT NULL,
    "content_type" TEXT NOT NULL,
    "size_bytes" BIGINT NOT NULL,
    "storage_path" TEXT,
    "checksum" TEXT,
    "created_at" TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS "idx_email_attachments_email_id" ON "public"."email_attachments"("email_id");

-- =============================================================================
-- 4. Tasks Table
-- =============================================================================

CREATE TABLE IF NOT EXISTS "public"."tasks" (
    "id" TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    "description" TEXT NOT NULL,
    "task_type" TEXT NOT NULL,
    "status" TEXT NOT NULL DEFAULT 'pending',
    "priority" TEXT NOT NULL DEFAULT 'normal',
    "assigned_agent" TEXT,
    "email_id" TEXT REFERENCES "public"."emails"("message_id"),
    "parent_task_id" TEXT REFERENCES "public"."tasks"("id"),
    "result" JSONB,
    "error_message" TEXT,
    "retry_count" INTEGER NOT NULL DEFAULT 0,
    "max_retries" INTEGER NOT NULL DEFAULT 3,
    "created_at" TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    "updated_at" TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    "started_at" TIMESTAMPTZ,
    "completed_at" TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS "idx_tasks_status" ON "public"."tasks"("status");
CREATE INDEX IF NOT EXISTS "idx_tasks_priority" ON "public"."tasks"("priority");
CREATE INDEX IF NOT EXISTS "idx_tasks_assigned_agent" ON "public"."tasks"("assigned_agent");
CREATE INDEX IF NOT EXISTS "idx_tasks_created_at" ON "public"."tasks"("created_at" DESC);

-- =============================================================================
-- 5. Agent States
-- =============================================================================

CREATE TABLE IF NOT EXISTS "public"."agent_states" (
    "id" TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    "role" TEXT NOT NULL,
    "status" TEXT NOT NULL DEFAULT 'idle',
    "last_heartbeat" TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    "tasks_processed" BIGINT NOT NULL DEFAULT 0,
    "current_task_id" TEXT REFERENCES "public"."tasks"("id"),
    "metadata" JSONB,
    "created_at" TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS "idx_agent_states_role" ON "public"."agent_states"("role");
CREATE INDEX IF NOT EXISTS "idx_agent_states_status" ON "public"."agent_states"("status");

-- =============================================================================
-- 6. Task History / Audit Log
-- =============================================================================

CREATE TABLE IF NOT EXISTS "public"."task_history" (
    "id" TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    "task_id" TEXT NOT NULL REFERENCES "public"."tasks"("id"),
    "agent_id" TEXT,
    "action" TEXT NOT NULL,
    "details" JSONB,
    "created_at" TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS "idx_task_history_task_id" ON "public"."task_history"("task_id");
CREATE INDEX IF NOT EXISTS "idx_task_history_created_at" ON "public"."task_history"("created_at" DESC);

-- =============================================================================
-- 7. Outbound Emails Queue
-- =============================================================================

CREATE TABLE IF NOT EXISTS "public"."outbound_emails" (
    "id" TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    "task_id" TEXT REFERENCES "public"."tasks"("id"),
    "to_address" TEXT NOT NULL,
    "cc_addresses" TEXT[] NOT NULL DEFAULT '{}',
    "subject" TEXT NOT NULL,
    "body" TEXT NOT NULL,
    "html_body" TEXT,
    "status" TEXT NOT NULL DEFAULT 'pending',
    "send_attempts" INTEGER NOT NULL DEFAULT 0,
    "last_error" TEXT,
    "created_at" TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    "sent_at" TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS "idx_outbound_emails_status" ON "public"."outbound_emails"("status");

-- =============================================================================
-- 8. Human Review Queue
-- =============================================================================

CREATE TABLE IF NOT EXISTS "public"."human_reviews" (
    "id" TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    "task_id" TEXT REFERENCES "public"."tasks"("id"),
    "email_id" TEXT REFERENCES "public"."emails"("message_id"),
    "agent_id" TEXT,
    "reason" TEXT NOT NULL,
    "urgency" TEXT NOT NULL DEFAULT 'medium',
    "draft_response" TEXT,
    "status" TEXT NOT NULL DEFAULT 'pending',
    "confidence_score" DECIMAL(5,4),
    "telegram_message_id" BIGINT,
    "decision" TEXT,
    "edited_response" TEXT,
    "reviewer_notes" TEXT,
    "reviewed_by" TEXT,
    "created_at" TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    "reviewed_at" TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS "idx_human_reviews_status" ON "public"."human_reviews"("status");
CREATE INDEX IF NOT EXISTS "idx_human_reviews_urgency" ON "public"."human_reviews"("urgency");
CREATE INDEX IF NOT EXISTS "idx_human_reviews_email_id" ON "public"."human_reviews"("email_id");
CREATE INDEX IF NOT EXISTS "idx_human_reviews_confidence" ON "public"."human_reviews"("confidence_score");

-- =============================================================================
-- 9. Draft Responses
-- =============================================================================

CREATE TABLE IF NOT EXISTS "public"."draft_responses" (
    "id" TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    "email_id" TEXT NOT NULL REFERENCES "public"."emails"("message_id"),
    "agent_id" TEXT NOT NULL,
    "agent_role" TEXT NOT NULL,
    "subject" TEXT NOT NULL,
    "body_text" TEXT NOT NULL,
    "body_html" TEXT,
    "confidence_score" DECIMAL(5,4) NOT NULL,
    "reasoning" TEXT,
    "status" TEXT NOT NULL DEFAULT 'pending',
    "review_id" TEXT REFERENCES "public"."human_reviews"("id"),
    "created_at" TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    "approved_at" TIMESTAMPTZ,
    "sent_at" TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS "idx_draft_responses_email_id" ON "public"."draft_responses"("email_id");
CREATE INDEX IF NOT EXISTS "idx_draft_responses_status" ON "public"."draft_responses"("status");
CREATE INDEX IF NOT EXISTS "idx_draft_responses_agent_id" ON "public"."draft_responses"("agent_id");

-- =============================================================================
-- 10. Analytics - Daily Metrics
-- =============================================================================

CREATE TABLE IF NOT EXISTS "public"."daily_metrics" (
    "id" TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    "date" DATE NOT NULL UNIQUE,
    "emails_received" BIGINT NOT NULL DEFAULT 0,
    "emails_sent" BIGINT NOT NULL DEFAULT 0,
    "emails_auto_replied" BIGINT NOT NULL DEFAULT 0,
    "emails_forwarded" BIGINT NOT NULL DEFAULT 0,
    "tasks_created" BIGINT NOT NULL DEFAULT 0,
    "tasks_completed" BIGINT NOT NULL DEFAULT 0,
    "tasks_failed" BIGINT NOT NULL DEFAULT 0,
    "avg_response_time_ms" BIGINT,
    "avg_processing_time_ms" BIGINT,
    "reviews_requested" BIGINT NOT NULL DEFAULT 0,
    "reviews_approved" BIGINT NOT NULL DEFAULT 0,
    "reviews_rejected" BIGINT NOT NULL DEFAULT 0,
    "llm_calls" BIGINT NOT NULL DEFAULT 0,
    "llm_tokens_used" BIGINT NOT NULL DEFAULT 0,
    "created_at" TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS "idx_daily_metrics_date" ON "public"."daily_metrics"("date" DESC);

-- =============================================================================
-- 11. Analytics - Hourly Metrics
-- =============================================================================

CREATE TABLE IF NOT EXISTS "public"."hourly_metrics" (
    "id" TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    "hour" TIMESTAMPTZ NOT NULL UNIQUE,
    "emails_received" INTEGER NOT NULL DEFAULT 0,
    "emails_sent" INTEGER NOT NULL DEFAULT 0,
    "tasks_completed" INTEGER NOT NULL DEFAULT 0,
    "avg_response_time_ms" INTEGER,
    "created_at" TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS "idx_hourly_metrics_hour" ON "public"."hourly_metrics"("hour" DESC);

-- =============================================================================
-- 12. Analytics - Agent Metrics
-- =============================================================================

CREATE TABLE IF NOT EXISTS "public"."agent_metrics" (
    "id" TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    "agent_id" TEXT NOT NULL,
    "date" DATE NOT NULL,
    "tasks_processed" BIGINT NOT NULL DEFAULT 0,
    "tasks_successful" BIGINT NOT NULL DEFAULT 0,
    "tasks_failed" BIGINT NOT NULL DEFAULT 0,
    "avg_processing_time_ms" BIGINT,
    "llm_calls" BIGINT NOT NULL DEFAULT 0,
    "created_at" TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE("agent_id", "date")
);

CREATE INDEX IF NOT EXISTS "idx_agent_metrics_agent_id" ON "public"."agent_metrics"("agent_id");
CREATE INDEX IF NOT EXISTS "idx_agent_metrics_date" ON "public"."agent_metrics"("date" DESC);

-- =============================================================================
-- 13. Scheduler Configuration
-- =============================================================================

CREATE TABLE IF NOT EXISTS "public"."scheduler_config" (
    "id" INTEGER PRIMARY KEY DEFAULT 1 CHECK (id = 1),
    "auto_start_enabled" BOOLEAN NOT NULL DEFAULT TRUE,
    "daily_start_time" TIME NOT NULL DEFAULT '08:00:00',
    "daily_summary_enabled" BOOLEAN NOT NULL DEFAULT TRUE,
    "process_overnight_emails" BOOLEAN NOT NULL DEFAULT TRUE,
    "auto_cleanup_enabled" BOOLEAN NOT NULL DEFAULT TRUE,
    "retention_days" INTEGER NOT NULL DEFAULT 30,
    "activate_agents_on_start" BOOLEAN NOT NULL DEFAULT TRUE,
    "telegram_daily_digest" BOOLEAN NOT NULL DEFAULT TRUE,
    "updated_at" TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

INSERT INTO "public"."scheduler_config" ("id") VALUES (1) ON CONFLICT ("id") DO NOTHING;

-- =============================================================================
-- 14. Scheduled Job Runs Log
-- =============================================================================

CREATE TABLE IF NOT EXISTS "public"."scheduled_job_runs" (
    "id" TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    "job_name" TEXT NOT NULL,
    "started_at" TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    "completed_at" TIMESTAMPTZ,
    "status" TEXT NOT NULL DEFAULT 'running',
    "result" JSONB,
    "error_message" TEXT
);

CREATE INDEX IF NOT EXISTS "idx_scheduled_job_runs_job_name" ON "public"."scheduled_job_runs"("job_name");
CREATE INDEX IF NOT EXISTS "idx_scheduled_job_runs_started_at" ON "public"."scheduled_job_runs"("started_at" DESC);

-- =============================================================================
-- 15. Server Configuration
-- =============================================================================

CREATE TABLE IF NOT EXISTS "public"."server_config" (
    "id" INTEGER PRIMARY KEY DEFAULT 1 CHECK (id = 1),
    "smtp_port" INTEGER NOT NULL DEFAULT 2525,
    "http_port" INTEGER NOT NULL DEFAULT 4000,
    "ollama_url" TEXT NOT NULL DEFAULT 'http://localhost:11434',
    "llm_model" TEXT NOT NULL DEFAULT 'mistral',
    "stalwart_url" TEXT NOT NULL DEFAULT 'http://localhost:8080',
    "stalwart_admin_token" TEXT,
    "telegram_bot_token" TEXT,
    "telegram_chat_id" TEXT,
    "telegram_enabled" BOOLEAN NOT NULL DEFAULT FALSE,
    "updated_at" TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

INSERT INTO "public"."server_config" ("id") VALUES (1) ON CONFLICT ("id") DO NOTHING;

-- =============================================================================
-- 16. Update Triggers
-- =============================================================================

CREATE OR REPLACE FUNCTION "public"."update_updated_at_column"()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE 'plpgsql';

DROP TRIGGER IF EXISTS "update_tasks_updated_at" ON "public"."tasks";
CREATE TRIGGER "update_tasks_updated_at" BEFORE UPDATE ON "public"."tasks"
    FOR EACH ROW EXECUTE FUNCTION "public"."update_updated_at_column"();

DROP TRIGGER IF EXISTS "update_scheduler_config_updated_at" ON "public"."scheduler_config";
CREATE TRIGGER "update_scheduler_config_updated_at" BEFORE UPDATE ON "public"."scheduler_config"
    FOR EACH ROW EXECUTE FUNCTION "public"."update_updated_at_column"();

DROP TRIGGER IF EXISTS "update_server_config_updated_at" ON "public"."server_config";
CREATE TRIGGER "update_server_config_updated_at" BEFORE UPDATE ON "public"."server_config"
    FOR EACH ROW EXECUTE FUNCTION "public"."update_updated_at_column"();

-- =============================================================================
-- 17. Row Level Security (RLS)
-- =============================================================================

ALTER TABLE "public"."domains" ENABLE ROW LEVEL SECURITY;
ALTER TABLE "public"."mailboxes" ENABLE ROW LEVEL SECURITY;
ALTER TABLE "public"."emails" ENABLE ROW LEVEL SECURITY;
ALTER TABLE "public"."email_attachments" ENABLE ROW LEVEL SECURITY;
ALTER TABLE "public"."tasks" ENABLE ROW LEVEL SECURITY;
ALTER TABLE "public"."agent_states" ENABLE ROW LEVEL SECURITY;
ALTER TABLE "public"."task_history" ENABLE ROW LEVEL SECURITY;
ALTER TABLE "public"."outbound_emails" ENABLE ROW LEVEL SECURITY;
ALTER TABLE "public"."human_reviews" ENABLE ROW LEVEL SECURITY;
ALTER TABLE "public"."draft_responses" ENABLE ROW LEVEL SECURITY;
ALTER TABLE "public"."daily_metrics" ENABLE ROW LEVEL SECURITY;
ALTER TABLE "public"."hourly_metrics" ENABLE ROW LEVEL SECURITY;
ALTER TABLE "public"."agent_metrics" ENABLE ROW LEVEL SECURITY;
ALTER TABLE "public"."scheduler_config" ENABLE ROW LEVEL SECURITY;
ALTER TABLE "public"."scheduled_job_runs" ENABLE ROW LEVEL SECURITY;
ALTER TABLE "public"."server_config" ENABLE ROW LEVEL SECURITY;

-- Allow all operations (adjust for production)
CREATE POLICY "allow_all" ON "public"."domains" FOR ALL USING (true) WITH CHECK (true);
CREATE POLICY "allow_all" ON "public"."mailboxes" FOR ALL USING (true) WITH CHECK (true);
CREATE POLICY "allow_all" ON "public"."emails" FOR ALL USING (true) WITH CHECK (true);
CREATE POLICY "allow_all" ON "public"."email_attachments" FOR ALL USING (true) WITH CHECK (true);
CREATE POLICY "allow_all" ON "public"."tasks" FOR ALL USING (true) WITH CHECK (true);
CREATE POLICY "allow_all" ON "public"."agent_states" FOR ALL USING (true) WITH CHECK (true);
CREATE POLICY "allow_all" ON "public"."task_history" FOR ALL USING (true) WITH CHECK (true);
CREATE POLICY "allow_all" ON "public"."outbound_emails" FOR ALL USING (true) WITH CHECK (true);
CREATE POLICY "allow_all" ON "public"."human_reviews" FOR ALL USING (true) WITH CHECK (true);
CREATE POLICY "allow_all" ON "public"."draft_responses" FOR ALL USING (true) WITH CHECK (true);
CREATE POLICY "allow_all" ON "public"."daily_metrics" FOR ALL USING (true) WITH CHECK (true);
CREATE POLICY "allow_all" ON "public"."hourly_metrics" FOR ALL USING (true) WITH CHECK (true);
CREATE POLICY "allow_all" ON "public"."agent_metrics" FOR ALL USING (true) WITH CHECK (true);
CREATE POLICY "allow_all" ON "public"."scheduler_config" FOR ALL USING (true) WITH CHECK (true);
CREATE POLICY "allow_all" ON "public"."scheduled_job_runs" FOR ALL USING (true) WITH CHECK (true);
CREATE POLICY "allow_all" ON "public"."server_config" FOR ALL USING (true) WITH CHECK (true);

-- =============================================================================
-- 18. Grant Permissions
-- =============================================================================

GRANT ALL ON TABLE "public"."domains" TO "anon", "authenticated", "service_role";
GRANT ALL ON TABLE "public"."mailboxes" TO "anon", "authenticated", "service_role";
GRANT ALL ON TABLE "public"."emails" TO "anon", "authenticated", "service_role";
GRANT ALL ON TABLE "public"."email_attachments" TO "anon", "authenticated", "service_role";
GRANT ALL ON TABLE "public"."tasks" TO "anon", "authenticated", "service_role";
GRANT ALL ON TABLE "public"."agent_states" TO "anon", "authenticated", "service_role";
GRANT ALL ON TABLE "public"."task_history" TO "anon", "authenticated", "service_role";
GRANT ALL ON TABLE "public"."outbound_emails" TO "anon", "authenticated", "service_role";
GRANT ALL ON TABLE "public"."human_reviews" TO "anon", "authenticated", "service_role";
GRANT ALL ON TABLE "public"."draft_responses" TO "anon", "authenticated", "service_role";
GRANT ALL ON TABLE "public"."daily_metrics" TO "anon", "authenticated", "service_role";
GRANT ALL ON TABLE "public"."hourly_metrics" TO "anon", "authenticated", "service_role";
GRANT ALL ON TABLE "public"."agent_metrics" TO "anon", "authenticated", "service_role";
GRANT ALL ON TABLE "public"."scheduler_config" TO "anon", "authenticated", "service_role";
GRANT ALL ON TABLE "public"."scheduled_job_runs" TO "anon", "authenticated", "service_role";
GRANT ALL ON TABLE "public"."server_config" TO "anon", "authenticated", "service_role";
