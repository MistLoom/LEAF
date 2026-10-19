mod assembly;
mod config;
mod driver;
mod gateway;
mod persist;
mod runtime;
mod web;

use std::path::PathBuf;

use clap::{Parser, Subcommand};
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(name = "agent-test", about = "LEAF Agent Test 应用")]
struct Cli {
    #[arg(long, global = true)]
    expose: Option<PathBuf>,
    #[arg(long, global = true)]
    provider: Option<PathBuf>,
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// 启动 WebUI 服务（默认）
    Serve {
        #[arg(long, default_value = "127.0.0.1")]
        host: String,
        #[arg(short, long, default_value_t = 8787)]
        port: u16,
    },
    /// 命令行执行单轮任务
    Run {
        #[arg(short = 'p', long)]
        prompt: String,
        #[arg(long)]
        session: Option<String>,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("warn".parse()?))
        .init();

    let cli = Cli::parse();
    let app = match &cli.command {
        Some(Commands::Run { .. }) => {
            assembly::bootstrap(cli.expose.as_deref(), cli.provider.as_deref()).await?
        }
        _ => assembly::bootstrap_web(cli.expose.as_deref(), cli.provider.as_deref()).await?,
    };

    match cli.command.unwrap_or(Commands::Serve {
        host: "127.0.0.1".into(),
        port: 8787,
    }) {
        Commands::Serve { host, port } => {
            web::serve(app, &host, port).await?;
        }
        Commands::Run { prompt, session } => {
            let _ = persist::restore_all(&app.gateway).await;
            let out = gateway::run_turn(&app.gateway, &prompt, session.as_deref()).await?;
            if let Some(sid) = out.get("session_id").and_then(|v| v.as_str()) {
                let _ = persist::save_session(&app.gateway, sid).await;
            }
            let text = out.get("text").and_then(|v| v.as_str()).unwrap_or("");
            let reason = out.get("reason").and_then(|v| v.as_str()).unwrap_or("");
            let rounds = out.get("rounds").and_then(|v| v.as_u64()).unwrap_or(0);
            let session_id = out.get("session_id").and_then(|v| v.as_str()).unwrap_or("");
            eprintln!("session: {session_id}");
            eprintln!("rounds: {rounds} reason: {reason}");
            println!("{text}");
        }
    }

    Ok(())
}
