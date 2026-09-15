use clap::{Parser, Subcommand};

use frame_compose::build_clis;

#[derive(Debug, Parser)]
#[command(name = "tool-bin", about = "leafEX 能力 CLI 编译")]
struct RootCli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Build {
        #[arg(long)]
        ability: Option<String>,
        #[arg(long, default_value = "dev")]
        profile: String,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = RootCli::parse();
    match root.command {
        Command::Build { ability, profile } => {
            build_clis(ability.as_deref(), &profile)?;
        }
    }
    Ok(())
}
