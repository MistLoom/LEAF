//! 参考运行时：读配置装配引擎，可选管理面。

use std::sync::Arc;

use clap::Parser;
use frame_compose::{
    access_allowed, build_clis, build_gateway_with, AppBuilder, ComposeOptions, ConfigCatalog,
    ToolHost,
};
use frame_core::AccessKind;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    #[cfg(feature = "port-cli")]
    {
        use frame_port_cli::{
            AbilitiesCmd, CatalogCmd, Cli, CliBuildCmd, CliPort, Commands, ServeCmd,
        };

        let cli = Cli::parse();
        let config_path = cli.config.as_deref();
        let mut opts = load_compose_options(config_path)?;

        if cli.management {
            #[cfg(feature = "mgmt")]
            spawn_management(cli.management_bind, config_path).await?;
            #[cfg(not(feature = "mgmt"))]
            return Err("未启用 mgmt feature".into());
        }

        match cli.command {
            Commands::Abilities { action } => match action {
                AbilitiesCmd::Cli { action } => match action {
                    CliBuildCmd::Build { ability } => {
                        build_clis(ability.as_deref(), "dev")?;
                    }
                },
            },
            other => {
                opts.access_gate = Some(AccessKind::Cli);
                let gateway = build_gateway_with(opts.clone()).await;
                let listed = gateway.catalog();
                let ids: Vec<&str> = listed.iter().map(|e| e.instance.as_str()).collect();

                if !access_allowed(&opts, AccessKind::Cli, &ids) {
                    return Err("当前已 load 能力均未声明 cli 协议".into());
                }

                match other {
                    Commands::Catalog { action } | Commands::Tools { action } => match action {
                        CatalogCmd::List => CliPort::run_catalog_list(gateway.as_ref()).await?,
                        CatalogCmd::Show { target } => {
                            CliPort::run_catalog_show(gateway.as_ref(), &target).await?
                        }
                    },
                    Commands::Invoke {
                        target,
                        args,
                        session,
                    } => {
                        CliPort::run_invoke(gateway.as_ref(), &target, &args, session.as_deref())
                            .await?
                    }
                    Commands::Serve { target } => match target {
                        ServeCmd::Http { bind } => {
                            if !access_allowed(&opts, AccessKind::Http, &ids) {
                                return Err("当前开放能力均未启用 http 接入".into());
                            }
                            serve_http(&opts, &bind).await?
                        }
                        ServeCmd::Mcp => {
                            if !access_allowed(&opts, AccessKind::Mcp, &ids) {
                                return Err("当前开放能力均未启用 mcp 接入".into());
                            }
                            serve_mcp(&opts).await?
                        }
                        ServeCmd::Grpc { bind } => {
                            if !access_allowed(&opts, AccessKind::Grpc, &ids) {
                                return Err("当前开放能力均未声明 grpc 协议".into());
                            }
                            serve_grpc(&opts, &bind).await?
                        }
                    },
                    _ => unreachable!(),
                }
            }
        }
        return Ok(());
    }

    #[cfg(not(feature = "port-cli"))]
    {
        let _ = build_gateway_with(ComposeOptions::default()).await;
        Err("至少启用 port-cli feature 以提供入口".into())
    }
}

fn load_compose_options(config: Option<&str>) -> Result<ComposeOptions, Box<dyn std::error::Error>> {
    let Some(spec) = config else {
        return Ok(ComposeOptions::default());
    };
    let catalog = Arc::new(ConfigCatalog::new());
    let (id, value) = catalog.load_id(None, spec)?;
    let host = if value.get("tool_expose").is_some()
        || value.get("ability_expose").is_some()
        || value.get("capability_expose").is_some()
        || value.get("expose").is_some()
        || value.get("tools").is_some()
        || value.get("abilities").is_some()
        || value.get("capabilities").is_some()
    {
        ToolHost::from_value(&value)?
    } else {
        ToolHost::default()
    };
    Ok(ComposeOptions {
        config: Some(catalog),
        config_id: Some(id),
        host,
        access_gate: None,
        ..ComposeOptions::default()
    })
}

#[cfg(all(feature = "mgmt", feature = "port-cli"))]
async fn spawn_management(
    bind: std::net::SocketAddr,
    config: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    use frame_compose::mgmt::{router, MgmtState};

    let mut builder = AppBuilder::new();
    if let Some(path) = config {
        builder = builder.expose_from_config(path)?;
    }
    let app = builder.bootstrap().await;
    let state = MgmtState::from(app);
    let listener = tokio::net::TcpListener::bind(bind).await?;
    tracing::info!(%bind, "runtime 管理面监听");
    tokio::spawn(async move {
        let _ = axum::serve(listener, router(state)).await;
    });
    Ok(())
}

#[cfg(all(feature = "port-cli", feature = "port-http"))]
async fn serve_http(
    opts: &ComposeOptions,
    bind: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    use frame_core::{DefaultHttpController, Port};
    use tracing::info;

    let mut http_opts = opts.clone();
    http_opts.access_gate = Some(AccessKind::Http);
    let gateway = build_gateway_with(http_opts).await;
    let ctrl = opts
        .host
        .controller(AccessKind::Http)
        .unwrap_or_else(|| Arc::new(DefaultHttpController));
    let addr = bind.parse()?;
    let port: Box<dyn Port> = Box::new(frame_port_http::HttpPort::with_controller(addr, ctrl));
    info!(port = port.name(), %bind, "starting port");
    port.serve(gateway).await?;
    Ok(())
}

#[cfg(all(feature = "port-cli", not(feature = "port-http")))]
async fn serve_http(
    _opts: &ComposeOptions,
    _bind: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    Err("未启用 port-http feature".into())
}

#[cfg(all(feature = "port-cli", feature = "port-mcp"))]
async fn serve_mcp(opts: &ComposeOptions) -> Result<(), Box<dyn std::error::Error>> {
    use frame_core::{DefaultMcpController, Port};
    use tracing::info;

    let mut mcp_opts = opts.clone();
    mcp_opts.access_gate = Some(AccessKind::Mcp);
    let gateway = build_gateway_with(mcp_opts).await;
    let ctrl = opts
        .host
        .controller(AccessKind::Mcp)
        .unwrap_or_else(|| Arc::new(DefaultMcpController));
    let port: Box<dyn Port> = Box::new(frame_port_mcp::McpPort::with_controller(
        "runtime",
        env!("CARGO_PKG_VERSION"),
        ctrl,
    ));
    info!(port = port.name(), "starting port");
    port.serve(gateway).await?;
    Ok(())
}

#[cfg(all(feature = "port-cli", not(feature = "port-mcp")))]
async fn serve_mcp(_opts: &ComposeOptions) -> Result<(), Box<dyn std::error::Error>> {
    Err("未启用 port-mcp feature".into())
}

#[cfg(all(feature = "port-cli", feature = "port-grpc"))]
async fn serve_grpc(opts: &ComposeOptions, bind: &str) -> Result<(), Box<dyn std::error::Error>> {
    use frame_core::{DefaultGrpcController, Port};
    use tracing::info;

    let mut grpc_opts = opts.clone();
    grpc_opts.access_gate = Some(AccessKind::Grpc);
    let gateway = build_gateway_with(grpc_opts).await;
    let ctrl = opts
        .host
        .controller(AccessKind::Grpc)
        .unwrap_or_else(|| Arc::new(DefaultGrpcController));
    let addr = bind.parse()?;
    let port: Box<dyn Port> = Box::new(frame_port_grpc::GrpcPort::with_controller(addr, ctrl));
    info!(port = port.name(), %bind, "starting port");
    port.serve(gateway).await?;
    Ok(())
}

#[cfg(all(feature = "port-cli", not(feature = "port-grpc")))]
async fn serve_grpc(
    _opts: &ComposeOptions,
    _bind: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    Err("未启用 port-grpc feature".into())
}
