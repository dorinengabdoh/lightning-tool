use serde::{Deserialize, Serialize};
use std::sync::Arc;
use ldk_node::{Builder, Node};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChannelDetails {
    pub channel_id: String,
    pub peer_pubkey: String,
    pub local_name: String,
    pub remote_name: String,
    pub capacity_sat: u64,
    pub local_balance_sat: u64,
    pub remote_balance_sat: u64,
    pub is_usable: bool,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HtlcHopInfo {
    pub hop_index: usize,
    pub sender: String,
    pub receiver: String,
    pub channel_id: String,
    pub amount_sat: u64,
    pub timelock_blocks: u32,
    pub fee_sat: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HtlcDetails {
    pub htlc_id: String,
    pub payment_hash: String,
    pub preimage: String,
    pub amount_sat: u64,
    pub total_routing_fee_sat: u64,
    pub hops: Vec<HtlcHopInfo>,
    pub status: String,
    pub timestamp: String,
    pub log_trace: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ForceCloseDetails {
    pub channel_id: String,
    pub close_type: String,
    pub cheater_name: String,
    pub honest_party_name: String,
    pub revoked_state_number: u64,
    pub actual_state_number: u64,
    pub attempted_fraud_amount_sat: u64,
    pub confiscated_penalty_sat: u64,
    pub revocation_secret: String,
    pub justice_txid: String,
    pub csv_timelock_blocks: u32,
    pub status: String,
    pub timestamp: String,
    pub log_trace: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PaymentRecord {
    pub payment_hash: String,
    pub amount_sat: u64,
    pub description: String,
    pub settled: bool,
    pub timestamp: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LightningNodeState {
    pub node_id: String,
    pub network: String,
    pub onchain_balance_sat: u64,
    pub total_local_balance_sat: u64,
    pub total_remote_balance_sat: u64,
    pub channels: Vec<ChannelDetails>,
    pub payment_history: Vec<PaymentRecord>,
    pub is_real_ldk_node: bool,
}

pub struct LightningNodeManager {
    state: LightningNodeState,
    ldk_node: Option<Arc<Node>>,
}

impl LightningNodeManager {
    pub fn new(network: &str) -> Self {
        // Attempt to load or initialize real LDK Node if storage path environment variable is set
        if let Ok(data_dir) = std::env::var("LDK_DATA_DIR") {
            if let Ok(manager) = Self::try_init_ldk_node(&data_dir, network) {
                tracing::info!("🚀 Successfully initialized authentic LDK Node via ldk-node v0.7.0!");
                return manager;
            }
        }

        let node_id = "03a341905901edc09674b3fc64d8892f3a61f38e6e968417c80521e4b3e6d87e01".to_string();
        
        let sample_channels = vec![
            ChannelDetails {
                channel_id: "800000x1x0".to_string(),
                peer_pubkey: "02e7f61c28c8b417e901a88b56012a97ef15c0e1a2d399c4b789123456789abcde".to_string(),
                local_name: "Merchant (Alice)".to_string(),
                remote_name: "Customer (Bob)".to_string(),
                capacity_sat: 1_000_000,
                local_balance_sat: 600_000,
                remote_balance_sat: 400_000,
                is_usable: true,
                status: "ACTIVE".to_string(),
            },
            ChannelDetails {
                channel_id: "800001x2x1".to_string(),
                peer_pubkey: "03456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123".to_string(),
                local_name: "Merchant (Alice)".to_string(),
                remote_name: "Client (Charlie)".to_string(),
                capacity_sat: 500_000,
                local_balance_sat: 250_000,
                remote_balance_sat: 250_000,
                is_usable: true,
                status: "ACTIVE".to_string(),
            }
        ];

        let history = vec![
            PaymentRecord {
                payment_hash: "72fad1824ae23ef61a9cb97949c76ab77179a525b8feb3f5b945f00000000000".to_string(),
                amount_sat: 10,
                description: "AI Prompt Token Charge".to_string(),
                settled: true,
                timestamp: "2026-09-29 10:00:00 UTC".to_string(),
            },
            PaymentRecord {
                payment_hash: "ab93dfbbe9fa8bcf54e172fad1824ae23ef61a9cb97949c76ab77179a525b8fe".to_string(),
                amount_sat: 50,
                description: "Merchant Expresso Payment".to_string(),
                settled: true,
                timestamp: "2026-09-29 10:15:00 UTC".to_string(),
            }
        ];

        Self {
            state: LightningNodeState {
                node_id,
                network: network.to_string(),
                onchain_balance_sat: 250_000,
                total_local_balance_sat: 850_000,
                total_remote_balance_sat: 650_000,
                channels: sample_channels,
                payment_history: history,
                is_real_ldk_node: false,
            },
            ldk_node: None,
        }
    }

    /// Initializes a real LDK node instance backed by `ldk-node` crate v0.7.0
    pub fn try_init_ldk_node(data_dir: &str, network_str: &str) -> anyhow::Result<Self> {
        let mut builder = Builder::new();
        builder.set_storage_dir_path(data_dir.to_string());

        let ldk_network = match network_str.to_lowercase().as_str() {
            "mainnet" | "bitcoin" => ldk_node::bitcoin::Network::Bitcoin,
            "testnet" => ldk_node::bitcoin::Network::Testnet,
            "signet" => ldk_node::bitcoin::Network::Signet,
            _ => ldk_node::bitcoin::Network::Regtest,
        };
        builder.set_network(ldk_network);

        let node = builder.build()?;
        node.start()?;
        let node_id = node.node_id().to_string();

        let ldk_arc = Arc::new(node);

        Ok(Self {
            state: LightningNodeState {
                node_id,
                network: network_str.to_string(),
                onchain_balance_sat: 0,
                total_local_balance_sat: 0,
                total_remote_balance_sat: 0,
                channels: Vec::new(),
                payment_history: Vec::new(),
                is_real_ldk_node: true,
            },
            ldk_node: Some(ldk_arc),
        })
    }

    pub fn get_state(&self) -> &LightningNodeState {
        &self.state
    }

    pub fn fund_onchain_wallet(&mut self, sats: u64) -> String {
        self.state.onchain_balance_sat += sats;
        "bcrt1qmockfundingaddressregtest0000000000000000".to_string()
    }

    pub fn open_channel(
        &mut self,
        peer_pubkey: &str,
        capacity_sat: u64,
        local_name: Option<String>,
        remote_name: Option<String>,
    ) -> Result<String, String> {
        if self.state.onchain_balance_sat < capacity_sat {
            return Err(format!(
                "Solde Bitcoin On-Chain insuffisant ({} sats disponibles sur la blockchain). Impossible d'ouvrir un canal de {} satoshis. Veuillez d'abord approvisionner votre portefeuille On-Chain.",
                self.state.onchain_balance_sat, capacity_sat
            ));
        }

        let l_name = local_name.filter(|s| !s.trim().is_empty()).unwrap_or_else(|| "Merchant (Alice)".to_string());
        let r_name = remote_name.filter(|s| !s.trim().is_empty()).unwrap_or_else(|| "Customer (Bob)".to_string());

        self.state.onchain_balance_sat = self.state.onchain_balance_sat.saturating_sub(capacity_sat);
        let channel_id = format!("80000{}x{}x0", self.state.channels.len() + 2, self.state.channels.len());

        let local_balance_sat = capacity_sat / 2;
        let remote_balance_sat = capacity_sat - local_balance_sat;

        self.state.channels.push(ChannelDetails {
            channel_id: channel_id.clone(),
            peer_pubkey: peer_pubkey.to_string(),
            local_name: l_name,
            remote_name: r_name,
            capacity_sat,
            local_balance_sat,
            remote_balance_sat,
            is_usable: true,
            status: "ACTIVE".to_string(),
        });

        self.state.total_local_balance_sat = self.state.channels.iter().filter(|c| c.status == "ACTIVE").map(|c| c.local_balance_sat).sum();
        self.state.total_remote_balance_sat = self.state.channels.iter().filter(|c| c.status == "ACTIVE").map(|c| c.remote_balance_sat).sum();

        Ok(channel_id)
    }

    pub fn record_payment(&mut self, amount_sat: u64, description: &str) -> String {
        let now_nanos = chrono::Utc::now().timestamp_nanos_opt().unwrap_or(123456789);
        let payment_hash = format!("{:016x}{:016x}{:016x}{:016x}", now_nanos, now_nanos ^ 0xDEADBEEF, now_nanos, amount_sat);
        
        self.state.payment_history.push(PaymentRecord {
            payment_hash: payment_hash.clone(),
            amount_sat,
            description: description.to_string(),
            settled: true,
            timestamp: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
        });

        payment_hash
    }

    /// Processes an off-chain invoice payment, deducting from buyer (remote) balance and crediting merchant (local) balance for a channel
    pub fn process_invoice_payment(&mut self, amount_sat: u64, _description: &str, channel_id: Option<&str>) -> Result<String, String> {
        let target_chan = if let Some(cid) = channel_id {
            self.state.channels.iter_mut().find(|c| c.channel_id == cid && c.status == "ACTIVE")
                .ok_or_else(|| format!("Active channel ID {} not found", cid))?
        } else {
            let pos = self.state.channels.iter().position(|c| c.status == "ACTIVE" && c.remote_balance_sat >= amount_sat)
                .or_else(|| self.state.channels.iter().position(|c| c.status == "ACTIVE"))
                .ok_or_else(|| "No active Lightning channel available for payment.".to_string())?;
            &mut self.state.channels[pos]
        };

        if target_chan.remote_balance_sat < amount_sat {
            return Err(format!(
                "Insufficient remote balance for buyer {} ({} sats available, payment requires {} sats).",
                target_chan.remote_name, target_chan.remote_balance_sat, amount_sat
            ));
        }

        target_chan.remote_balance_sat -= amount_sat;
        target_chan.local_balance_sat += amount_sat;

        let cid = target_chan.channel_id.clone();
        let l_name = target_chan.local_name.clone();
        let r_name = target_chan.remote_name.clone();

        self.state.total_local_balance_sat = self.state.channels.iter().filter(|c| c.status == "ACTIVE").map(|c| c.local_balance_sat).sum();
        self.state.total_remote_balance_sat = self.state.channels.iter().filter(|c| c.status == "ACTIVE").map(|c| c.remote_balance_sat).sum();

        let full_desc = format!("POS Purchase [Channel #{}] : {} paid {} sats to {}", cid, r_name, amount_sat, l_name);
        self.record_payment(amount_sat, &full_desc);

        Ok(full_desc)
    }

    /// Executes a Cooperative Close on a channel, settling remaining balances back on-chain
    pub fn close_channel(&mut self, channel_id: &str) -> Result<(u64, u64, String), String> {
        let chan = self.state.channels.iter_mut().find(|c| c.channel_id == channel_id)
            .ok_or_else(|| format!("Channel ID {} not found", channel_id))?;

        if chan.status == "CLOSED" {
            return Err(format!("Channel {} is already closed.", channel_id));
        }

        let alice_refund = chan.local_balance_sat;
        let bob_refund = chan.remote_balance_sat;

        chan.status = "CLOSED".to_string();
        chan.is_usable = false;

        // Cooperative settlement: Local balance goes back to on-chain wallet
        self.state.onchain_balance_sat += alice_refund;
        self.state.total_local_balance_sat = self.state.channels.iter().filter(|c| c.status == "ACTIVE").map(|c| c.local_balance_sat).sum();
        self.state.total_remote_balance_sat = self.state.channels.iter().filter(|c| c.status == "ACTIVE").map(|c| c.remote_balance_sat).sum();

        let closing_txid = format!("00000000000000000000{:016x}{:016x}", alice_refund, bob_refund);

        let desc = format!("Cooperative Close Channel {}: Refunded {} sats to On-Chain", channel_id, alice_refund);
        let now_nanos = chrono::Utc::now().timestamp_nanos_opt().unwrap_or(123456789);
        let payment_hash = format!("{:016x}{:016x}{:016x}{:016x}", now_nanos, now_nanos ^ 0xDEADBEEF, now_nanos, alice_refund);
        
        self.state.payment_history.push(PaymentRecord {
            payment_hash,
            amount_sat: alice_refund,
            description: desc,
            settled: true,
            timestamp: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
        });

        Ok((alice_refund, bob_refund, closing_txid))
    }

    /// Dynamically shifts satoshis between Local and Remote nodes in a specific active channel
    pub fn transfer_peer_balance(&mut self, channel_id: &str, direction: &str, amount_sat: u64) -> Result<ChannelDetails, String> {
        let updated_chan = {
            let chan = self.state.channels.iter_mut().find(|c| c.channel_id == channel_id)
                .ok_or_else(|| format!("Channel ID {} not found", channel_id))?;

            if chan.status != "ACTIVE" {
                return Err(format!("Channel {} is closed. Transfers are disabled.", channel_id));
            }

            if direction == "alice_to_bob" {
                if chan.local_balance_sat < amount_sat {
                    return Err(format!("{} has insufficient local balance ({} sats) to send {} sats to {}", chan.local_name, chan.local_balance_sat, amount_sat, chan.remote_name));
                }
                chan.local_balance_sat -= amount_sat;
                chan.remote_balance_sat += amount_sat;
            } else if direction == "bob_to_alice" {
                if chan.remote_balance_sat < amount_sat {
                    return Err(format!("{} has insufficient remote balance ({} sats) to send {} sats to {}", chan.remote_name, chan.remote_balance_sat, amount_sat, chan.local_name));
                }
                chan.remote_balance_sat -= amount_sat;
                chan.local_balance_sat += amount_sat;
            } else {
                return Err("Invalid direction. Must be 'alice_to_bob' or 'bob_to_alice'".to_string());
            }

            chan.clone()
        };

        // Recalculate totals
        self.state.total_local_balance_sat = self.state.channels.iter().filter(|c| c.status == "ACTIVE").map(|c| c.local_balance_sat).sum();
        self.state.total_remote_balance_sat = self.state.channels.iter().filter(|c| c.status == "ACTIVE").map(|c| c.remote_balance_sat).sum();

        let desc = if direction == "alice_to_bob" {
            format!("Direct Payment: {} ➔ {} ({} sats)", updated_chan.local_name, updated_chan.remote_name, amount_sat)
        } else {
            format!("Direct Payment: {} ➔ {} ({} sats)", updated_chan.remote_name, updated_chan.local_name, amount_sat)
        };
        self.record_payment(amount_sat, &desc);

        Ok(updated_chan)
    }

    /// Executes a multi-hop HTLC payment routed through intermediate relay node (e.g. Alice ➔ Bob ➔ Charlie)
    pub fn process_multihop_htlc_payment(
        &mut self,
        sender_name: &str,
        receiver_name: &str,
        amount_sat: u64,
        routing_fee_sat: u64,
    ) -> Result<HtlcDetails, String> {
        let active_indices: Vec<usize> = self.state.channels.iter().enumerate()
            .filter(|(_, c)| c.status == "ACTIVE")
            .map(|(i, _)| i)
            .collect();

        if active_indices.len() < 2 {
            return Err("Multi-hop HTLC routing requires at least 2 ACTIVE payment channels in the network.".to_string());
        }

        let hop1_idx = active_indices[0];
        let hop2_idx = active_indices[1];

        let total_hop1 = amount_sat + routing_fee_sat;

        // Verify Hop 1 sender balance
        let chan1 = &self.state.channels[hop1_idx];
        let hop1_remote_is_sender = chan1.remote_name.to_lowercase().contains(&sender_name.to_lowercase())
            || !chan1.local_name.to_lowercase().contains(&sender_name.to_lowercase());

        if hop1_remote_is_sender {
            if self.state.channels[hop1_idx].remote_balance_sat < total_hop1 {
                return Err(format!("Hop 1 Sender {} has insufficient balance ({} sats available, required {} sats).", self.state.channels[hop1_idx].remote_name, self.state.channels[hop1_idx].remote_balance_sat, total_hop1));
            }
            self.state.channels[hop1_idx].remote_balance_sat -= total_hop1;
            self.state.channels[hop1_idx].local_balance_sat += total_hop1;
        } else {
            if self.state.channels[hop1_idx].local_balance_sat < total_hop1 {
                return Err(format!("Hop 1 Sender {} has insufficient balance ({} sats available, required {} sats).", self.state.channels[hop1_idx].local_name, self.state.channels[hop1_idx].local_balance_sat, total_hop1));
            }
            self.state.channels[hop1_idx].local_balance_sat -= total_hop1;
            self.state.channels[hop1_idx].remote_balance_sat += total_hop1;
        }

        // Verify Hop 2 relay liquidity
        let chan2 = &self.state.channels[hop2_idx];
        let hop2_remote_is_receiver = chan2.remote_name.to_lowercase().contains(&receiver_name.to_lowercase())
            || !chan2.local_name.to_lowercase().contains(&receiver_name.to_lowercase());

        if hop2_remote_is_receiver {
            if self.state.channels[hop2_idx].local_balance_sat < amount_sat {
                return Err(format!("Hop 2 Relay node has insufficient channel liquidity ({} sats available, required {} sats).", self.state.channels[hop2_idx].local_balance_sat, amount_sat));
            }
            self.state.channels[hop2_idx].local_balance_sat -= amount_sat;
            self.state.channels[hop2_idx].remote_balance_sat += amount_sat;
        } else {
            if self.state.channels[hop2_idx].remote_balance_sat < amount_sat {
                return Err(format!("Hop 2 Relay node has insufficient channel liquidity ({} sats available, required {} sats).", self.state.channels[hop2_idx].remote_balance_sat, amount_sat));
            }
            self.state.channels[hop2_idx].remote_balance_sat -= amount_sat;
            self.state.channels[hop2_idx].local_balance_sat += amount_sat;
        }

        // Recalculate totals
        self.state.total_local_balance_sat = self.state.channels.iter().filter(|c| c.status == "ACTIVE").map(|c| c.local_balance_sat).sum();
        self.state.total_remote_balance_sat = self.state.channels.iter().filter(|c| c.status == "ACTIVE").map(|c| c.remote_balance_sat).sum();

        let now = chrono::Utc::now();
        let nanos = now.timestamp_nanos_opt().unwrap_or(999888777);
        let preimage = format!("72fad1824ae23ef61a9cb97949c76ab{:08x}{:08x}", nanos as u32, amount_sat as u32);
        let payment_hash = format!("ab93dfbbe9fa8bcf54e172fad1824ae{:08x}{:08x}", (nanos ^ 0xDEADBEEF) as u32, routing_fee_sat as u32);

        let hop1_sender = if hop1_remote_is_sender { self.state.channels[hop1_idx].remote_name.clone() } else { self.state.channels[hop1_idx].local_name.clone() };
        let hop1_relay = if hop1_remote_is_sender { self.state.channels[hop1_idx].local_name.clone() } else { self.state.channels[hop1_idx].remote_name.clone() };
        let hop2_receiver = if hop2_remote_is_receiver { self.state.channels[hop2_idx].remote_name.clone() } else { self.state.channels[hop2_idx].local_name.clone() };

        let hop1_info = HtlcHopInfo {
            hop_index: 1,
            sender: hop1_sender.clone(),
            receiver: hop1_relay.clone(),
            channel_id: self.state.channels[hop1_idx].channel_id.clone(),
            amount_sat: total_hop1,
            timelock_blocks: 144,
            fee_sat: routing_fee_sat,
        };

        let hop2_info = HtlcHopInfo {
            hop_index: 2,
            sender: hop1_relay.clone(),
            receiver: hop2_receiver.clone(),
            channel_id: self.state.channels[hop2_idx].channel_id.clone(),
            amount_sat,
            timelock_blocks: 72,
            fee_sat: 0,
        };

        let log_trace = vec![
            format!("🔒 Step 1: Receiver ({}) generates Secret Preimage R & Payment Hash H = SHA256(R)", hop2_receiver),
            format!("⚡ Step 2 [Hop 1: {} ➔ {}]: Lock {} sats (Amount {} + Fee {}) on Channel {} (CLTV Timelock T1 = 144 blocks)", hop1_sender, hop1_relay, total_hop1, amount_sat, routing_fee_sat, self.state.channels[hop1_idx].channel_id),
            format!("⚡ Step 3 [Hop 2: {} ➔ {}]: Lock {} sats on Channel {} (CLTV Timelock T2 = 72 blocks)", hop1_relay, hop2_receiver, amount_sat, self.state.channels[hop2_idx].channel_id),
            format!("🔑 Step 4: {} presents Preimage R ({}) to claim {} sats", hop2_receiver, &preimage[..16], amount_sat),
            format!("✅ Step 5: Backward Resolution complete! Relay ({}) earned {} sat routing fee!", hop1_relay, routing_fee_sat),
        ];

        let htlc_details = HtlcDetails {
            htlc_id: format!("htlc_{:08x}", nanos as u32),
            payment_hash: payment_hash.clone(),
            preimage: preimage.clone(),
            amount_sat,
            total_routing_fee_sat: routing_fee_sat,
            hops: vec![hop1_info, hop2_info],
            status: "SETTLED".to_string(),
            timestamp: now.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
            log_trace,
        };

        let desc = format!("Multi-Hop HTLC Routing: {} ➔ {} ➔ {} ({} sats + {} sat relay fee)", hop1_sender, hop1_relay, hop2_receiver, amount_sat, routing_fee_sat);
        self.record_payment(amount_sat, &desc);

        Ok(htlc_details)
    }

    /// Executes an honest Unilateral Force Close on a channel (locks closer funds for CSV 144 blocks)
    pub fn force_close_channel(&mut self, channel_id: &str) -> Result<ForceCloseDetails, String> {
        let (local_name, remote_name, local_val, remote_val) = {
            let chan = self.state.channels.iter_mut().find(|c| c.channel_id == channel_id)
                .ok_or_else(|| format!("Channel ID {} not found", channel_id))?;

            if chan.status == "CLOSED" || chan.status == "BREACH_PENALIZED" {
                return Err(format!("Channel {} is already closed ({})", channel_id, chan.status));
            }

            chan.status = "FORCE_CLOSED".to_string();
            chan.is_usable = false;
            (chan.local_name.clone(), chan.remote_name.clone(), chan.local_balance_sat, chan.remote_balance_sat)
        };

        let now = chrono::Utc::now();
        let nanos = now.timestamp_nanos_opt().unwrap_or(11223344);

        // In Unilateral Force Close, local node's balance is locked for CSV timelock (144 blocks)
        // Remote node's balance can be claimed immediately
        self.state.onchain_balance_sat += remote_val;
        self.state.total_local_balance_sat = self.state.channels.iter().filter(|c| c.status == "ACTIVE").map(|c| c.local_balance_sat).sum();
        self.state.total_remote_balance_sat = self.state.channels.iter().filter(|c| c.status == "ACTIVE").map(|c| c.remote_balance_sat).sum();

        let force_txid = format!("fc{:016x}{:016x}{:016x}{:016x}", nanos, nanos ^ 0x00FF00FF, local_val, remote_val);

        let log_trace = vec![
            format!("⚠️ Step 1: Unilateral Force Close triggered on Channel {}", channel_id),
            format!("🔒 Step 2: Local party ({}) output locked under CSV 144 blocks timelock ({} sats)", local_name, local_val),
            format!("⚡ Step 3: Remote party ({}) output ({} sats) returned to On-Chain balance", remote_name, remote_val),
            format!("📜 Step 4: Force Close Tx published to Bitcoin mempool: {}", &force_txid[..24]),
        ];

        let details = ForceCloseDetails {
            channel_id: channel_id.to_string(),
            close_type: "UNILATERAL_FORCE_CLOSE".to_string(),
            cheater_name: "None".to_string(),
            honest_party_name: local_name,
            revoked_state_number: 0,
            actual_state_number: 1,
            attempted_fraud_amount_sat: 0,
            confiscated_penalty_sat: 0,
            revocation_secret: "None (Honest Unilateral Close)".to_string(),
            justice_txid: force_txid.clone(),
            csv_timelock_blocks: 144,
            status: "TIMELOCKED".to_string(),
            timestamp: now.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
            log_trace,
        };

        let desc = format!("Unilateral Force Close Channel {}: Timelocked {} sats for 144 blocks", channel_id, local_val);
        self.record_payment(local_val, &desc);

        Ok(details)
    }

    /// Simulates a Fraudulent Force Close (Breach Attempt) where a malicious peer publishes a revoked commitment state.
    /// The honest node extracts the Revocation Secret Key and publishes a Justice Transaction, confiscating 100% of the channel capacity!
    pub fn simulate_breach_and_justice(&mut self, channel_id: &str, cheater_is_remote: bool) -> Result<ForceCloseDetails, String> {
        let (channel_id_str, capacity, cheater_name, honest_name) = {
            let chan = self.state.channels.iter_mut().find(|c| c.channel_id == channel_id)
                .ok_or_else(|| format!("Channel ID {} not found", channel_id))?;

            if chan.status == "CLOSED" || chan.status == "BREACH_PENALIZED" {
                return Err(format!("Channel {} is already closed ({})", channel_id, chan.status));
            }

            chan.status = "BREACH_PENALIZED".to_string();
            chan.is_usable = false;

            let (cheater_n, honest_n) = if cheater_is_remote {
                (chan.remote_name.clone(), chan.local_name.clone())
            } else {
                (chan.local_name.clone(), chan.remote_name.clone())
            };

            let capacity = chan.capacity_sat;
            chan.local_balance_sat = capacity; // 100% confiscated to local node
            chan.remote_balance_sat = 0;

            (chan.channel_id.clone(), capacity, cheater_n, honest_n)
        };

        // Transfer 100% channel capacity to honest node's On-Chain wallet as Justice Penalty!
        self.state.onchain_balance_sat += capacity;
        self.state.total_local_balance_sat = self.state.channels.iter().filter(|c| c.status == "ACTIVE").map(|c| c.local_balance_sat).sum();
        self.state.total_remote_balance_sat = self.state.channels.iter().filter(|c| c.status == "ACTIVE").map(|c| c.remote_balance_sat).sum();

        let now = chrono::Utc::now();
        let nanos = now.timestamp_nanos_opt().unwrap_or(99887766);
        let revocation_secret = format!("rev_sec_{:016x}{:016x}", nanos ^ 0xCAFEBABE, capacity);
        let justice_txid = format!("justice_tx_{:016x}{:016x}", nanos, nanos ^ 0xDEADBEEF);

        let attempted_fraud = capacity * 80 / 100; // Fraudulent claim attempt

        let log_trace = vec![
            format!("🚨 ALERT: Breach Detected! Cheater ({}) broadcasted Revoked Commitment State #1 (attempting to steal {} sats)", cheater_name, attempted_fraud),
            format!("🔑 Watchtower / Honest Node ({}) matched Revocation Secret: {}", honest_name, &revocation_secret[..24]),
            format!("⚖️ Justice Transaction Constructed! Sweeping 100% of Channel Capacity ({} sats) before CSV 144 blocks timelock expires!", capacity),
            format!("⚡ Justice Tx Broadcasted to Bitcoin Network: {}", justice_txid),
            format!("✅ PENALTY EXECUTED: 100% of channel capacity ({} sats) confiscated to {}'s On-Chain Wallet!", capacity, honest_name),
        ];

        let details = ForceCloseDetails {
            channel_id: channel_id_str.clone(),
            close_type: "BREACH_JUSTICE_PENALTY".to_string(),
            cheater_name,
            honest_party_name: honest_name,
            revoked_state_number: 1,
            actual_state_number: 4,
            attempted_fraud_amount_sat: attempted_fraud,
            confiscated_penalty_sat: capacity,
            revocation_secret,
            justice_txid,
            csv_timelock_blocks: 144,
            status: "REVOKED_AND_SWEPT".to_string(),
            timestamp: now.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
            log_trace,
        };

        let record_desc = format!("⚖️ Justice Tx Penalty on Channel {}: Confiscated 100% capacity ({} sats) to On-Chain", channel_id_str, capacity);
        self.record_payment(capacity, &record_desc);

        Ok(details)
    }
}


