use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about = "I a just a fancy software", long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub subcmd: CliSubCmd,
}

#[derive(Subcommand)]
pub enum CliSubCmd {
    #[clap(about = "Backup section")]
    Backup(CliBackupSubCmd),
    #[clap(about = "Upload section")]
    Upload(CliUploadSubCmd),
}

#[derive(Args)]
pub struct CliBackupSubCmd {
    #[clap(subcommand)]
    pub subcmd: CliBackupCmd,
}

#[derive(Args)]
pub struct CliUploadSubCmd {
    #[clap(subcommand)]
    pub subcmd: CliUploadCmd,
}

#[derive(Subcommand)]
pub enum CliBackupCmd {
    #[clap(about = "Show section")]
    Show(CliBackup),
    #[clap(about = "Film section")]
    Film(CliBackup),
}

#[derive(Subcommand)]
pub enum CliUploadCmd {
    #[clap(about = "Show section")]
    Show(CliUpload),
    #[clap(about = "Film section")]
    Film(CliUpload),
}

#[derive(Args, Debug)]
pub struct CliBackup {
    /// Title of the show
    pub title: String,

    /// Links of the show
    pub links: Vec<String>,

    /// Language of the show
    #[arg(long, short, default_value_t = String::new())]
    pub languages: String,

    /// Source of the show
    #[arg(long, short, default_value_t = String::new())]
    pub sources: String,
}

#[derive(Args, Debug)]
pub struct CliUpload {
    /// Title of the show
    pub title: String,

    /// Paths of the show
    pub paths: Vec<String>,

    /// Language of the show
    #[arg(long, short, default_value_t = String::new())]
    pub languages: String,

    /// Source of the show
    #[arg(long, short, default_value_t = String::new())]
    pub sources: String,
}
