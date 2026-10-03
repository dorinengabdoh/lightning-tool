use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::decoder::{decode_bolt11_invoice, Bolt11InvoiceDetails};
use crate::encoder::create_sample_invoice;
use crate::server::AppState;

#[derive(Debug, Deserialize)]
pub struct DecodeRequest {
    pub invoice: String,
}

#[derive(Debug, Deserialize)]
pub struct PayInvoiceRequest {
    pub invoice: String,
    pub channel_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateInvoiceRequest {
    pub amount_sat: u64,
    pub memo: String,
}

#[derive(Debug, Deserialize)]
pub struct OpenChannelRequest {
    pub peer_pubkey: String,
    pub capacity_sat: u64,
    pub local_name: Option<String>,
    pub remote_name: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub message: String,
    pub data: Option<T>,
}

/// POST /api/invoice/decode
pub async fn handle_decode_invoice(
    Json(payload): Json<DecodeRequest>,
) -> (StatusCode, Json<ApiResponse<Bolt11InvoiceDetails>>) {
    match decode_bolt11_invoice(&payload.invoice) {
        Ok(details) => (
            StatusCode::OK,
            Json(ApiResponse {
                success: true,
                message: "Invoice successfully parsed and validated".to_string(),
                data: Some(details),
            }),
        ),
        Err(err) => (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse {
                success: false,
                message: err,
                data: None,
            }),
        ),
    }
}

/// POST /api/invoice/create
pub async fn handle_create_invoice(
    State(state): State<AppState>,
    Json(payload): Json<CreateInvoiceRequest>,
) -> (StatusCode, Json<ApiResponse<serde_json::Value>>) {
    let mut node = state.node.lock().unwrap();
    
    // Try to create a valid signed BOLT11 invoice or fallback to recorded sample
    let invoice_str = match create_sample_invoice(payload.amount_sat, &payload.memo) {
        Ok(inv) => inv,
        Err(_) => {
            let p_hash = node.record_payment(payload.amount_sat, &payload.memo);
            format!("lnbcrt{}u1p{}", payload.amount_sat, &p_hash[..16])
        }
    };

    let payment_hash = node.record_payment(payload.amount_sat, &payload.memo);

    let response_data = serde_json::json!({
        "invoice": invoice_str,
        "payment_hash": payment_hash,
        "amount_sat": payload.amount_sat,
        "memo": payload.memo
    });

    (
        StatusCode::CREATED,
        Json(ApiResponse {
            success: true,
            message: "Lightning BOLT11 invoice created successfully".to_string(),
            data: Some(response_data),
        }),
    )
}

/// POST /api/invoice/pay
pub async fn handle_pay_invoice(
    State(state): State<AppState>,
    Json(payload): Json<PayInvoiceRequest>,
) -> (StatusCode, Json<ApiResponse<serde_json::Value>>) {
    match decode_bolt11_invoice(&payload.invoice) {
        Ok(details) => {
            let mut node = state.node.lock().unwrap();
            let amount = details.amount_sat.unwrap_or(100);
            match node.process_invoice_payment(amount, &details.description, payload.channel_id.as_deref()) {
                Ok(desc) => {
                    let result = serde_json::json!({
                        "settled": true,
                        "preimage": "72fad1824ae23ef61a9cb97949c76ab77179a525b8feb3f5b945f00000000000",
                        "payment_hash": details.payment_hash,
                        "amount_paid_sat": amount,
                        "description": desc
                    });

                    (
                        StatusCode::OK,
                        Json(ApiResponse {
                            success: true,
                            message: format!("Off-chain Lightning payment settled! {}", desc),
                            data: Some(result),
                        }),
                    )
                }
                Err(err) => (
                    StatusCode::BAD_REQUEST,
                    Json(ApiResponse {
                        success: false,
                        message: err,
                        data: None,
                    }),
                )
            }
        }
        Err(err) => (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse {
                success: false,
                message: format!("Failed to pay invoice: {}", err),
                data: None,
            }),
        ),
    }
}

/// POST /api/node/open-channel
pub async fn handle_open_channel(
    State(state): State<AppState>,
    Json(payload): Json<OpenChannelRequest>,
) -> (StatusCode, Json<ApiResponse<serde_json::Value>>) {
    let mut node = state.node.lock().unwrap();
    match node.open_channel(&payload.peer_pubkey, payload.capacity_sat, payload.local_name, payload.remote_name) {
        Ok(channel_id) => (
            StatusCode::OK,
            Json(ApiResponse {
                success: true,
                message: format!("Payment channel opened with peer {}", payload.peer_pubkey),
                data: Some(serde_json::json!({ "channel_id": channel_id, "capacity_sat": payload.capacity_sat })),
            }),
        ),
        Err(err) => (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse {
                success: false,
                message: err,
                data: None,
            }),
        ),
    }
}

#[derive(Debug, Deserialize)]
pub struct FundWalletRequest {
    pub amount_sat: u64,
}

/// POST /api/node/fund
pub async fn handle_fund_wallet(
    State(state): State<AppState>,
    Json(payload): Json<FundWalletRequest>,
) -> (StatusCode, Json<ApiResponse<serde_json::Value>>) {
    let mut node = state.node.lock().unwrap();
    let funding_address = node.fund_onchain_wallet(payload.amount_sat);

    (
        StatusCode::OK,
        Json(ApiResponse {
            success: true,
            message: format!("Funded on-chain wallet with {} satoshis", payload.amount_sat),
            data: Some(serde_json::json!({
                "funding_address": funding_address,
                "amount_funded_sat": payload.amount_sat,
                "new_onchain_balance_sat": node.get_state().onchain_balance_sat
            })),
        }),
    )
}

#[derive(Debug, Deserialize)]
pub struct CloseChannelRequest {
    pub channel_id: String,
}

#[derive(Debug, Deserialize)]
pub struct TransferPeerBalanceRequest {
    pub channel_id: String,
    pub direction: String,
    pub amount_sat: u64,
}

/// POST /api/node/close-channel
pub async fn handle_close_channel(
    State(state): State<AppState>,
    Json(payload): Json<CloseChannelRequest>,
) -> (StatusCode, Json<ApiResponse<serde_json::Value>>) {
    let mut node = state.node.lock().unwrap();
    match node.close_channel(&payload.channel_id) {
        Ok((alice_refund, bob_refund, closing_txid)) => (
            StatusCode::OK,
            Json(ApiResponse {
                success: true,
                message: format!("Channel {} closed successfully on-chain (Cooperative Close)", payload.channel_id),
                data: Some(serde_json::json!({
                    "channel_id": payload.channel_id,
                    "alice_settled_onchain_sat": alice_refund,
                    "bob_settled_onchain_sat": bob_refund,
                    "closing_txid": closing_txid,
                    "new_onchain_balance_sat": node.get_state().onchain_balance_sat
                })),
            }),
        ),
        Err(err) => (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse {
                success: false,
                message: err,
                data: None,
            }),
        ),
    }
}

/// POST /api/node/transfer
pub async fn handle_transfer_peer_balance(
    State(state): State<AppState>,
    Json(payload): Json<TransferPeerBalanceRequest>,
) -> (StatusCode, Json<ApiResponse<crate::node::ChannelDetails>>) {
    let mut node = state.node.lock().unwrap();
    match node.transfer_peer_balance(&payload.channel_id, &payload.direction, payload.amount_sat) {
        Ok(updated_channel) => (
            StatusCode::OK,
            Json(ApiResponse {
                success: true,
                message: format!("Successfully transferred {} sats ({})", payload.amount_sat, payload.direction),
                data: Some(updated_channel),
            }),
        ),
        Err(err) => (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse {
                success: false,
                message: err,
                data: None,
            }),
        ),
    }
}

/// GET /api/node/status
pub async fn handle_node_status(
    State(state): State<AppState>,
) -> (StatusCode, Json<ApiResponse<crate::node::LightningNodeState>>) {
    let node = state.node.lock().unwrap();
    (
        StatusCode::OK,
        Json(ApiResponse {
            success: true,
            message: "Lightning node status retrieved".to_string(),
            data: Some(node.get_state().clone()),
        }),
    )
}

#[derive(Debug, Deserialize)]
pub struct MultiHopPaymentRequest {
    pub sender_name: String,
    pub receiver_name: String,
    pub amount_sat: u64,
    pub routing_fee_sat: Option<u64>,
}

/// POST /api/node/pay-multihop
pub async fn handle_multihop_payment(
    State(state): State<AppState>,
    Json(payload): Json<MultiHopPaymentRequest>,
) -> (StatusCode, Json<ApiResponse<crate::node::HtlcDetails>>) {
    let mut node = state.node.lock().unwrap();
    let fee = payload.routing_fee_sat.unwrap_or(1);
    match node.process_multihop_htlc_payment(&payload.sender_name, &payload.receiver_name, payload.amount_sat, fee) {
        Ok(htlc_details) => (
            StatusCode::OK,
            Json(ApiResponse {
                success: true,
                message: format!("Multi-hop HTLC payment successfully routed from {} to {}!", payload.sender_name, payload.receiver_name),
                data: Some(htlc_details),
            }),
        ),
        Err(err) => (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse {
                success: false,
                message: err,
                data: None,
            }),
        ),
    }
}

#[derive(Debug, Deserialize)]
pub struct ForceCloseRequest {
    pub channel_id: String,
}

#[derive(Debug, Deserialize)]
pub struct SimulateBreachRequest {
    pub channel_id: String,
    pub cheater_is_remote: Option<bool>,
}

/// POST /api/node/force-close
pub async fn handle_force_close(
    State(state): State<AppState>,
    Json(payload): Json<ForceCloseRequest>,
) -> (StatusCode, Json<ApiResponse<crate::node::ForceCloseDetails>>) {
    let mut node = state.node.lock().unwrap();
    match node.force_close_channel(&payload.channel_id) {
        Ok(details) => (
            StatusCode::OK,
            Json(ApiResponse {
                success: true,
                message: format!("Unilateral Force Close triggered on Channel {}. Funds timelocked for 144 blocks.", payload.channel_id),
                data: Some(details),
            }),
        ),
        Err(err) => (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse {
                success: false,
                message: err,
                data: None,
            }),
        ),
    }
}

/// POST /api/node/simulate-breach
pub async fn handle_simulate_breach(
    State(state): State<AppState>,
    Json(payload): Json<SimulateBreachRequest>,
) -> (StatusCode, Json<ApiResponse<crate::node::ForceCloseDetails>>) {
    let mut node = state.node.lock().unwrap();
    let cheater_is_remote = payload.cheater_is_remote.unwrap_or(true);
    match node.simulate_breach_and_justice(&payload.channel_id, cheater_is_remote) {
        Ok(details) => (
            StatusCode::OK,
            Json(ApiResponse {
                success: true,
                message: format!("🚨 BREACH DETECTED & PENALIZED! 100% of Channel {} capacity confiscated via Justice Tx!", payload.channel_id),
                data: Some(details),
            }),
        ),
        Err(err) => (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse {
                success: false,
                message: err,
                data: None,
            }),
        ),
    }
}


