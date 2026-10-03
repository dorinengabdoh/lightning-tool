use colored::Colorize;
use comfy_table::{presets::UTF8_FULL, Cell, Color, Table};
use indicatif::{ProgressBar, ProgressStyle};

use crate::decoder::Bolt11InvoiceDetails;
use crate::node::LightningNodeState;

pub fn print_invoice_details(details: &Bolt11InvoiceDetails) {
    println!("\n⚡ {}", "DETAILED BOLT11 INVOICE ANALYSIS".bold().yellow());
    println!("{}", "============================================================".dimmed());

    let mut table = Table::new();
    table.load_preset(UTF8_FULL);

    table.add_row(vec![
        Cell::new("Network").fg(Color::Cyan),
        Cell::new(&details.network),
    ]);
    table.add_row(vec![
        Cell::new("Amount (sat)").fg(Color::Yellow),
        Cell::new(
            &details
                .amount_sat
                .map(|a| format!("{} sats ({} msats)", a, details.amount_msat.unwrap_or(0)))
                .unwrap_or_else(|| "Any Amount / Unspecified".to_string()),
        ),
    ]);
    table.add_row(vec![
        Cell::new("Description").fg(Color::White),
        Cell::new(&details.description),
    ]);
    table.add_row(vec![
        Cell::new("Status").fg(Color::Magenta),
        Cell::new(if details.is_expired {
            "EXPIRED ❌".red().to_string()
        } else {
            "VALID ✅".green().to_string()
        }),
    ]);
    table.add_row(vec![
        Cell::new("Created At").fg(Color::Blue),
        Cell::new(&details.timestamp_readable),
    ]);
    table.add_row(vec![
        Cell::new("Expires At").fg(Color::Red),
        Cell::new(&details.expires_at_readable),
    ]);
    table.add_row(vec![
        Cell::new("Payment Hash").fg(Color::Cyan),
        Cell::new(&details.payment_hash),
    ]);
    table.add_row(vec![
        Cell::new("Payee Pubkey").fg(Color::Green),
        Cell::new(&details.payee_pubkey),
    ]);

    let sig_status = if details.is_signature_valid {
        "Valid ECDSA Signature (secp256k1) ✅".green().to_string()
    } else {
        "INVALID SIGNATURE ❌".red().to_string()
    };
    table.add_row(vec![Cell::new("Signature Validation").fg(Color::Yellow), Cell::new(sig_status)]);

    println!("{}", table);

    if !details.route_hints.is_empty() {
        println!("\n🛣️  {}", "ROUTE HINTS DETECTED".bold().cyan());
        for (i, route) in details.route_hints.iter().enumerate() {
            println!("  Route Hint #{}:", i + 1);
            for hop in route {
                println!(
                    "    -> Hop Node: {} | Channel: {} | Base Fee: {} msat | CLTV: {}",
                    hop.src_node_id.cyan(),
                    hop.short_channel_id,
                    hop.base_fee_msat,
                    hop.cltv_expiry_delta
                );
            }
        }
    }
}

pub fn print_node_status(state: &LightningNodeState) {
    println!("\n⚡ {}", "LIGHTNING NODE & CHANNEL STATUS".bold().yellow());
    println!("{}", "============================================================".dimmed());

    let mut overview = Table::new();
    overview.load_preset(UTF8_FULL);

    overview.add_row(vec![Cell::new("Node Public Key"), Cell::new(&state.node_id)]);
    overview.add_row(vec![Cell::new("Network"), Cell::new(&state.network)]);
    overview.add_row(vec![
        Cell::new("On-Chain Balance"),
        Cell::new(format!("{} sats", state.onchain_balance_sat)),
    ]);
    overview.add_row(vec![
        Cell::new("Total Local Capacity"),
        Cell::new(format!("{} sats", state.total_local_balance_sat)),
    ]);
    overview.add_row(vec![
        Cell::new("Total Remote Capacity"),
        Cell::new(format!("{} sats", state.total_remote_balance_sat)),
    ]);
    overview.add_row(vec![
        Cell::new("Active Channels"),
        Cell::new(state.channels.len().to_string()),
    ]);

    println!("{}", overview);

    if !state.channels.is_empty() {
        println!("\n📊 {}", "ACTIVE PAYMENT CHANNELS".bold().green());
        let mut chan_table = Table::new();
        chan_table.load_preset(UTF8_FULL);
        chan_table.set_header(vec![
            "Channel ID",
            "Peer Pubkey",
            "Capacity (sat)",
            "Local Balance",
            "Remote Balance",
            "Usable",
        ]);

        for chan in &state.channels {
            chan_table.add_row(vec![
                Cell::new(&chan.channel_id),
                Cell::new(format!("{}...", &chan.peer_pubkey[..16])),
                Cell::new(chan.capacity_sat.to_string()),
                Cell::new(chan.local_balance_sat.to_string()).fg(Color::Green),
                Cell::new(chan.remote_balance_sat.to_string()).fg(Color::Yellow),
                Cell::new(if chan.is_usable { "YES ✅" } else { "NO ❌" }),
            ]);
        }
        println!("{}", chan_table);
    }
}

pub fn show_progress_bar(msg: &str) {
    let pb = ProgressBar::new(100);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {msg}")
            .unwrap()
            .progress_chars("#>-"),
    );
    pb.set_message(msg.to_string());
    for _ in 0..100 {
        pb.inc(1);
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    pb.finish_with_message("Done! ✅");
}
