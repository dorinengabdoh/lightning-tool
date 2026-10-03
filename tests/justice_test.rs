use lightning_satgate::node::LightningNodeManager;

#[test]
fn test_breach_and_justice_penalty_transaction() {
    let mut manager = LightningNodeManager::new("regtest");
    let initial_onchain = manager.get_state().onchain_balance_sat;

    let target_channel_id = "800000x1x0"; // Capacity: 1,000,000 sats
    let capacity = 1_000_000;

    // Trigger Breach Simulation (Remote customer Bob tries to publish old revoked state #1)
    let result = manager.simulate_breach_and_justice(target_channel_id, true);
    assert!(result.is_ok(), "Breach simulation & Justice Tx failed: {:?}", result.err());

    let details = result.unwrap();
    assert_eq!(details.close_type, "BREACH_JUSTICE_PENALTY");
    assert_eq!(details.confiscated_penalty_sat, capacity);
    assert_eq!(details.revoked_state_number, 1);
    assert_eq!(details.status, "REVOKED_AND_SWEPT");
    assert!(details.revocation_secret.starts_with("rev_sec_"));
    assert!(details.justice_txid.starts_with("justice_tx_"));

    // Verify 100% of capacity was swept into On-Chain wallet balance as penalty
    let new_onchain = manager.get_state().onchain_balance_sat;
    assert_eq!(new_onchain, initial_onchain + capacity);

    // Verify channel status is BREACH_PENALIZED
    let chan = manager.get_state().channels.iter().find(|c| c.channel_id == target_channel_id).unwrap();
    assert_eq!(chan.status, "BREACH_PENALIZED");
    assert!(!chan.is_usable);
}

#[test]
fn test_unilateral_force_close() {
    let mut manager = LightningNodeManager::new("regtest");
    let target_channel_id = "800001x2x1";

    let result = manager.force_close_channel(target_channel_id);
    assert!(result.is_ok());

    let details = result.unwrap();
    assert_eq!(details.close_type, "UNILATERAL_FORCE_CLOSE");
    assert_eq!(details.csv_timelock_blocks, 144);
    assert_eq!(details.status, "TIMELOCKED");

    let chan = manager.get_state().channels.iter().find(|c| c.channel_id == target_channel_id).unwrap();
    assert_eq!(chan.status, "FORCE_CLOSED");
}
