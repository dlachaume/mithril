//! Commands for the Cardano db v2 artifact
mod download;
mod list;
mod show;

pub use download::*;
pub use list::*;
pub use show::*;

use crate::CommandContext;
use clap::Subcommand;
use mithril_client::MithrilResult;

/// Cardano db v2 management (alias: cdbv2)
#[derive(Subcommand, Debug, Clone)]
#[command(about = "[unstable] Cardano db v2 management (alias: cdbv2)")]
pub enum CardanoDbV2Commands {
    /// Cardano db snapshot v2 commands
    #[clap(subcommand)]
    Snapshot(CardanoDbV2SnapshotCommands),

    /// Download a Cardano db v2 snapshot to restore a partial Cardano database.
    #[clap(arg_required_else_help = true)]
    Download(CardanoDbV2DownloadCommand),
}

/// Cardano db v2 snapshots
#[derive(Subcommand, Debug, Clone)]
pub enum CardanoDbV2SnapshotCommands {
    /// List available Cardano db v2 snapshots
    #[clap(arg_required_else_help = false)]
    List(CardanoDbListCommand),

    /// Show detailed information about a Cardano db v2 snapshot
    #[clap(arg_required_else_help = true)]
    Show(CardanoDbShowCommand),
}

impl CardanoDbV2Commands {
    /// Execute Cardano db v2 command
    pub async fn execute(&self, config_builder: CommandContext) -> MithrilResult<()> {
        match self {
            Self::Snapshot(cmd) => cmd.execute(config_builder).await,
            Self::Download(cmd) => cmd.execute(config_builder).await,
        }
    }

    /// Is JSON output enabled
    pub fn is_json_output_enabled(&self) -> bool {
        match self {
            Self::Download(cmd) => cmd.is_json_output_enabled(),
            Self::Snapshot(cmd) => cmd.is_json_output_enabled(),
        }
    }
}

impl CardanoDbV2SnapshotCommands {
    /// Execute Cardano db v2 snapshot command
    pub async fn execute(&self, config_builder: CommandContext) -> MithrilResult<()> {
        match self {
            Self::List(cmd) => cmd.execute(config_builder).await,
            Self::Show(cmd) => cmd.execute(config_builder).await,
        }
    }

    /// Is JSON output enabled
    pub fn is_json_output_enabled(&self) -> bool {
        match self {
            CardanoDbV2SnapshotCommands::List(cmd) => cmd.is_json_output_enabled(),
            CardanoDbV2SnapshotCommands::Show(cmd) => cmd.is_json_output_enabled(),
        }
    }
}
