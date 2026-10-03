use lightning_satgate::node::LightningNodeManager;

#[test]
fn test_multihop_htlc_routing() {
    let mut manager = LightningNodeManager::new("regtest");
    
    // Default initial state has 2 channels:
    // Channel 1 (800000x1x0): Merchant (Alice) / Customer (Bob), balances: local=600k, remote=400k
    // Channel 2 (800001x2x1): Merchant (Alice) / Client (Charlie), balances: local=250k, remote=250k

    let sender = "Customer (Bob)";
    let receiver = "Client (Charlie)";
    let amount_sat = 5_000;
    let routing_fee_sat = 5;

    let result = manager.process_multihop_htlc_payment(sender, receiver, amount_sat, routing_fee_sat);
    assert!(result.is_ok(), "Multi-hop HTLC routing failed: {:?}", result.err());

    let htlc = result.unwrap();
    assert_eq!(htlc.amount_sat, 5_000);
    assert_eq!(htlc.total_routing_fee_sat, 5);
    assert_eq!(htlc.hops.len(), 2);
    assert_eq!(htlc.status, "SETTLED");
    assert_eq!(htlc.hops[0].timelock_blocks, 144);
    assert_eq!(htlc.hops[1].timelock_blocks, 72);
    assert!(htlc.preimage.starts_with("72fad"));
}
