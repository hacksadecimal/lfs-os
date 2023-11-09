use clap::{Parser, Subcommand, ValueEnum};
use std::sync::Arc;

use object_store::{
    aws::AmazonS3Builder, gcp::GoogleCloudStorageBuilder, local::LocalFileSystem, ObjectStore,
};
use tracing::{debug, subscriber::set_global_default, Level};

mod error;
mod protocol;
mod service;

#[tokio::main]
async fn main() {
    let opts = Opts::from_args();

    let subscriber = tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_max_level(if opts.debug() {
            Level::DEBUG
        } else {
            Level::ERROR
        })
        .finish();
    set_global_default(subscriber).expect("Unable to instrument logging");

    debug!("Received options: {opts:?}");

    if let Some(_local) = opts.setup() {
        debug!("Starting setup");
    } else {
        debug!("Starting lfs");
        let object_store = create_os_client(opts.provider(), opts.uri()).unwrap();
        service::serve(object_store).await;
    }
}

fn create_os_client(
    provider: Option<Provider>,
    uri: Option<&String>,
) -> Result<Arc<dyn ObjectStore>, crate::error::Error> {
    match provider {
        Some(Provider::Local) => Ok(Arc::new(LocalFileSystem::new_with_prefix(
            uri.ok_or_else(|| {
                error::Error::InvalidConfiguration(String::from(
                    "Local LFS storage selected but no path was provided",
                ))
            })?,
        )?)),
        Some(Provider::Gcp) => {
            let mut builder = GoogleCloudStorageBuilder::from_env();
            if let Some(uri) = uri {
                builder = builder.with_url(uri);
            }
            Ok(Arc::new(builder.build()?))
        }
        Some(Provider::Aws) => {
            let mut builder = AmazonS3Builder::from_env();
            if let Some(uri) = uri {
                builder = builder.with_url(uri);
            }
            Ok(Arc::new(builder.build()?))
        }
        _ => unimplemented!(),
    }
}

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Provider {
    Aws,
    Gcp,
    Azure,
    S3Compat,
    Local,
}

#[derive(Subcommand, Debug, Clone, Copy)]
pub enum Commands {
    /// Set up the current repository or the global git config for LFS-OS
    Setup {
        #[arg(short, long)]
        local: bool,
    },
}

/// Object store backend for git-lfs
#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about,
    after_help = "NOTE: This binary shouldn't be run manually. It is invoked by git-lfs"
)]
pub struct Opts {
    /// Enable debug output
    #[arg(short, long)]
    debug: bool,

    /// Storage provider to use.
    #[arg(short, long, value_name = "PROVIDER", value_enum)]
    provider: Option<Provider>,

    // URL/Bucket path to store files
    #[arg(short, long)]
    uri: Option<String>,

    /// Configure object store to use with git-lfs
    #[command(subcommand)]
    command: Option<Commands>,
}

impl Opts {
    pub fn from_args() -> Self {
        Self::parse()
    }

    pub fn debug(&self) -> bool {
        self.debug
    }

    pub fn setup(&self) -> Option<bool> {
        self.command
            .map(|cmd| matches!(cmd, Commands::Setup { local: true }))
    }

    pub fn provider(&self) -> Option<Provider> {
        self.provider
    }

    pub fn uri(&self) -> Option<&String> {
        self.uri.as_ref()
    }
}
