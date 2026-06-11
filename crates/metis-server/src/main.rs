//! `metis` — the Metis 3.0 server binary and admin CLI.

use std::net::SocketAddr;
use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};
use metis_server::config::ServerConfig;
use metis_server::{admin, build_state, mcp, serve};

#[derive(Parser)]
#[command(name = "metis", version, about = "Metis 3.0 server")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Run the HTTP server.
    Serve(ServeArgs),
    /// Run the MCP server on stdio (for AI agents).
    Mcp(McpArgs),
    /// Administrative commands run directly against the database (bootstrap).
    #[command(subcommand)]
    Admin(AdminCommand),
}

#[derive(Args)]
struct McpArgs {
    /// Local solo mode: SQLite, implicit single user, no token required.
    #[arg(long)]
    local: bool,
    /// Database URL (env: DATABASE_URL). Required in team mode.
    #[arg(long)]
    database_url: Option<String>,
    /// Store item bodies as files under this root (team mode; default db-blob).
    #[arg(long)]
    object_root: Option<PathBuf>,
    /// Bearer token to act as (team mode; attribution is recorded from it).
    #[arg(long, env = "METIS_TOKEN")]
    token: Option<String>,
}

#[derive(Args)]
struct ServeArgs {
    /// Local solo mode: SQLite, localhost-only, auth disabled.
    #[arg(long)]
    local: bool,
    /// Database URL (env: DATABASE_URL). Required in team mode.
    #[arg(long)]
    database_url: Option<String>,
    /// Address to bind (env: METIS_BIND).
    #[arg(long)]
    bind: Option<SocketAddr>,
    /// Store item bodies as files under this root (team mode; default db-blob).
    #[arg(long)]
    object_root: Option<PathBuf>,
}

#[derive(Subcommand)]
enum AdminCommand {
    /// Create a user.
    CreateUser(CreateUserArgs),
    /// Create a token for a user. Prints the plaintext token once.
    CreateToken(CreateTokenArgs),
}

#[derive(Args)]
struct CreateUserArgs {
    /// Username (unique).
    username: String,
    /// Display name (defaults to the username).
    #[arg(long)]
    display_name: Option<String>,
    /// Email.
    #[arg(long)]
    email: Option<String>,
    /// Grant admin.
    #[arg(long)]
    admin: bool,
    /// Database URL (env: DATABASE_URL).
    #[arg(long, env = "DATABASE_URL")]
    database_url: String,
}

#[derive(Args)]
struct CreateTokenArgs {
    /// Username to issue the token for.
    #[arg(long)]
    user: String,
    /// A label for the token.
    #[arg(long, default_value = "default")]
    name: String,
    /// Tie the token to a named agent (attribution: "user via agent").
    #[arg(long)]
    agent_name: Option<String>,
    /// Database URL (env: DATABASE_URL).
    #[arg(long, env = "DATABASE_URL")]
    database_url: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    let cli = Cli::parse();
    match cli.command {
        Command::Serve(args) => {
            let config = if args.local {
                ServerConfig::local(args.database_url, args.bind)
            } else {
                ServerConfig::team(args.database_url, args.bind, args.object_root)?
            };
            serve(config).await
        }
        Command::Mcp(args) => {
            let config = if args.local {
                ServerConfig::local(args.database_url, None)
            } else {
                ServerConfig::team(args.database_url, None, args.object_root)?
            };
            let app = build_state(config)?;
            let state = mcp::mcp_state_from(&app, args.token)?;
            mcp::run_stdio(state).await
        }
        Command::Admin(AdminCommand::CreateUser(args)) => {
            let display = args.display_name.unwrap_or_else(|| args.username.clone());
            let id = admin::create_user(
                &args.database_url,
                &args.username,
                &display,
                args.email.as_deref(),
                args.admin,
            )?;
            println!("created user {} ({id})", args.username);
            Ok(())
        }
        Command::Admin(AdminCommand::CreateToken(args)) => {
            let token = admin::create_token(
                &args.database_url,
                &args.user,
                &args.name,
                args.agent_name.as_deref(),
            )?;
            println!("token for {} (shown once):\n{token}", args.user);
            Ok(())
        }
    }
}
