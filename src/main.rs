use clap::{Parser, Subcommand};
use dotenvy::dotenv;
use tracing_subscriber::EnvFilter;

use lightning_tool::{
    cli::{print_invoice_details, print_node_status, show_progress_bar},
    decoder::decode_bolt11_invoice,
    encoder::create_sample_invoice,
    node::LightningNodeManager,
    server::start_web_server,
};

#[derive(Parser)]
#[command(name = "lightning-tool")]
#[command(author = "Student Developer")]
#[command(version = "0.1.0")]
#[command(about = "Lightning SatGate Terminal - Terminal de Paiement Lightning Marchand & IA, Décodeur BOLT11 et Nœud LDK en Rust", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Decode and validate a BOLT11 invoice string
    Decode {
        /// Raw BOLT11 invoice string (bech32 format, e.g. lnbc... or lntb...)
        invoice: String,
    },
    /// Show current Lightning node status, wallet balances, and active channels
    NodeStatus,
    /// Generate a signed sample BOLT11 invoice for testing
    SampleInvoice {
        /// Amount in satoshis
        #[arg(short, long, default_value_t = 1000)]
        sats: u64,

        /// Memo / description
        #[arg(short, long, default_value = "Test Payment")]
        memo: String,
    },
    /// Start the Axum HTTP REST API Server & Web Dashboard UI
    Serve {
        /// HTTP Server Port
        #[arg(short, long, default_value_t = 3000)]
        port: u16,

        /// Bitcoin Network (regtest, signet, testnet, mainnet)
        #[arg(short, long, default_value = "regtest")]
        network: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("lightning_tool=info".parse()?))
        .init();

    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Decode { invoice }) => {
            show_progress_bar("Parsing & Validating BOLT11 Invoice Signature...");
            match decode_bolt11_invoice(invoice) {
                Ok(details) => print_invoice_details(&details),
                Err(e) => eprintln!("❌ Decoding Error: {}", e),
            }
        }
        Some(Commands::NodeStatus) => {
            let manager = LightningNodeManager::new("regtest");
            print_node_status(manager.get_state());
        }
        Some(Commands::SampleInvoice { sats, memo }) => {
            show_progress_bar("Building and Signing BOLT11 Invoice (secp256k1)...");
            match create_sample_invoice(*sats, memo) {
                Ok(inv) => {
                    println!("\n✅ Generated Valid BOLT11 Invoice:");
                    println!("{}", inv);
                }
                Err(e) => eprintln!("❌ Failed to build invoice: {}", e),
            }
        }
        Some(Commands::Serve { port, network }) => {
            start_web_server(*port, network.clone()).await?;
        }
        None => {
            start_web_server(3000, "regtest".to_string()).await?;
        }
    }

    Ok(())
}
