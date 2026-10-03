use lightning_invoice::{Currency, InvoiceBuilder, PaymentSecret};
use secp256k1::{SecretKey, Secp256k1};
use std::time::SystemTime;
use bitcoin::hashes::Hash;

/// Generates a valid signed BOLT11 Lightning Invoice for testing
pub fn create_sample_invoice(amount_sat: u64, memo: &str) -> Result<String, String> {
    let secp = Secp256k1::new();
    let private_key = SecretKey::from_slice(&[42u8; 32])
        .map_err(|e| format!("Failed to generate secret key: {:?}", e))?;

    let now = SystemTime::now();

    let payment_hash_bytes = [170u8; 32];
    let payment_secret_bytes = [99u8; 32];

    let invoice = InvoiceBuilder::new(Currency::Regtest)
        .amount_milli_satoshis(amount_sat * 1000)
        .description(memo.to_string())
        .timestamp(now)
        .payee_pub_key(secp256k1::PublicKey::from_secret_key(&secp, &private_key))
        .payment_hash(bitcoin::hashes::sha256::Hash::from_slice(&payment_hash_bytes).unwrap())
        .payment_secret(PaymentSecret(payment_secret_bytes))
        .min_final_cltv_expiry_delta(18)
        .expiry_time(std::time::Duration::from_secs(3600))
        .build_signed(|hash| secp.sign_ecdsa_recoverable(hash, &private_key))
        .map_err(|e| format!("Failed to sign BOLT11 invoice: {:?}", e))?;

    Ok(invoice.to_string())
}
