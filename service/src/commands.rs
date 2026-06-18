use clap::Subcommand;

#[derive(Debug, Clone, Subcommand)]
pub enum Commands {
    /// Seeds the database with the initial data
    Seed,
}
