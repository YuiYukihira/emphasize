use std::{
    future::Future,
    io::Write,
    path::{Path, PathBuf},
    pin::Pin,
    str::FromStr,
    sync::atomic::AtomicBool,
    task::Poll,
};

use clap::{ArgGroup, Parser, Subcommand};
use clytia::Clytia;
use git2::Repository;
use tokio::task::JoinHandle;

use crate::config::{Config, OperatingMode};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
pub struct Cli {
    /// Specify location of a custom config file
    #[clap(short, long, parse(from_os_str), value_name = "FILE")]
    pub config: Option<PathBuf>,

    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Gets the current contents of the remote deployment
    Fetch,
    /// Pushes your changes to the remote deployment
    Update,
    /// Runs the local development server
    Start,
    /// Creates a new local development setup
    Setup,
}

impl Cli {
    pub async fn run(&self) -> crate::Result<()> {
        match self.command {
            Command::Fetch => self.fetch().await,
            Command::Update => self.update().await,
            Command::Start => self.start().await,
            Command::Setup => self.setup().await,
        }
    }

    async fn fetch(&self) -> crate::Result<()> {
        todo!()
    }

    async fn update(&self) -> crate::Result<()> {
        todo!()
    }

    async fn start(&self) -> crate::Result<()> {
        let config_file = self
            .config
            .as_deref()
            .unwrap_or_else(|| Path::new(".emphasize/config.yml"));
        crate::run_local_server(config_file).await
    }

    async fn setup(&self) -> crate::Result<()> {
        let mut cli = Clytia::default();
        let config_file: PathBuf = match self.config.clone() {
            Some(p) => p,
            None => {
                cli.parsed_input::<_, PathBufWrapper>(
                    "Where do you want to store your config file?",
                    Some(Path::new(".emphasize/config.yml").to_path_buf().into()),
                )?
                .0
            }
        };
        let cache_dir: PathBuf = cli
            .parsed_input::<_, PathBufWrapper>(
                "Where should your cache directory go?",
                Some(Path::new(".emphasize/cache").to_path_buf().into()),
            )?
            .0;
        let db: PathBuf = cli
            .parsed_input::<_, PathBufWrapper>(
                "Where should your database file go?",
                Some(Path::new(".emphasize/content.db").to_path_buf().into()),
            )?
            .0;
        let content_dir: PathBuf = cli
            .parsed_input::<_, PathBufWrapper>(
                "Where should your content directory go?",
                Some(Path::new("blog").to_path_buf().into()),
            )?
            .0;
        let config = Config::new(
            cache_dir.clone(),
            db,
            true,
            content_dir.clone(),
            OperatingMode::ReadWrite,
        );
        cli.static_background_spinner::<_, _, _, eyre::Report>("Writing config file", || {
            if let Some(dir) = config_file.parent() {
                std::fs::create_dir_all(dir)?;
            }
            config.save(&config_file)?;

            Ok(())
        })??;
        cli.static_background_spinner("Creating cache directory", || {
            std::fs::create_dir_all(&cache_dir)
        })??;
        cli.static_background_spinner::<_, _, _, eyre::Report>(
            "Creating content directories",
            || {
                let dirs = ["content", "static", "sass", "templates"]
                    .iter()
                    .map(|&s| content_dir.join(s));
                for dir in dirs {
                    std::fs::create_dir_all(dir)?;
                }

                Ok(())
            },
        )??;
        println!("All done!");

        Ok(())
    }
}

struct PathBufWrapper(PathBuf);

impl From<PathBuf> for PathBufWrapper {
    fn from(path: PathBuf) -> Self {
        Self(path)
    }
}

impl std::fmt::Display for PathBufWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.display())
    }
}

impl FromStr for PathBufWrapper {
    type Err = eyre::Report;

    fn from_str(s: &str) -> crate::Result<Self> {
        Ok(Self(PathBuf::from_str(s)?))
    }
}

impl From<PathBufWrapper> for PathBuf {
    fn from(wrapper: PathBufWrapper) -> Self {
        wrapper.0
    }
}
