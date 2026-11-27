use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(
    name = "fovea",
    version,
    about = "Pure-Rust, GPU-native screen recorder and AI demo video studio"
)]
pub struct CliArgs {
    #[command(subcommand)]
    pub command: Option<CliCommand>,
}

#[derive(Debug, Subcommand, Clone, PartialEq, Eq)]
pub enum CliCommand {
    /// Open an existing .fovea project bundle in Fovea Studio.
    Open { path: PathBuf },
    /// Run the live Model Context Protocol (MCP) server for Claude / AGY.
    Mcp,
}
