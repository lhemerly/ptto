use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(
    name = "ptto",
    version,
    about = "Deploy single-binary web apps to a single VPS"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Prepare a target VPS (Caddy/systemd prerequisites)
    Init {
        /// SSH target in user@host format
        target: Option<String>,
        /// Print remote commands instead of executing them
        #[arg(long, default_value_t = false)]
        dry_run: bool,
    },
    /// Build and deploy the app to a server
    Deploy {
        /// Public domain that should route to this deployment
        #[arg(long)]
        domain: Option<String>,
        /// SSH target in user@host format
        #[arg(long)]
        target: Option<String>,
        /// Output path for the compiled Linux amd64 binary
        #[arg(long, default_value = "./app")]
        artifact: String,
        /// Go package/directory to build
        #[arg(long, default_value = ".")]
        source: String,
        /// Print remote commands instead of executing them
        #[arg(long, default_value_t = false)]
        dry_run: bool,
    },
    /// Stream remote service logs
    Logs {
        /// Name of the service to stream
        #[arg(default_value = "ptto-app")]
        service: String,
    },
    /// Generate a deploy key string for CI usage
    GenerateKey,
}
