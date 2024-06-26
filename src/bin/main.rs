use clap::Parser;
use distrib_kv_store::start_example_raft_node;
use tracing_subscriber::EnvFilter;
use tokio::sync::watch;

#[derive(Parser, Clone, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Opt {
    #[clap(long)]
    pub id: u64,

    #[clap(long)]
    pub http_addr: String,

    #[clap(long)]
    pub rpc_addr: String,
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Setup the logger
    tracing_subscriber::fmt()
        .with_target(true)
        .with_thread_ids(true)
        .with_level(true)
        .with_ansi(false)
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    // Parse the parameters passed by arguments.
    let options = Opt::parse();

    let (_shutdown_tx, shutdown_rx) = watch::channel(());

    start_example_raft_node(
        options.id,
        format!("{}-db", options.rpc_addr),
        options.http_addr,
        options.rpc_addr,
        shutdown_rx.clone()
    )
    .await
}
