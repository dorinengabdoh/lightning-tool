use lightning_satgate::{
    decoder::decode_bolt11_invoice,
    encoder::create_sample_invoice,
};

#[test]
fn test_valid_bolt11_decoding_and_validation() {
    let sats = 2500;
    let memo = "Unit Test Payment";

    let raw_invoice = create_sample_invoice(sats, memo).expect("Should construct sample invoice");
    let details = decode_bolt11_invoice(&raw_invoice).expect("Should decode invoice cleanly");

    assert_eq!(details.network, "Bitcoin Regtest");
    assert_eq!(details.amount_sat, Some(sats));
    assert_eq!(details.description, memo);
    assert!(details.is_signature_valid);
    assert!(!details.is_expired);
}
