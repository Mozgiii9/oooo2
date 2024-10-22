mod args;
mod balance;
mod benchmark;
mod busses;
mod claim;
mod close;
mod config;
mod cu_limits;
#[cfg(feature = "admin")]
mod initialize;
mod mine;
mod open;
mod rewards;
mod send_and_confirm;
mod stake;
mod upgrade;
mod utils;

use std::sync::Arc;

use args::*;
use clap::{command, Parser, Subcommand};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    signature::{read_keypair_file, Keypair},
};

struct Miner {
    pub keypair_filepath: Option<String>,
    pub priority_fee: u64,
    pub rpc_client: Arc<RpcClient>,
    pub exec_rpc_client: Arc<RpcClient>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    #[command(about = "Fetch an account balance")]
    Balance(BalanceArgs),

    #[command(about = "Benchmark your hashpower")]
    Benchmark(BenchmarkArgs),

    #[command(about = "Fetch the bus account balances")]
    Busses(BussesArgs),

    #[command(about = "Claim your mining rewards")]
    Claim(ClaimArgs),

    #[command(about = "Close your account to recover rent")]
    Close(CloseArgs),

    #[command(about = "Fetch the program config")]
    Config(ConfigArgs),

    #[command(about = "Start mining")]
    Mine(MineArgs),

    #[command(about = "Fetch the current reward rate for each difficulty level")]
    Rewards(RewardsArgs),

    #[command(about = "Stake to earn a rewards multiplier")]
    Stake(StakeArgs),

    #[command(about = "Upgrade your ORE tokens from v1 to v2")]
    Upgrade(UpgradeArgs),

    #[cfg(feature = "admin")]
    #[command(about = "Initialize the program")]
    Initialize(InitializeArgs),
}

#[derive(Parser, Debug)]
#[command(about, version)]
struct Args {
    #[arg(
        long,
        value_name = "NETWORK_URL",
        help = "Network address of your RPC provider",
        global = true
    )]
    rpc: Option<String>,
    
    #[arg(
        long,
        value_name = "EXEC_NETWORK_URL",
        help = "Network address of your execution RPC provider",
        global = true
    )]
    exec_rpc: Option<String>,

    #[clap(
        global = true,
        short = 'C',
        long = "config",
        id = "PATH",
        help = "Filepath to config file."
    )]
    config_file: Option<String>,

    #[arg(
        long,
        value_name = "KEYPAIR_FILEPATH",
        help = "Filepath to keypair to use",
        global = true
    )]
    keypair: Option<String>,

    #[arg(
        long,
        value_name = "MICROLAMPORTS",
        help = "Number of microlamports to pay as priority fee per transaction",
        default_value = "0",
        global = true
    )]
    priority_fee: u64,

    #[command(subcommand)]
    command: Commands,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    // Load the config file from custom path, the default path, or use default config values
    let cli_config = if let Some(config_file) = &args.config_file {
        solana_cli_config::Config::load(config_file).unwrap_or_else(|_| {
            eprintln!("error: Could not find config file `{}`", config_file);
            std::process::exit(1);
        })
    } else if let Some(config_file) = &*solana_cli_config::CONFIG_FILE {
        solana_cli_config::Config::load(config_file).unwrap_or_default()
    } else {
        solana_cli_config::Config::default()
    };

    // Initialize miner.
    let cluster = args.rpc.unwrap_or(cli_config.json_rpc_url.clone());
    let exec_cluster=args.exec_rpc.unwrap_or(cli_config.json_rpc_url.clone());
    let default_keypair = args.keypair.unwrap_or(cli_config.keypair_path);
    let rpc_client = RpcClient::new_with_commitment(cluster, CommitmentConfig::confirmed());
    let exec_rpc_client=RpcClient::new_with_commitment(exec_cluster, CommitmentConfig::confirmed());
    let miner = Arc::new(Miner::new(
        Arc::new(rpc_client),
        Arc::new(exec_rpc_client),
        args.priority_fee,
        Some(default_keypair),
    ));

    // Execute user command.
    match args.command {
        Commands::Balance(args) => {
            miner.balance(args).await;
        }
        Commands::Benchmark(args) => {
            miner.benchmark(args).await;
        }
        Commands::Busses(_) => {
            miner.busses().await;
        }
        Commands::Claim(args) => {
            miner.claim(args).await;
        }
        Commands::Close(_) => {
            miner.close().await;
        }
        Commands::Config(_) => {
            miner.config().await;
        }
        Commands::Mine(args) => {
            miner.mine(args).await;
        }
        Commands::Rewards(_) => {
            miner.rewards().await;
        }
        Commands::Stake(args) => {
            miner.stake(args).await;
        }
        Commands::Upgrade(args) => {
            miner.upgrade(args).await;
        }
        #[cfg(feature = "admin")]
        Commands::Initialize(_) => {
            miner.initialize().await;
        }
    }
}

impl Miner {
    pub fn new(
        rpc_client: Arc<RpcClient>,
        exec_rpc_client: Arc<RpcClient>,
        priority_fee: u64,
        keypair_filepath: Option<String>,
    ) -> Self {
        Self {
            rpc_client,
            exec_rpc_client,
            keypair_filepath,
            priority_fee,
        }
    }

    pub fn signer(&self) -> Keypair {
        match self.keypair_filepath.clone() {
            Some(filepath) => read_keypair_file(filepath.clone())
                .expect(format!("No keypair found at {}", filepath).as_str()),
            None => panic!("No keypair provided"),
        }
    }
}
