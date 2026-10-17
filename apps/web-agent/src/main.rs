//! Web 编码代理 CLI（Serve / Doctor / Run）。

use std::path::PathBuf;

use clap::{Parser, Subcommand};
use tracing_subscriber::EnvFilter;
use web_agent::{
    bootstrap_with_options, default_assembly_path, default_plugins_profile_path, diagnose,
    default_provider_path, ecosystem,
};

#[derive(Parser)]
#[command(name = "web-agent", about = "leafEX Web 编码代理（生态 Ability 标杆）")]
struct Cli {
    #[arg(long, global = true, help = "装配 YAML")]
    assembly: Option<PathBuf>,
    #[arg(long, global = true, help = "插件清单 YAML（合并进 abilities）")]
    plugins: Option<PathBuf>,
    #[arg(long, global = true, help = "chat provider YAML")]
    provider: Option<PathBuf>,
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// 启动 Web UI（默认）
    Serve {
        #[arg(long, default_value = "127.0.0.1")]
        host: String,
        #[arg(short, long, default_value_t = 8790)]
        port: u16,
    },
    /// 检查装配与生态 Ability（JSON）
    Doctor,
    /// 单轮命令行任务
    Run {
        #[arg(short = 'p', long)]
        prompt: String,
        #[arg(long)]
        session: Option<String>,
    },
    /// 调用生态钩子（claude_mod / dsh_plugin invoke）
    Invoke {
        #[arg(long, default_value = "claude_mod")]
        ability: String,
        #[arg(long)]
        tool: String,
        #[arg(long, default_value = "{}")]
        args: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("warn".parse()?))
        .init();

    let cli = Cli::parse();
    let default_cmd = Commands::Serve {
        host: "127.0.0.1".into(),
        port: 8790,
    };
    let command = cli.command.unwrap_or(default_cmd);

    let plugins_path: Option<PathBuf> = match &command {
        Commands::Serve { .. } | Commands::Doctor => cli.plugins.clone().or_else(|| {
            let def = default_plugins_profile_path();
            if def.is_file() {
                Some(def)
            } else {
                None
            }
        }),
        _ => cli.plugins.clone(),
    };

    let boot = bootstrap_with_options(
        cli.assembly.as_deref(),
        plugins_path.as_deref(),
        cli.provider.as_deref(),
    )
    .await?;

    match command {
        Commands::Serve { host, port } => {
            web_agent::web::serve(boot.app, &host, port).await?;
        }
        Commands::Doctor => {
            let report = diagnose(&boot).await?;
            println!("{}", serde_json::to_string_pretty(&report)?);
            if cli.assembly.is_none() {
                eprintln!("装配: {}", default_assembly_path().display());
            }
            if cli.plugins.is_none() {
                eprintln!("插件清单（可选）: {}", default_plugins_profile_path().display());
            }
            if cli.provider.is_none() {
                eprintln!("Provider 示例: {}", default_provider_path().display());
            }
        }
        Commands::Run { prompt, session } => {
            let out = web_agent::gateway::run_turn(
                &boot.app.gateway,
                &prompt,
                session.as_deref(),
            )
            .await?;
            println!("{}", serde_json::to_string_pretty(&out)?);
        }
        Commands::Invoke { ability, tool, args } => {
            let args_val: serde_json::Value = serde_json::from_str(&args)
                .map_err(|e| format!("--args 须为 JSON: {e}"))?;
            let out = ecosystem::invoke_hook(
                &boot.app.gateway,
                &boot.app.expose,
                &ability,
                &tool,
                args_val,
            )
            .await?;
            println!("{}", serde_json::to_string_pretty(&out)?);
        }
    }
    Ok(())
}
