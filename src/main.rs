// =============================================================================
// AMAIL Server - Main Entry Point
// =============================================================================
// Table of Contents:
// 1. Configuration loading
// 2. Agent swarm initialization
// 3. HTTP server startup
// 4. Graceful shutdown handling
// =============================================================================

use actix::prelude::*;
use amail_core::traits::SupervisionStrategy;
use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
use amail_agents::{
    agents::{create_core_swarm, create_full_swarm},
    pipeline::{Pipeline, PipelineConfig},
    registry::AgentRegistry,
    supervisor::SupervisorActor,
};
use amail_core::{
    db::Database,
    queue::TaskQueue,
    config_manager::ConfigManager,
};
use amail_llm::LlmClient;
use amail_mcp::McpClient;
use amail_webhook::{configure_routes, configure_api_routes, handlers::AppState, ApiState, DashboardBroadcaster, SmtpConfig, StalwartClient, ConfigState, configure_config_routes};
use anyhow::Result;
use clap::Parser;
use std::sync::Arc;
use std::process::{Command, Stdio};
use tokio::sync::mpsc;
use tracing::{info, warn, Level};
use tracing_subscriber::FmtSubscriber;

/// Amail Server CLI arguments
#[derive(Parser, Debug)]
#[command(name = "amail-server")]
#[command(about = "Email-triggered AI agent swarm server")]
struct Args {
    /// HTTP server port
    #[arg(short, long, default_value = "4000")]
    port: u16,

    /// HTTP server host
    #[arg(long, default_value = "127.0.0.1")]
    host: String,

    /// Stalwart API URL
    #[arg(long, env = "STALWART_URL", default_value = "http://localhost:4040")]
    stalwart_url: String,

    /// Stalwart API key
    #[arg(long, env = "STALWART_API_KEY", default_value = "")]
    stalwart_api_key: String,

    /// Webhook secret for signature verification
    #[arg(long, env = "WEBHOOK_SECRET", default_value = "")]
    webhook_secret: String,

    /// Ollama URL (primary instance)
    #[arg(long, env = "OLLAMA_URL", default_value = "http://localhost:11434")]
    ollama_url: String,

    /// Ollama URLs (comma-separated for load balancing)
    #[arg(long, env = "OLLAMA_URLS")]
    ollama_urls: Option<String>,

    /// Use full agent swarm (30+ agents) instead of core (6 agents)
    #[arg(long, env = "FULL_SWARM", default_value = "false")]
    full_swarm: bool,

    /// Log level
    #[arg(long, default_value = "info")]
    log_level: String,

    /// Database URL
    #[arg(long, env = "DATABASE_URL")]
    database_url: Option<String>,

    /// Redis URL
    #[arg(long, env = "REDIS_URL")]
    redis_url: Option<String>,

    // SMTP Configuration
    /// SMTP server host
    #[arg(long, env = "SMTP_HOST")]
    smtp_host: Option<String>,

    /// SMTP server port
    #[arg(long, env = "SMTP_PORT", default_value = "587")]
    smtp_port: u16,

    /// SMTP username
    #[arg(long, env = "SMTP_USERNAME")]
    smtp_username: Option<String>,

    /// SMTP password
    #[arg(long, env = "SMTP_PASSWORD")]
    smtp_password: Option<String>,

    /// Use STARTTLS for SMTP
    #[arg(long, env = "SMTP_STARTTLS", default_value = "true")]
    smtp_starttls: bool,

    /// Default sender email address
    #[arg(long, env = "SMTP_FROM")]
    smtp_from: Option<String>,

    /// Default sender name
    #[arg(long, env = "SMTP_FROM_NAME")]
    smtp_from_name: Option<String>,

    // MCP Server Configuration
    /// MCP Filesystem server URL
    #[arg(long, env = "MCP_FILESYSTEM_URL")]
    mcp_filesystem_url: Option<String>,

    /// MCP Memory server URL
    #[arg(long, env = "MCP_MEMORY_URL")]
    mcp_memory_url: Option<String>,

    /// MCP Sequential Thinking server URL
    #[arg(long, env = "MCP_THINKING_URL")]
    mcp_thinking_url: Option<String>,

    /// MCP Git server URL
    #[arg(long, env = "MCP_GIT_URL")]
    mcp_git_url: Option<String>,

    // Embedded Mail Server Configuration
    /// Enable embedded SMTP server
    #[arg(long, env = "MAIL_ENABLED", default_value = "false")]
    mail_enabled: bool,

    /// SMTP server bind port (inbound mail)
    #[arg(long, env = "MAIL_SMTP_PORT", default_value = "2525")]
    mail_smtp_port: u16,

    /// Mail domain
    #[arg(long, env = "MAIL_DOMAIN", default_value = "localhost")]
    mail_domain: String,

    /// Mail hostname
    #[arg(long, env = "MAIL_HOSTNAME", default_value = "localhost")]
    mail_hostname: String,

    /// Start embedded Stalwart mail server
    #[arg(long, env = "STALWART_ENABLED", default_value = "true")]
    stalwart_enabled: bool,

    /// Stalwart config file path
    #[arg(long, env = "STALWART_CONFIG", default_value = "data/stalwart/config.toml")]
    stalwart_config: String,
}

#[actix_web::main]
async fn main() -> Result<()> {
    // Load .env file if present
    dotenv::dotenv().ok();

    // Parse CLI arguments
    let args = Args::parse();

    // Initialize logging
    let log_level = match args.log_level.to_lowercase().as_str() {
        "trace" => Level::TRACE,
        "debug" => Level::DEBUG,
        "info" => Level::INFO,
        "warn" => Level::WARN,
        "error" => Level::ERROR,
        _ => Level::INFO,
    };

    let subscriber = FmtSubscriber::builder()
        .with_max_level(log_level)
        .with_target(true)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;

    info!("Starting Amail Server v{}", env!("CARGO_PKG_VERSION"));

    // Start Stalwart mail server via Docker if enabled
    if args.stalwart_enabled {
        // Check if Docker is available
        let docker_check = Command::new("docker")
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();

        if docker_check.is_ok() && docker_check.unwrap().success() {
            // Check if stalwart container is already running
            let ps_output = Command::new("docker")
                .args(["ps", "-q", "-f", "name=amail-stalwart"])
                .output();

            let container_running = ps_output
                .map(|o| !o.stdout.is_empty())
                .unwrap_or(false);

            if container_running {
                info!("Stalwart mail server already running (Docker: amail-stalwart)");
            } else {
                // Remove any stopped container with same name
                let _ = Command::new("docker")
                    .args(["rm", "-f", "amail-stalwart"])
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status();

                // Start Stalwart via Docker
                let docker_result = Command::new("docker")
                    .args([
                        "run", "-d",
                        "--name", "amail-stalwart",
                        "-p", "4040:8080",   // HTTP/JMAP
                        "-p", "2525:25",     // SMTP
                        "-p", "1143:143",    // IMAP
                        "-v", "amail-stalwart-data:/opt/stalwart-mail",
                        "-e", "ADMIN_SECRET=amail-internal-secret",
                        "stalwartlabs/mail-server:latest"
                    ])
                    .stdout(Stdio::null())
                    .stderr(Stdio::piped())
                    .status();

                match docker_result {
                    Ok(status) if status.success() => {
                        info!("Stalwart mail server started (Docker: amail-stalwart)");
                        info!("  JMAP/HTTP: http://localhost:4040");
                        info!("  SMTP: localhost:2525");
                        info!("  IMAP: localhost:1143");
                    }
                    Ok(_) => {
                        warn!("Failed to start Stalwart Docker container");
                        warn!("Try manually: docker run -d --name amail-stalwart -p 4040:8080 -p 2525:25 stalwartlabs/mail-server:latest");
                    }
                    Err(e) => {
                        warn!("Docker command failed: {}", e);
                    }
                }
            }
        } else {
            warn!("Docker not found. Install Docker Desktop or disable Stalwart with STALWART_ENABLED=false");
            warn!("Inbox will show empty until Stalwart is running.");
        }
    } else {
        info!("Stalwart mail server disabled (set STALWART_ENABLED=true to enable)");
    }

    // Create task queue channel
    let (task_tx, mut task_rx) = mpsc::channel(1000);

    // Initialize Stalwart client with optional SMTP
    let stalwart = if let (Some(host), Some(username), Some(password)) = 
        (&args.smtp_host, &args.smtp_username, &args.smtp_password) 
    {
        let smtp_config = SmtpConfig {
            host: host.clone(),
            port: args.smtp_port,
            username: username.clone(),
            password: password.clone(),
            starttls: args.smtp_starttls,
            default_from: args.smtp_from.clone().unwrap_or_else(|| username.clone()),
            default_from_name: args.smtp_from_name.clone(),
        };
        let client = StalwartClient::with_smtp(&args.stalwart_url, &args.stalwart_api_key, smtp_config)?;
        info!("Stalwart client configured with SMTP at {}:{}", host, args.smtp_port);
        client
    } else {
        let client = StalwartClient::new(&args.stalwart_url, &args.stalwart_api_key)?;
        info!("Stalwart client configured (REST only, no SMTP)");
        client
    };
    info!("Stalwart REST API: {}", args.stalwart_url);

    // Initialize Stalwart with hostname and domain
    let mail_domain = std::env::var("MAIL_DOMAIN").unwrap_or_else(|_| "localhost".to_string());
    let mail_hostname = format!("mail.{}", mail_domain);
    if let Err(e) = stalwart.initialize(&mail_hostname, &mail_domain).await {
        warn!("Stalwart initialization warning: {}", e);
    }

    // Start agent registry
    let registry = AgentRegistry::new().start();
    info!("Agent registry started");

    // Create agent configurations
    let agent_configs = if args.full_swarm {
        info!("Initializing full agent swarm (30+ agents)");
        create_full_swarm()
    } else {
        info!("Initializing core agent swarm (6 agents)");
        create_core_swarm()
    };

    // Start supervisor with agents
    let mut supervisor = SupervisorActor::new("RootSupervisor", SupervisionStrategy::OneForOne)
        .with_registry(registry.clone());

    for config in agent_configs {
        supervisor.add_child(config);
    }

    let supervisor_addr = supervisor.start();
    info!("Supervisor started with {} agents", if args.full_swarm { "30+" } else { "6" });

    // Create spam filter
    let spam_filter = amail_core::SpamFilter::default();
    info!("Spam filter initialized with default settings");

    // Initialize ConfigManager for JSON-based config (no database required)
    let config_manager = Arc::new(ConfigManager::new("data/config"));
    if let Err(e) = config_manager.load_all().await {
        tracing::warn!("Failed to load config files: {} - using defaults", e);
    } else {
        info!("ConfigManager loaded from data/config/");
    }
    // ConfigState will use Supabase if DATABASE_URL is set, otherwise JSON fallback
    // We'll update this after db is initialized

    // Create app state
    let app_state = web::Data::new(AppState {
        stalwart,
        webhook_secret: args.webhook_secret.clone(),
        task_sender: task_tx,
        spam_filter,
    });

    // Initialize LLM pool (multiple Ollama instances for load balancing)
    let ollama_urls: Vec<String> = if let Some(ref urls) = args.ollama_urls {
        urls.split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    } else {
        vec![args.ollama_url.clone()]
    };
    
    let llm_pool = amail_llm::LlmPool::new(ollama_urls.clone(), "mistral:latest")
        .expect("Failed to create LLM pool");
    info!("LLM pool configured with {} instances: {:?}", llm_pool.instance_count(), ollama_urls);
    
    // Also keep single client for backwards compatibility
    let llm = LlmClient::with_url("mistral:latest", &args.ollama_url)
        .expect("Failed to create LLM client");

    // Initialize MCP client and connect to configured servers
    let mcp = McpClient::new().expect("Failed to create MCP client");
    let mut mcp_servers_connected = 0;
    
    if let Some(ref url) = args.mcp_filesystem_url {
        if let Err(e) = mcp.connect("filesystem", url).await {
            tracing::warn!("Failed to connect to MCP filesystem server: {}", e);
        } else {
            mcp_servers_connected += 1;
        }
    }
    
    if let Some(ref url) = args.mcp_memory_url {
        if let Err(e) = mcp.connect("memory", url).await {
            tracing::warn!("Failed to connect to MCP memory server: {}", e);
        } else {
            mcp_servers_connected += 1;
        }
    }
    
    if let Some(ref url) = args.mcp_thinking_url {
        if let Err(e) = mcp.connect("thinking", url).await {
            tracing::warn!("Failed to connect to MCP thinking server: {}", e);
        } else {
            mcp_servers_connected += 1;
        }
    }
    
    if let Some(ref url) = args.mcp_git_url {
        if let Err(e) = mcp.connect("git", url).await {
            tracing::warn!("Failed to connect to MCP git server: {}", e);
        } else {
            mcp_servers_connected += 1;
        }
    }
    
    let mcp = Arc::new(mcp);
    if mcp_servers_connected > 0 {
        info!("MCP client connected to {} servers", mcp_servers_connected);
        let tools = mcp.all_tools().await;
        info!("MCP tools available: {}", tools.len());
    } else {
        info!("MCP client initialized (no servers configured)");
    }

    // Initialize database (optional)
    let db = if let Some(ref db_url) = args.database_url {
        match Database::connect(db_url).await {
            Ok(db) => {
                info!("Connected to database");
                if let Err(e) = db.migrate().await {
                    tracing::warn!("Migration warning: {}", e);
                }
                Some(db)
            }
            Err(e) => {
                tracing::warn!("Database connection failed: {} - running without persistence", e);
                None
            }
        }
    } else {
        info!("No DATABASE_URL configured - using JSON config files (data/config/)");
        None
    };

    // Create API state for database-backed endpoints (Supabase/PostgreSQL)
    let api_state = if let Some(ref database) = db {
        let stalwart_for_api = amail_core::stalwart::StalwartClient::new(
            &args.stalwart_url,
            if args.stalwart_api_key.is_empty() { None } else { Some(args.stalwart_api_key.clone()) }
        );
        info!("Database API enabled (Supabase/PostgreSQL)");
        Some(web::Data::new(ApiState::new(database.pool().clone(), stalwart_for_api)))
    } else {
        info!("Database API disabled - domains/mailboxes stored in JSON files");
        None
    };

    // Create ConfigState with UnifiedStorage (Supabase primary, JSON fallback)
    let config_state = if let Some(ref database) = db {
        info!("ConfigState using Supabase storage (primary)");
        web::Data::new(ConfigState::with_database(config_manager.clone(), database.pool().clone()))
    } else {
        info!("ConfigState using JSON storage (fallback)");
        web::Data::new(ConfigState::new(config_manager.clone()))
    };

    // Initialize Redis queue (optional)
    let queue = if let Some(ref redis_url) = args.redis_url {
        match TaskQueue::new(redis_url) {
            Ok(q) => {
                info!("Connected to Redis queue");
                Some(q)
            }
            Err(e) => {
                tracing::warn!("Redis connection failed: {} - using in-memory queue", e);
                None
            }
        }
    } else {
        info!("No REDIS_URL configured - using in-memory queue");
        None
    };

    // Create processing pipeline
    let mut pipeline = Pipeline::new(registry.clone(), llm);
    if let Some(db) = db {
        pipeline = pipeline.with_database(db);
    }
    if let Some(q) = queue.clone() {
        pipeline = pipeline.with_queue(q);
    }
    let pipeline = Arc::new(pipeline);

    // Spawn task processor from channel (fallback when no Redis)
    let pipeline_clone = pipeline.clone();
    tokio::spawn(async move {
        while let Some(task) = task_rx.recv().await {
            info!("Processing task from channel: {}", task.id);
            
            // Process via pipeline - ingest the email to classify and route
            if let Some(email) = task.context.source_email.clone() {
                info!("Routing email to agents: {}", email.metadata.subject);
                match pipeline_clone.ingest_email(email).await {
                    Ok(task_id) => {
                        info!("Task {} processed and routed to agent", task_id);
                    }
                    Err(e) => {
                        tracing::error!("Failed to process task: {}", e);
                    }
                }
            } else {
                tracing::warn!("Task {} has no source email", task.id);
            }
        }
    });

    // Spawn queue processor (if Redis configured)
    if queue.is_some() {
        let pipeline_clone = pipeline.clone();
        tokio::spawn(async move {
            if let Err(e) = pipeline_clone.process_queue_loop().await {
                tracing::error!("Queue processor error: {}", e);
            }
        });
    }

    // Initialize dashboard broadcaster for WebSocket
    let dashboard = web::Data::new(DashboardBroadcaster::new());
    info!("Dashboard WebSocket broadcaster initialized");

    // Populate dashboard with initial agent states (before server starts)
    {
        use amail_webhook::{AgentState, AgentStatus, SystemStats};
        let agent_configs_for_dashboard = if args.full_swarm {
            create_full_swarm()
        } else {
            create_core_swarm()
        };
        let agent_count = agent_configs_for_dashboard.len() as u32;
        for config in agent_configs_for_dashboard {
            let agent = AgentState {
                id: config.id.to_string(),
                name: config.name.clone(),
                role: format!("{:?}", config.role),
                status: AgentStatus::Idle,
                active_tasks: 0,
                completed_tasks: 0,
                failed_tasks: 0,
                last_activity: None,
                current_task: None,
            };
            dashboard.update_agent(agent).await;
        }
        // Update stats with agent count
        dashboard.update_stats(SystemStats {
            total_agents: agent_count,
            active_agents: 0,
            ..Default::default()
        }).await;
        info!("Dashboard populated with {} agents", agent_count);
    }

    // Start embedded SMTP server if enabled
    if args.mail_enabled {
        use amail_mail::{MailConfig, SmtpServer, MailEventHandler, events::create_event_channel};
        
        let mail_config = MailConfig {
            smtp_bind: format!("0.0.0.0:{}", args.mail_smtp_port).parse()?,
            hostname: args.mail_hostname.clone(),
            domain: args.mail_domain.clone(),
            webhook_url: Some(format!("http://{}:{}/api/v1/mail/inbound", args.host, args.port)),
            ..Default::default()
        };

        let (event_tx, event_rx) = create_event_channel(1000);
        
        // Start SMTP server
        let smtp_server = SmtpServer::new(mail_config.clone(), event_tx);
        smtp_server.start().await?;
        info!("Embedded SMTP server started on port {}", args.mail_smtp_port);

        // Start event handler
        let webhook_url = mail_config.webhook_url.clone();
        tokio::spawn(async move {
            let handler = MailEventHandler::new(event_rx, webhook_url);
            handler.run().await;
        });
    }

    // Start HTTP server
    let bind_addr = format!("{}:{}", args.host, args.port);
    info!("Starting HTTP server on {}", bind_addr);
    info!("Dashboard WebSocket available at ws://{}:{}/ws/dashboard", args.host, args.port);

    let api_state_clone = api_state.clone();
    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        let mut app = App::new()
            .wrap(cors)
            .app_data(app_state.clone())
            .app_data(dashboard.clone())
            .app_data(config_state.clone())
            .configure(configure_routes)
            .configure(configure_config_routes); // JSON-based API (no database required)
        
        // Only configure database-backed API routes if database is available
        if let Some(ref api_data) = api_state_clone {
            app = app.app_data(api_data.clone()).configure(configure_api_routes);
        }
        
        app
    })
    .bind(&bind_addr)?
    .run()
    .await?;

    info!("Amail Server shutting down");
    Ok(())
}
