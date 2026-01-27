//! Internal Scheduler - Daily Tasks and Automated Jobs
//!
//! Table of Contents:
//! 1. SchedulerConfig - Configuration for scheduled tasks
//! 2. AmailScheduler - Main scheduler with job management
//! 3. ScheduledJob - Job types (daily summary, cleanup, etc.)
//! 4. OS Script Generation - Task Scheduler, systemd, launchd

use anyhow::Result;
use chrono::{NaiveTime, Timelike};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::{info, error};

// =============================================================================
// 1. Scheduler Configuration
// =============================================================================

/// Configuration for the Amail scheduler
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerConfig {
    /// Enable auto-start features at scheduled time
    pub auto_start_enabled: bool,
    /// Time to activate daily features (default: 08:00)
    pub daily_start_time: NaiveTime,
    /// Enable daily summary generation
    pub daily_summary_enabled: bool,
    /// Enable overnight email processing on startup
    pub process_overnight_emails: bool,
    /// Enable automatic cleanup of old data
    pub auto_cleanup_enabled: bool,
    /// Days to retain processed emails
    pub retention_days: u32,
    /// Enable AI agents at startup
    pub activate_agents_on_start: bool,
    /// Send Telegram digest at daily start time
    pub telegram_daily_digest: bool,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            auto_start_enabled: true,
            daily_start_time: NaiveTime::from_hms_opt(8, 0, 0).unwrap(),
            daily_summary_enabled: true,
            process_overnight_emails: true,
            auto_cleanup_enabled: true,
            retention_days: 30,
            activate_agents_on_start: true,
            telegram_daily_digest: true,
        }
    }
}

impl SchedulerConfig {
    /// Create config from environment variables
    pub fn from_env() -> Self {
        let mut config = Self::default();
        
        if let Ok(enabled) = std::env::var("AMAIL_AUTO_START_ENABLED") {
            config.auto_start_enabled = enabled.parse().unwrap_or(true);
        }
        
        if let Ok(time_str) = std::env::var("AMAIL_DAILY_START_TIME") {
            if let Ok(time) = NaiveTime::parse_from_str(&time_str, "%H:%M") {
                config.daily_start_time = time;
            }
        }
        
        if let Ok(days) = std::env::var("AMAIL_RETENTION_DAYS") {
            config.retention_days = days.parse().unwrap_or(30);
        }
        
        config
    }
    
    /// Get cron expression for daily start time
    pub fn daily_cron(&self) -> String {
        format!(
            "0 {} {} * * *",
            self.daily_start_time.minute(),
            self.daily_start_time.hour()
        )
    }
    
    /// Format time for display
    pub fn time_display(&self) -> String {
        self.daily_start_time.format("%I:%M %p").to_string()
    }
}

// =============================================================================
// 2. Amail Scheduler
// =============================================================================

/// Main scheduler for Amail automated tasks
pub struct AmailScheduler {
    scheduler: JobScheduler,
    config: Arc<RwLock<SchedulerConfig>>,
    running: Arc<RwLock<bool>>,
}

impl AmailScheduler {
    /// Create a new scheduler with the given configuration
    pub async fn new(config: SchedulerConfig) -> Result<Self> {
        let scheduler = JobScheduler::new().await?;
        
        Ok(Self {
            scheduler,
            config: Arc::new(RwLock::new(config)),
            running: Arc::new(RwLock::new(false)),
        })
    }
    
    /// Start the scheduler and register all jobs
    pub async fn start(&self) -> Result<()> {
        let config = self.config.read().await;
        
        if !config.auto_start_enabled {
            info!("Scheduler disabled in configuration");
            return Ok(());
        }
        
        info!("Starting Amail scheduler with daily time: {}", config.time_display());
        
        // Register daily jobs
        self.register_daily_jobs(&config).await?;
        
        // Start the scheduler
        self.scheduler.start().await?;
        
        *self.running.write().await = true;
        
        info!("Amail scheduler started successfully");
        Ok(())
    }
    
    /// Stop the scheduler
    pub async fn stop(&mut self) -> Result<()> {
        info!("Stopping Amail scheduler");
        self.scheduler.shutdown().await?;
        *self.running.write().await = false;
        Ok(())
    }
    
    /// Update scheduler configuration
    pub async fn update_config(&mut self, new_config: SchedulerConfig) -> Result<()> {
        info!("Updating scheduler configuration");
        
        // Stop current scheduler
        self.scheduler.shutdown().await?;
        
        // Update config
        *self.config.write().await = new_config;
        
        // Restart with new config
        self.start().await?;
        
        Ok(())
    }
    
    /// Register all daily scheduled jobs
    async fn register_daily_jobs(&self, config: &SchedulerConfig) -> Result<()> {
        let cron = config.daily_cron();
        
        // Daily summary job
        if config.daily_summary_enabled {
            let job = Job::new_async(cron.as_str(), |_uuid, _lock| {
                Box::pin(async move {
                    info!("Running daily summary job");
                    if let Err(e) = run_daily_summary().await {
                        error!("Daily summary failed: {}", e);
                    }
                })
            })?;
            self.scheduler.add(job).await?;
            info!("Registered daily summary job at {}", config.time_display());
        }
        
        // Overnight email processing job
        if config.process_overnight_emails {
            let job = Job::new_async(cron.as_str(), |_uuid, _lock| {
                Box::pin(async move {
                    info!("Processing overnight emails");
                    if let Err(e) = process_overnight_emails().await {
                        error!("Overnight email processing failed: {}", e);
                    }
                })
            })?;
            self.scheduler.add(job).await?;
            info!("Registered overnight email processing at {}", config.time_display());
        }
        
        // Cleanup job (runs at daily start time)
        if config.auto_cleanup_enabled {
            let retention_days = config.retention_days;
            let job = Job::new_async(cron.as_str(), move |_uuid, _lock| {
                Box::pin(async move {
                    info!("Running cleanup job (retention: {} days)", retention_days);
                    if let Err(e) = run_cleanup(retention_days).await {
                        error!("Cleanup failed: {}", e);
                    }
                })
            })?;
            self.scheduler.add(job).await?;
            info!("Registered cleanup job at {}", config.time_display());
        }
        
        // Telegram daily digest
        if config.telegram_daily_digest {
            let job = Job::new_async(cron.as_str(), |_uuid, _lock| {
                Box::pin(async move {
                    info!("Sending Telegram daily digest");
                    if let Err(e) = send_telegram_digest().await {
                        error!("Telegram digest failed: {}", e);
                    }
                })
            })?;
            self.scheduler.add(job).await?;
            info!("Registered Telegram digest at {}", config.time_display());
        }
        
        // Agent activation job
        if config.activate_agents_on_start {
            let job = Job::new_async(cron.as_str(), |_uuid, _lock| {
                Box::pin(async move {
                    info!("Activating AI agents");
                    if let Err(e) = activate_agents().await {
                        error!("Agent activation failed: {}", e);
                    }
                })
            })?;
            self.scheduler.add(job).await?;
            info!("Registered agent activation at {}", config.time_display());
        }
        
        Ok(())
    }
    
    /// Check if scheduler is running
    pub async fn is_running(&self) -> bool {
        *self.running.read().await
    }
    
    /// Get current configuration
    pub async fn get_config(&self) -> SchedulerConfig {
        self.config.read().await.clone()
    }
}

// =============================================================================
// 3. Scheduled Job Implementations
// =============================================================================

/// Run daily summary generation
async fn run_daily_summary() -> Result<()> {
    info!("Generating daily summary...");
    // TODO: Integrate with analytics module to generate summary
    // This will call KpiTracker::generate_summary() and format for display
    Ok(())
}

/// Process emails received overnight
async fn process_overnight_emails() -> Result<()> {
    info!("Processing overnight emails...");
    // TODO: Fetch unprocessed emails from Stalwart and route to agents
    Ok(())
}

/// Run cleanup of old data
async fn run_cleanup(retention_days: u32) -> Result<()> {
    info!("Running cleanup (retention: {} days)...", retention_days);
    // TODO: Delete old emails, logs, and analytics data beyond retention period
    Ok(())
}

/// Send Telegram daily digest
async fn send_telegram_digest() -> Result<()> {
    info!("Sending Telegram daily digest...");
    // TODO: Integrate with telegram module to send daily summary
    Ok(())
}

/// Activate AI agents
async fn activate_agents() -> Result<()> {
    info!("Activating AI agents...");
    // TODO: Start all configured agents via Actix actor system
    Ok(())
}

// =============================================================================
// 4. OS Script Generation
// =============================================================================

/// Generate Windows Task Scheduler XML
pub fn generate_task_scheduler_xml(config: &SchedulerConfig, exe_path: &str) -> String {
    let hour = config.daily_start_time.hour();
    let minute = config.daily_start_time.minute();
    
    format!(r#"<?xml version="1.0" encoding="UTF-16"?>
<Task version="1.4" xmlns="http://schemas.microsoft.com/windows/2004/02/mit/task">
  <RegistrationInfo>
    <Description>Start Amail AI Mail Server at scheduled time</Description>
    <Author>Amail</Author>
  </RegistrationInfo>
  <Triggers>
    <CalendarTrigger>
      <StartBoundary>2024-01-01T{:02}:{:02}:00</StartBoundary>
      <Enabled>true</Enabled>
      <ScheduleByDay>
        <DaysInterval>1</DaysInterval>
      </ScheduleByDay>
    </CalendarTrigger>
    <BootTrigger>
      <Enabled>true</Enabled>
      <Delay>PT1M</Delay>
    </BootTrigger>
  </Triggers>
  <Principals>
    <Principal id="Author">
      <LogonType>InteractiveToken</LogonType>
      <RunLevel>HighestAvailable</RunLevel>
    </Principal>
  </Principals>
  <Settings>
    <MultipleInstancesPolicy>IgnoreNew</MultipleInstancesPolicy>
    <DisallowStartIfOnBatteries>false</DisallowStartIfOnBatteries>
    <StopIfGoingOnBatteries>false</StopIfGoingOnBatteries>
    <AllowHardTerminate>true</AllowHardTerminate>
    <StartWhenAvailable>true</StartWhenAvailable>
    <RunOnlyIfNetworkAvailable>false</RunOnlyIfNetworkAvailable>
    <AllowStartOnDemand>true</AllowStartOnDemand>
    <Enabled>true</Enabled>
    <Hidden>false</Hidden>
    <RunOnlyIfIdle>false</RunOnlyIfIdle>
    <WakeToRun>true</WakeToRun>
    <ExecutionTimeLimit>PT0S</ExecutionTimeLimit>
    <Priority>7</Priority>
  </Settings>
  <Actions Context="Author">
    <Exec>
      <Command>{}</Command>
      <WorkingDirectory>{}</WorkingDirectory>
    </Exec>
  </Actions>
</Task>"#, 
        hour, minute, 
        exe_path,
        std::path::Path::new(exe_path).parent().map(|p| p.display().to_string()).unwrap_or_default()
    )
}

/// Generate systemd service unit file (Linux)
pub fn generate_systemd_unit(config: &SchedulerConfig, exe_path: &str, user: &str) -> String {
    format!(r#"[Unit]
Description=Amail AI Mail Server
After=network.target

[Service]
Type=simple
User={}
ExecStart={}
Restart=always
RestartSec=10
Environment=AMAIL_DAILY_START_TIME={:02}:{:02}

[Install]
WantedBy=multi-user.target
"#,
        user,
        exe_path,
        config.daily_start_time.hour(),
        config.daily_start_time.minute()
    )
}

/// Generate systemd timer unit file (Linux)
pub fn generate_systemd_timer(config: &SchedulerConfig) -> String {
    format!(r#"[Unit]
Description=Amail Daily Start Timer

[Timer]
OnCalendar=*-*-* {:02}:{:02}:00
Persistent=true

[Install]
WantedBy=timers.target
"#,
        config.daily_start_time.hour(),
        config.daily_start_time.minute()
    )
}

/// Generate launchd plist (macOS)
pub fn generate_launchd_plist(config: &SchedulerConfig, exe_path: &str, label: &str) -> String {
    format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>{}</string>
    <key>ProgramArguments</key>
    <array>
        <string>{}</string>
    </array>
    <key>StartCalendarInterval</key>
    <dict>
        <key>Hour</key>
        <integer>{}</integer>
        <key>Minute</key>
        <integer>{}</integer>
    </dict>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardOutPath</key>
    <string>/tmp/amail.log</string>
    <key>StandardErrorPath</key>
    <string>/tmp/amail.err</string>
</dict>
</plist>
"#,
        label,
        exe_path,
        config.daily_start_time.hour(),
        config.daily_start_time.minute()
    )
}

/// Generate PowerShell script to install Task Scheduler task
pub fn generate_powershell_install_script(config: &SchedulerConfig, exe_path: &str) -> String {
    let hour = config.daily_start_time.hour();
    let minute = config.daily_start_time.minute();
    
    format!(r#"# Amail Auto-Start Installation Script
# Run as Administrator

$TaskName = "Amail-AutoStart"
$ExePath = "{}"

# Remove existing task if present
Unregister-ScheduledTask -TaskName $TaskName -Confirm:$false -ErrorAction SilentlyContinue

# Create action
$Action = New-ScheduledTaskAction -Execute $ExePath

# Create triggers (daily at specified time + on boot)
$DailyTrigger = New-ScheduledTaskTrigger -Daily -At "{:02}:{:02}"
$BootTrigger = New-ScheduledTaskTrigger -AtStartup

# Create settings
$Settings = New-ScheduledTaskSettingsSet -AllowStartIfOnBatteries -DontStopIfGoingOnBatteries -StartWhenAvailable -WakeToRun

# Register the task
Register-ScheduledTask -TaskName $TaskName -Action $Action -Trigger $DailyTrigger,$BootTrigger -Settings $Settings -Description "Start Amail AI Mail Server"

Write-Host "Amail auto-start task installed successfully!"
Write-Host "Server will start at {:02}:{:02} daily and on system boot."
"#,
        exe_path, hour, minute, hour, minute
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_config() {
        let config = SchedulerConfig::default();
        assert!(config.auto_start_enabled);
        assert_eq!(config.daily_start_time.hour(), 8);
        assert_eq!(config.daily_start_time.minute(), 0);
    }
    
    #[test]
    fn test_cron_expression() {
        let config = SchedulerConfig::default();
        assert_eq!(config.daily_cron(), "0 0 8 * * *");
        
        let mut config2 = config.clone();
        config2.daily_start_time = NaiveTime::from_hms_opt(14, 30, 0).unwrap();
        assert_eq!(config2.daily_cron(), "0 30 14 * * *");
    }
    
    #[test]
    fn test_time_display() {
        let config = SchedulerConfig::default();
        assert_eq!(config.time_display(), "08:00 AM");
    }
}
