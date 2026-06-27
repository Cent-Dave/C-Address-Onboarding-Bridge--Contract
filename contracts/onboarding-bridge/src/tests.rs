use crate::{BridgeError, OnboardingBridge};

use soroban_sdk::{
    contract, contractimpl, contracttype,
    testutils::{Address as _, Events, Ledger},
    Address, Bytes, BytesN, Env, IntoVal, Vec,
};

fn register_all_contracts(env: &Env) -> (Address, Address) {
    let bridge_id = env.register(OnboardingBridge, ());
    let token_id = env.register(TestToken, ());
    env.mock_all_auths();
    (bridge_id, token_id)
}

fn init_token(env: &Env, token_id: &Address, admin: &Address) {
    let token = TestTokenClient::new(env, token_id);
    token.initialize(admin, &7u32, &"Test".into_val(env), &"TST".into_val(env));
}

fn create_bridge_client<'a>(
    env: &'a Env,
    bridge_id: &Address,
) -> crate::OnboardingBridgeClient<'a> {
    crate::OnboardingBridgeClient::new(env, bridge_id)
}

fn create_test_users(env: &Env) -> (Address, Address, Address) {
    let admin = Address::generate(env);
    let user = Address::generate(env);
    let fee_collector = Address::generate(env);
    (admin, user, fee_collector)
}

fn mint_tokens(env: &Env, token_id: &Address, to: &Address, amount: i128) {
    let token = TestTokenClient::new(env, token_id);
    token.mint(to, &amount);
}

fn check_balance(env: &Env, token_id: &Address, addr: &Address) -> i128 {
    let token = TestTokenClient::new(env, token_id);
    token.balance(addr)
}

#[test]
fn test_initialize() {
    let env = Env::default();
    let (admin, _user, fee_collector) = create_test_users(&env);
    let (bridge_id, _) = register_all_contracts(&env);
    let bridge = create_bridge_client(&env, &bridge_id);

    bridge.initialize(&admin, &fee_collector, &50u32);

    assert_eq!(bridge.query_fee_bps(), 50u32);
    assert_eq!(bridge.query_fee_collector(), fee_collector);
    assert_eq!(bridge.query_admin(), admin);
    assert!(bridge.query_is_initialized());
}

#[test]
fn test_initialize_twice() {
    let env = Env::default();
    let (admin, _user, fee_collector) = create_test_users(&env);
    let (bridge_id, _) = register_all_contracts(&env);
    let bridge = create_bridge_client(&env, &bridge_id);

    bridge.initialize(&admin, &fee_collector, &50u32);
    assert_eq!(
        bridge.try_initialize(&admin, &fee_collector, &50u32),
        Err(Ok(BridgeError::AlreadyInitialized))
    );
}

#[test]
fn test_initialize_fee_too_high() {
    let env = Env::default();
    let (admin, _user, fee_collector) = create_test_users(&env);
    let (bridge_id, _) = register_all_contracts(&env);
    let bridge = create_bridge_client(&env, &bridge_id);

    assert_eq!(
        bridge.try_initialize(&admin, &fee_collector, &2000u32),
        Err(Ok(BridgeError::FeeTooHigh))
    );
}

#[test]
fn test_fund_c_address() {
    let env = Env::default();
    let (admin, user, fee_collector) = create_test_users(&env);
    let (bridge_id, token_id) = register_all_contracts(&env);
    let bridge = create_bridge_client(&env, &bridge_id);
    init_token(&env, &token_id, &admin);

    bridge.initialize(&admin, &fee_collector, &100u32);
    bridge.add_asset(&token_id);
    mint_tokens(&env, &token_id, &user, 1000i128);

    let target = Address::generate(&env);
    bridge.fund_c_address(&user, &target, &token_id, &500i128);

    assert_eq!(check_balance(&env, &token_id, &user), 500i128);
    assert_eq!(check_balance(&env, &token_id, &target), 495i128);
    assert_eq!(check_balance(&env, &token_id, &fee_collector), 0i128);
    assert_eq!(check_balance(&env, &token_id, &bridge_id), 5i128);
}

#[test]
fn test_fund_without_initialize() {
    let env = Env::default();
    let (_admin, user, _fee_collector) = create_test_users(&env);
    let (bridge_id, token_id) = register_all_contracts(&env);
    let bridge = create_bridge_client(&env, &bridge_id);

    bridge.initialize(&Address::generate(&env), &Address::generate(&env), &50u32);

    let b2_id = env.register(OnboardingBridge, ());
    let b2 = crate::OnboardingBridgeClient::new(&env, &b2_id);
    let target = Address::generate(&env);
    assert_eq!(
        b2.try_fund_c_address(&user, &target, &token_id, &100i128),
        Err(Ok(BridgeError::NotInitialized))
    );
}

#[test]
fn test_batch_fund_c_addresses() {
    let env = Env::default();
    let (admin, user, fee_collector) = create_test_users(&env);
    let (bridge_id, token_id) = register_all_contracts(&env);
    let bridge = create_bridge_client(&env, &bridge_id);
    init_token(&env, &token_id, &admin);

    bridge.initialize(&admin, &fee_collector, &100u32);
    bridge.add_asset(&token_id);
    mint_tokens(&env, &token_id, &user, 3000i128);

    let target1 = Address::generate(&env);
    let target2 = Address::generate(&env);
    let targets = Vec::from_array(&env, [target1.clone(), target2.clone()]);
    let amounts = Vec::from_array(&env, [1000i128, 500i128]);

    bridge.batch_fund_c_address(&user, &targets, &amounts, &token_id);

    assert_eq!(check_balance(&env, &token_id, &user), 1500i128);
    assert_eq!(check_balance(&env, &token_id, &target1), 990i128);
    assert_eq!(check_balance(&env, &token_id, &target2), 495i128);
    assert_eq!(check_balance(&env, &token_id, &fee_collector), 0i128);
    assert_eq!(check_balance(&env, &token_id, &bridge_id), 15i128);
}

#[test]
fn test_fund_with_zero_fee() {
    let env = Env::default();
    let (admin, user, fee_collector) = create_test_users(&env);
    let (bridge_id, token_id) = register_all_contracts(&env);
    let bridge = create_bridge_client(&env, &bridge_id);
    init_token(&env, &token_id, &admin);

    bridge.initialize(&admin, &fee_collector, &0u32);
    bridge.add_asset(&token_id);
    mint_tokens(&env, &token_id, &user, 1000i128);

    let target = Address::generate(&env);
    bridge.fund_c_address(&user, &target, &token_id, &500i128);

    assert_eq!(check_balance(&env, &token_id, &user), 500i128);
    assert_eq!(check_balance(&env, &token_id, &target), 500i128);
    assert_eq!(check_balance(&env, &token_id, &fee_collector), 0i128);
    assert_eq!(check_balance(&env, &token_id, &bridge_id), 0i128);
}

#[test]
fn test_set_fee_bps() {
    let env = Env::default();
    let (admin, _user, fee_collector) = create_test_users(&env);
    let (bridge_id, _) = register_all_contracts(&env);
    let bridge = create_bridge_client(&env, &bridge_id);

    bridge.initialize(&admin, &fee_collector, &50u32);
    assert_eq!(bridge.query_fee_bps(), 50u32);

    bridge.set_fee_bps(&200u32);
    assert_eq!(bridge.query_fee_bps(), 200u32);
}

#[test]
fn test_set_fee_bps_too_high() {
    let env = Env::default();
    let (admin, _user, fee_collector) = create_test_users(&env);
    let (bridge_id, _) = register_all_contracts(&env);
    let bridge = create_bridge_client(&env, &bridge_id);

    bridge.initialize(&admin, &fee_collector, &50u32);
    assert_eq!(
        bridge.try_set_fee_bps(&2000u32),
        Err(Ok(BridgeError::FeeTooHigh))
    );
}

#[test]
fn test_set_fee_collector() {
    let env = Env::default();
    let (admin, _user, fee_collector) = create_test_users(&env);
    let (bridge_id, _) = register_all_contracts(&env);
    let bridge = create_bridge_client(&env, &bridge_id);

    bridge.initialize(&admin, &fee_collector, &50u32);
    let new_collector = Address::generate(&env);
    bridge.set_fee_collector(&new_collector);
    assert_eq!(bridge.query_fee_collector(), new_collector);
}

#[test]
fn test_set_admin() {
    let env = Env::default();
    let (admin, _user, fee_collector) = create_test_users(&env);
    let (bridge_id, _) = register_all_contracts(&env);
    let bridge = create_bridge_client(&env, &bridge_id);

    bridge.initialize(&admin, &fee_collector, &50u32);
    let new_admin = Address::generate(&env);
    bridge.set_admin(&new_admin);
    assert_eq!(bridge.query_admin(), new_admin);
}

#[test]
fn test_withdraw_fees() {
    let env = Env::default();
    let (admin, user, fee_collector) = create_test_users(&env);
    let (bridge_id, token_id) = register_all_contracts(&env);
    let bridge = create_bridge_client(&env, &bridge_id);
    init_token(&env, &token_id, &admin);

    bridge.initialize(&admin, &fee_collector, &100u32);
    bridge.add_asset(&token_id);
    mint_tokens(&env, &token_id, &user, 1000i128);

    let target = Address::generate(&env);
    bridge.fund_c_address(&user, &target, &token_id, &500i128);

    assert_eq!(check_balance(&env, &token_id, &fee_collector), 0i128);
    assert_eq!(check_balance(&env, &token_id, &bridge_id), 5i128);

    bridge.withdraw_fees(&token_id, &5i128);

    assert_eq!(check_balance(&env, &token_id, &fee_collector), 5i128);
    assert_eq!(check_balance(&env, &token_id, &bridge_id), 0i128);
}

#[test]
fn test_query_balance() {
    let env = Env::default();
    let (admin, user, _fee_collector) = create_test_users(&env);
    let (bridge_id, token_id) = register_all_contracts(&env);
    let bridge = create_bridge_client(&env, &bridge_id);
    init_token(&env, &token_id, &admin);

    bridge.initialize(&admin, &Address::generate(&env), &0u32);
    mint_tokens(&env, &token_id, &user, 1000i128);

    let bal = bridge.query_balance(&user, &token_id);
    assert_eq!(bal, 1000i128);
}

#[test]
fn test_batch_empty() {
    let env = Env::default();
    let (admin, _user, fee_collector) = create_test_users(&env);
    let (bridge_id, _) = register_all_contracts(&env);
    let bridge = create_bridge_client(&env, &bridge_id);

    let token_id = Address::generate(&env);
    bridge.initialize(&admin, &fee_collector, &50u32);

    let targets: Vec<Address> = Vec::new(&env);
    let amounts: Vec<i128> = Vec::new(&env);

    bridge.batch_fund_c_address(&admin, &targets, &amounts, &token_id);
}

#[test]
fn test_fund_events() {
    let env = Env::default();
    let (admin, user, fee_collector) = create_test_users(&env);
    let (bridge_id, token_id) = register_all_contracts(&env);
    let bridge = create_bridge_client(&env, &bridge_id);
    init_token(&env, &token_id, &admin);

    bridge.initialize(&admin, &fee_collector, &100u32);
    bridge.add_asset(&token_id);
    mint_tokens(&env, &token_id, &user, 1000i128);

    let target = Address::generate(&env);
    bridge.fund_c_address(&user, &target, &token_id, &500i128);

    let events = env.events().all();
    assert!(events.len() > 0);

    let (contract_id, _topics, _data) = &events.get(events.len() - 1).unwrap();
    assert_eq!(contract_id, &bridge_id);
}

#[test]
fn test_query_fee_bps_uninitialized() {
    let env = Env::default();
    let (bridge_id, _) = register_all_contracts(&env);
    let bridge = create_bridge_client(&env, &bridge_id);
    assert_eq!(
        bridge.try_query_fee_bps(),
        Err(Ok(BridgeError::NotInitialized))
    );
}

/********** Pause / Upgrade tests **********/

#[test]
fn test_pause_and_unpause() {
    let env = Env::default();
    let (admin, _user, fee_collector) = create_test_users(&env);
    let (bridge_id, _) = register_all_contracts(&env);
    let bridge = create_bridge_client(&env, &bridge_id);

    bridge.initialize(&admin, &fee_collector, &50u32);

    assert!(!bridge.query_is_paused());

    bridge.pause();
    assert!(bridge.query_is_paused());

    bridge.unpause();
    assert!(!bridge.query_is_paused());
}

#[test]
fn test_fund_c_address_paused() {
    let env = Env::default();
    let (admin, user, fee_collector) = create_test_users(&env);
    let (bridge_id, token_id) = register_all_contracts(&env);
    let bridge = create_bridge_client(&env, &bridge_id);
    init_token(&env, &token_id, &admin);

    bridge.initialize(&admin, &fee_collector, &100u32);
    mint_tokens(&env, &token_id, &user, 1000i128);
    bridge.pause();

    let target = Address::generate(&env);
    assert_eq!(
        bridge.try_fund_c_address(&user, &target, &token_id, &500i128),
        Err(Ok(BridgeError::ContractPaused))
    );
}

#[test]
fn test_batch_fund_paused() {
    let env = Env::default();
    let (admin, user, fee_collector) = create_test_users(&env);
    let (bridge_id, token_id) = register_all_contracts(&env);
    let bridge = create_bridge_client(&env, &bridge_id);
    init_token(&env, &token_id, &admin);

    bridge.initialize(&admin, &fee_collector, &100u32);
    mint_tokens(&env, &token_id, &user, 1000i128);
    bridge.pause();

    let target = Address::generate(&env);
    let targets = Vec::from_array(&env, [target.clone()]);
    let amounts = Vec::from_array(&env, [500i128]);
    assert_eq!(
        bridge.try_batch_fund_c_address(&user, &targets, &amounts, &token_id),
        Err(Ok(BridgeError::ContractPaused))
    );
}

#[test]
fn test_withdraw_fees_paused() {
    let env = Env::default();
    let (admin, user, fee_collector) = create_test_users(&env);
    let (bridge_id, token_id) = register_all_contracts(&env);
    let bridge = create_bridge_client(&env, &bridge_id);
    init_token(&env, &token_id, &admin);

    bridge.initialize(&admin, &fee_collector, &100u32);
    bridge.add_asset(&token_id);
    mint_tokens(&env, &token_id, &user, 1000i128);
    let target = Address::generate(&env);
    bridge.fund_c_address(&user, &target, &token_id, &500i128);
    bridge.pause();

    assert_eq!(
        bridge.try_withdraw_fees(&token_id, &5i128),
        Err(Ok(BridgeError::ContractPaused))
    );
}

#[test]
fn test_set_fee_bps_paused() {
    let env = Env::default();
    let (admin, _user, fee_collector) = create_test_users(&env);
    let (bridge_id, _) = register_all_contracts(&env);
    let bridge = create_bridge_client(&env, &bridge_id);

    bridge.initialize(&admin, &fee_collector, &50u32);
    bridge.pause();
    assert_eq!(
        bridge.try_set_fee_bps(&100u32),
        Err(Ok(BridgeError::ContractPaused))
    );
}

#[test]
fn test_set_fee_collector_paused() {
    let env = Env::default();
    let (admin, _user, fee_collector) = create_test_users(&env);
    let (bridge_id, _) = register_all_contracts(&env);
    let bridge = create_bridge_client(&env, &bridge_id);

    bridge.initialize(&admin, &fee_collector, &50u32);
    bridge.pause();
    assert_eq!(
        bridge.try_set_fee_collector(&Address::generate(&env)),
        Err(Ok(BridgeError::ContractPaused))
    );
}

#[test]
fn test_set_admin_paused() {
    let env = Env::default();
    let (admin, _user, fee_collector) = create_test_users(&env);
    let (bridge_id, _) = register_all_contracts(&env);
    let bridge = create_bridge_client(&env, &bridge_id);

    bridge.initialize(&admin, &fee_collector, &50u32);
    bridge.pause();
    assert_eq!(
        bridge.try_set_admin(&Address::generate(&env)),
        Err(Ok(BridgeError::ContractPaused))
    );
}

#[test]
fn test_view_functions_work_when_paused() {
    let env = Env::default();
    let (admin, user, fee_collector) = create_test_users(&env);
    let (bridge_id, token_id) = register_all_contracts(&env);
    let bridge = create_bridge_client(&env, &bridge_id);
    init_token(&env, &token_id, &admin);

    bridge.initialize(&admin, &fee_collector, &50u32);
    mint_tokens(&env, &token_id, &user, 1000i128);
    bridge.pause();

    assert_eq!(bridge.query_fee_bps(), 50u32);
    assert_eq!(bridge.query_fee_collector(), fee_collector);
    assert_eq!(bridge.query_admin(), admin);
    assert!(bridge.query_is_initialized());
    assert!(bridge.query_is_paused());
    assert_eq!(bridge.query_balance(&user, &token_id), 1000i128);
}

#[test]
fn test_pause_emits_event() {
    let env = Env::default();
    let (admin, _user, fee_collector) = create_test_users(&env);
    let (bridge_id, _) = register_all_contracts(&env);
    let bridge = create_bridge_client(&env, &bridge_id);

    bridge.initialize(&admin, &fee_collector, &50u32);
    bridge.pause();

    let events = env.events().all();
    let (contract_id, _topics, _data) = &events.get(events.len() - 1).unwrap();
    assert_eq!(contract_id, &bridge_id);
}

#[test]
fn test_unpause_emits_event() {
    let env = Env::default();
    let (admin, _user, fee_collector) = create_test_users(&env);
    let (bridge_id, _) = register_all_contracts(&env);
    let bridge = create_bridge_client(&env, &bridge_id);

    bridge.initialize(&admin, &fee_collector, &50u32);
    bridge.pause();
    bridge.unpause();

    let events = env.events().all();
    let (contract_id, _topics, _data) = &events.get(events.len() - 1).unwrap();
    assert_eq!(contract_id, &bridge_id);
}

#[test]
fn test_fund_works_after_unpause() {
    let env = Env::default();
    let (admin, user, fee_collector) = create_test_users(&env);
    let (bridge_id, token_id) = register_all_contracts(&env);
    let bridge = create_bridge_client(&env, &bridge_id);
    init_token(&env, &token_id, &admin);

    bridge.initialize(&admin, &fee_collector, &100u32);
    bridge.add_asset(&token_id);
    mint_tokens(&env, &token_id, &user, 1000i128);
    bridge.pause();
    bridge.unpause();

    let target = Address::generate(&env);
    bridge.fund_c_address(&user, &target, &token_id, &500i128);

    assert_eq!(check_balance(&env, &token_id, &target), 495i128);
}

// The soroban-sdk ships a known-good compiled wasm fixture used for doc/unit
// tests. We reuse it here as our "v2" wasm to get a real BytesN<32> hash that
// the host accepts, so we can exercise the full auth → wasm-swap → event path.
const V2_WASM: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../target/wasm32-unknown-unknown/release/onboarding_bridge.wasm"
));

#[test]
fn test_upgrade_admin_only_and_event() {
    let env = Env::default();
    let (admin, _user, fee_collector) = create_test_users(&env);
    let (bridge_id, _) = register_all_contracts(&env);
    let bridge = create_bridge_client(&env, &bridge_id);
    env.mock_all_auths();

    bridge.initialize(&admin, &fee_collector, &50u32);

    let wasm_bytes = Bytes::from_slice(&env, V2_WASM);
    let wasm_hash: BytesN<32> = env.deployer().upload_contract_wasm(wasm_bytes);

    bridge.upgrade(&wasm_hash);

    // Verify the Upgraded event was emitted from the bridge contract.
    let events = env.events().all();
    let (contract_id, _topics, _data) = &events.get(events.len() - 1).unwrap();
    assert_eq!(contract_id, &bridge_id);
}

#[test]
#[should_panic]
fn test_upgrade_non_admin_rejected() {
    let env = Env::default();
    let (admin, _user, fee_collector) = create_test_users(&env);
    let bridge_id = env.register(OnboardingBridge, ());
    env.mock_all_auths();
    let bridge = create_bridge_client(&env, &bridge_id);

    bridge.initialize(&admin, &fee_collector, &50u32);

    let wasm_bytes = Bytes::from_slice(&env, V2_WASM);
    let wasm_hash: BytesN<32> = env.deployer().upload_contract_wasm(wasm_bytes);

    // Clear all mocked auths so upgrade is called without admin authorization.
    use soroban_sdk::xdr::SorobanAuthorizationEntry;
    env.set_auths(&[] as &[SorobanAuthorizationEntry]);
    bridge.upgrade(&wasm_hash);
}

// --------- Blocklist / Allowlist Tests ---------

fn setup_bridge(env: &Env) -> (crate::OnboardingBridgeClient, Address, Address, Address) {
    let (bridge_id, token_id) = register_all_contracts(env);
    let bridge = create_bridge_client(env, &bridge_id);
    let (admin, user, fee_collector) = create_test_users(env);
    init_token(env, &token_id, &admin);
    bridge.initialize(&admin, &fee_collector, &0u32);
    bridge.add_asset(&token_id);
    mint_tokens(env, &token_id, &user, 1000i128);
    (bridge, user, token_id, admin)
}

#[test]
fn test_blocklist_prevents_fund() {
    let env = Env::default();
    let (bridge, user, token_id, _admin) = setup_bridge(&env);
    let target = Address::generate(&env);

    bridge.add_to_blocklist(&target);
    assert!(bridge.query_is_blocked(&target));

    assert_eq!(
        bridge.try_fund_c_address(&user, &target, &token_id, &500i128),
        Err(Ok(crate::BridgeError::AddressBlocked))
    );
}

#[test]
fn test_remove_from_blocklist_allows_fund() {
    let env = Env::default();
    let (bridge, user, token_id, _admin) = setup_bridge(&env);
    let target = Address::generate(&env);

    bridge.add_to_blocklist(&target);
    bridge.remove_from_blocklist(&target);
    assert!(!bridge.query_is_blocked(&target));

    bridge.fund_c_address(&user, &target, &token_id, &500i128);
    assert_eq!(check_balance(&env, &token_id, &target), 500i128);
}

#[test]
fn test_allowlist_mode_blocks_non_allowlisted() {
    let env = Env::default();
    let (bridge, user, token_id, _admin) = setup_bridge(&env);
    let target = Address::generate(&env);

    bridge.set_allowlist_mode(&true);
    assert!(bridge.query_allowlist_mode());

    assert_eq!(
        bridge.try_fund_c_address(&user, &target, &token_id, &500i128),
        Err(Ok(crate::BridgeError::AddressNotAllowlisted))
    );
}

#[test]
fn test_allowlist_mode_allows_allowlisted() {
    let env = Env::default();
    let (bridge, user, token_id, _admin) = setup_bridge(&env);
    let target = Address::generate(&env);

    bridge.set_allowlist_mode(&true);
    bridge.add_to_allowlist(&target);
    assert!(bridge.query_is_allowlisted(&target));

    bridge.fund_c_address(&user, &target, &token_id, &500i128);
    assert_eq!(check_balance(&env, &token_id, &target), 500i128);
}

#[test]
fn test_remove_from_allowlist_blocks_in_allowlist_mode() {
    let env = Env::default();
    let (bridge, user, token_id, _admin) = setup_bridge(&env);
    let target = Address::generate(&env);

    bridge.set_allowlist_mode(&true);
    bridge.add_to_allowlist(&target);
    bridge.remove_from_allowlist(&target);
    assert!(!bridge.query_is_allowlisted(&target));

    assert_eq!(
        bridge.try_fund_c_address(&user, &target, &token_id, &500i128),
        Err(Ok(crate::BridgeError::AddressNotAllowlisted))
    );
}

#[test]
fn test_blocklist_overrides_allowlist() {
    let env = Env::default();
    let (bridge, user, token_id, _admin) = setup_bridge(&env);
    let target = Address::generate(&env);

    bridge.set_allowlist_mode(&true);
    bridge.add_to_allowlist(&target);
    bridge.add_to_blocklist(&target);

    assert_eq!(
        bridge.try_fund_c_address(&user, &target, &token_id, &500i128),
        Err(Ok(crate::BridgeError::AddressBlocked))
    );
}

#[test]
fn test_batch_fund_blocked_address_fails() {
    let env = Env::default();
    let (bridge, user, token_id, _admin) = setup_bridge(&env);
    let t1 = Address::generate(&env);
    let t2 = Address::generate(&env);

    bridge.add_to_blocklist(&t2);

    let targets = Vec::from_array(&env, [t1, t2]);
    let amounts = Vec::from_array(&env, [200i128, 300i128]);

    assert_eq!(
        bridge.try_batch_fund_c_address(&user, &targets, &amounts, &token_id),
        Err(Ok(crate::BridgeError::AddressBlocked))
    );
}

#[test]
fn test_allowlist_mode_off_allows_all() {
    let env = Env::default();
    let (bridge, user, token_id, _admin) = setup_bridge(&env);
    let target = Address::generate(&env);

    // allowlist mode off by default
    assert!(!bridge.query_allowlist_mode());
    bridge.fund_c_address(&user, &target, &token_id, &500i128);
    assert_eq!(check_balance(&env, &token_id, &target), 500i128);
}

// --------- reclaim_tokens Tests ---------

#[test]
fn test_reclaim_accidentally_sent_tokens() {
    let env = Env::default();
    let (bridge, _user, token_id, admin) = setup_bridge(&env);

    // Directly mint tokens to bridge (simulating accidental transfer, no fees accrued)
    mint_tokens(&env, &token_id, &bridge.address, 500i128);

    let destination = Address::generate(&env);
    bridge.reclaim_tokens(&token_id, &500i128, &destination);

    assert_eq!(check_balance(&env, &token_id, &destination), 500i128);
    let _ = admin; // suppress unused warning
}

#[test]
fn test_reclaim_cannot_take_accrued_fees() {
    let env = Env::default();
    let (bridge, user, token_id, _admin) = setup_bridge(&env);

    // Fund so fees (10%) accrue in contract
    bridge.set_fee_bps(&1000u32); // 10%
    let target = Address::generate(&env);
    bridge.fund_c_address(&user, &target, &token_id, &1000i128);
    // contract now holds 100 in accrued fees, 0 reclaimable

    let destination = Address::generate(&env);
    assert_eq!(
        bridge.try_reclaim_tokens(&token_id, &1i128, &destination),
        Err(Ok(crate::BridgeError::InsufficientReclaimable))
    );
}

#[test]
fn test_reclaim_only_excess_over_fees() {
    let env = Env::default();
    let (bridge, user, token_id, _admin) = setup_bridge(&env);

    bridge.set_fee_bps(&1000u32); // 10%
    let target = Address::generate(&env);
    bridge.fund_c_address(&user, &target, &token_id, &1000i128);
    // 100 accrued fees in contract; mint 200 more directly
    mint_tokens(&env, &token_id, &bridge.address, 200i128);

    let destination = Address::generate(&env);
    // Can reclaim exactly 200 (the excess)
    bridge.reclaim_tokens(&token_id, &200i128, &destination);
    assert_eq!(check_balance(&env, &token_id, &destination), 200i128);

    // Cannot reclaim 1 more
    let dest2 = Address::generate(&env);
    assert_eq!(
        bridge.try_reclaim_tokens(&token_id, &1i128, &dest2),
        Err(Ok(crate::BridgeError::InsufficientReclaimable))
    );
}

#[test]
fn test_reclaim_emits_event() {
    let env = Env::default();
    let (bridge, _user, token_id, _admin) = setup_bridge(&env);

    mint_tokens(&env, &token_id, &bridge.address, 300i128);
    let destination = Address::generate(&env);
    bridge.reclaim_tokens(&token_id, &300i128, &destination);

    let events = env.events().all();
    let (contract_id, _topics, _data) = &events.get(events.len() - 1).unwrap();
    assert_eq!(contract_id, &bridge.address);
}

/********** Asset whitelist tests **********/

#[test]
fn test_add_asset_whitelists_it() {
    let env = Env::default();
    let (admin, _user, fee_collector) = create_test_users(&env);
    let (bridge_id, token_id) = register_all_contracts(&env);
    let bridge = create_bridge_client(&env, &bridge_id);

    bridge.initialize(&admin, &fee_collector, &50u32);
    assert_eq!(bridge.query_is_asset_whitelisted(&token_id), false);

    bridge.add_asset(&token_id);
    assert_eq!(bridge.query_is_asset_whitelisted(&token_id), true);
}

#[test]
fn test_remove_asset() {
    let env = Env::default();
    let (admin, _user, fee_collector) = create_test_users(&env);
    let (bridge_id, token_id) = register_all_contracts(&env);
    let bridge = create_bridge_client(&env, &bridge_id);

    bridge.initialize(&admin, &fee_collector, &50u32);
    bridge.add_asset(&token_id);
    assert_eq!(bridge.query_is_asset_whitelisted(&token_id), true);

    bridge.remove_asset(&token_id);
    assert_eq!(bridge.query_is_asset_whitelisted(&token_id), false);
}

#[test]
fn test_query_whitelisted_assets() {
    let env = Env::default();
    let (admin, _user, fee_collector) = create_test_users(&env);
    let (bridge_id, _) = register_all_contracts(&env);
    let bridge = create_bridge_client(&env, &bridge_id);

    bridge.initialize(&admin, &fee_collector, &50u32);

    let asset1 = Address::generate(&env);
    let asset2 = Address::generate(&env);
    bridge.add_asset(&asset1);
    bridge.add_asset(&asset2);

    let assets = bridge.query_whitelisted_assets();
    assert_eq!(assets.len(), 2);

    let mut found1 = false;
    let mut found2 = false;
    for a in assets.iter() {
        if a == asset1 {
            found1 = true;
        }
        if a == asset2 {
            found2 = true;
        }
    }
    assert!(found1 && found2);
}

#[test]
fn test_add_asset_is_idempotent() {
    let env = Env::default();
    let (admin, _user, fee_collector) = create_test_users(&env);
    let (bridge_id, token_id) = register_all_contracts(&env);
    let bridge = create_bridge_client(&env, &bridge_id);

    bridge.initialize(&admin, &fee_collector, &50u32);
    bridge.add_asset(&token_id);
    bridge.add_asset(&token_id);

    assert_eq!(bridge.query_whitelisted_assets().len(), 1);
}

#[test]
#[should_panic]
fn test_add_asset_non_admin_rejected() {
    let env = Env::default();
    let (admin, _user, fee_collector) = create_test_users(&env);
    let (bridge_id, token_id) = register_all_contracts(&env);
    let bridge = create_bridge_client(&env, &bridge_id);

    bridge.initialize(&admin, &fee_collector, &50u32);

    env.set_auths(&[]);
    bridge.add_asset(&token_id);
}

#[test]
#[should_panic]
fn test_remove_asset_non_admin_rejected() {
    let env = Env::default();
    let (admin, _user, fee_collector) = create_test_users(&env);
    let (bridge_id, token_id) = register_all_contracts(&env);
    let bridge = create_bridge_client(&env, &bridge_id);

    bridge.initialize(&admin, &fee_collector, &50u32);
    bridge.add_asset(&token_id);

    env.set_auths(&[]);
    bridge.remove_asset(&token_id);
}

#[test]
fn test_whitelist_query_uninitialized() {
    let env = Env::default();
    let (bridge_id, token_id) = register_all_contracts(&env);
    let bridge = create_bridge_client(&env, &bridge_id);
    assert_eq!(
        bridge.try_query_is_asset_whitelisted(&token_id),
        Err(Ok(BridgeError::NotInitialized))
    );
}

#[test]
fn test_fund_rejects_non_whitelisted_asset() {
    let env = Env::default();
    let (admin, user, fee_collector) = create_test_users(&env);
    let (bridge_id, token_id) = register_all_contracts(&env);
    let bridge = create_bridge_client(&env, &bridge_id);
    init_token(&env, &token_id, &admin);

    bridge.initialize(&admin, &fee_collector, &100u32);
    mint_tokens(&env, &token_id, &user, 1000i128);

    let target = Address::generate(&env);
    assert_eq!(
        bridge.try_fund_c_address(&user, &target, &token_id, &500i128),
        Err(Ok(BridgeError::AssetNotWhitelisted))
    );
}

#[test]
fn test_batch_fund_rejects_non_whitelisted_asset() {
    let env = Env::default();
    let (admin, user, fee_collector) = create_test_users(&env);
    let (bridge_id, token_id) = register_all_contracts(&env);
    let bridge = create_bridge_client(&env, &bridge_id);
    init_token(&env, &token_id, &admin);

    bridge.initialize(&admin, &fee_collector, &100u32);
    mint_tokens(&env, &token_id, &user, 3000i128);

    let target1 = Address::generate(&env);
    let targets = Vec::from_array(&env, [target1]);
    let amounts = Vec::from_array(&env, [1000i128]);

    assert_eq!(
        bridge.try_batch_fund_c_address(&user, &targets, &amounts, &token_id),
        Err(Ok(BridgeError::AssetNotWhitelisted))
    );
}

/********** query_all_balances Tests **********/

#[test]
fn test_query_all_balances_returns_contract_balances() {
    let env = Env::default();
    let (admin, _user, fee_collector) = create_test_users(&env);
    let (bridge_id, token_id) = register_all_contracts(&env);
    let bridge = create_bridge_client(&env, &bridge_id);
    init_token(&env, &token_id, &admin);
    bridge.initialize(&admin, &fee_collector, &0u32);

    // Mint directly to the bridge contract
    mint_tokens(&env, &token_id, &bridge_id, 750i128);

    let assets = Vec::from_array(&env, [token_id.clone()]);
    let balances = bridge.query_all_balances(&assets);

    assert_eq!(balances.get(token_id).unwrap(), 750i128);
}

#[test]
fn test_query_all_balances_empty_input() {
    let env = Env::default();
    let (admin, _user, fee_collector) = create_test_users(&env);
    let (bridge_id, _token_id) = register_all_contracts(&env);
    let bridge = create_bridge_client(&env, &bridge_id);
    bridge.initialize(&admin, &fee_collector, &0u32);

    let assets: Vec<Address> = Vec::new(&env);
    let balances = bridge.query_all_balances(&assets);
    assert_eq!(balances.len(), 0);
}

/********** Minimal Test Token **********/

#[contracttype]
pub enum TDataKey {
    Admin,
    Decimal,
    Name,
    Symbol,
    Initialized,
    Balance,
}

#[contract]
pub struct TestToken;

#[contractimpl]
impl TestToken {
    pub fn initialize(
        e: Env,
        admin: Address,
        decimal: u32,
        name: soroban_sdk::String,
        symbol: soroban_sdk::String,
    ) {
        e.storage().instance().set(&TDataKey::Admin, &admin);
        e.storage().instance().set(&TDataKey::Decimal, &decimal);
        e.storage().instance().set(&TDataKey::Name, &name);
        e.storage().instance().set(&TDataKey::Symbol, &symbol);
        e.storage().instance().set(&TDataKey::Initialized, &true);
    }

    pub fn mint(e: Env, to: Address, amount: i128) {
        let admin: Address = e.storage().instance().get(&TDataKey::Admin).unwrap();
        admin.require_auth();
        let bal = Self::balance(e.clone(), to.clone());
        e.storage()
            .persistent()
            .set(&(TDataKey::Balance, to), &(bal + amount));
    }

    pub fn balance(e: Env, id: Address) -> i128 {
        e.storage()
            .persistent()
            .get(&(TDataKey::Balance, id))
            .unwrap_or(0)
    }

    pub fn transfer(e: Env, from: Address, to: Address, amount: i128) {
        from.require_auth();
        let from_bal = Self::balance(e.clone(), from.clone());
        if from_bal < amount {
            panic!("insufficient balance");
        }
        let to_bal = Self::balance(e.clone(), to.clone());
        e.storage()
            .persistent()
            .set(&(TDataKey::Balance, from), &(from_bal - amount));
        e.storage()
            .persistent()
            .set(&(TDataKey::Balance, to), &(to_bal + amount));
    }
}

/********** query_calculate_fee tests **********/

#[test]
fn test_query_calculate_fee() {
    let env = Env::default();
    let (admin, _user, fee_collector) = create_test_users(&env);
    let (bridge_id, _) = register_all_contracts(&env);
    let bridge = create_bridge_client(&env, &bridge_id);

    bridge.initialize(&admin, &fee_collector, &100u32);

    let (fee, net) = bridge.query_calculate_fee(&1000i128);
    assert_eq!(fee, 10i128);
    assert_eq!(net, 990i128);
}

#[test]
fn test_query_calculate_fee_zero_fee() {
    let env = Env::default();
    let (admin, _user, fee_collector) = create_test_users(&env);
    let (bridge_id, _) = register_all_contracts(&env);
    let bridge = create_bridge_client(&env, &bridge_id);

    bridge.initialize(&admin, &fee_collector, &0u32);

    let (fee, net) = bridge.query_calculate_fee(&1000i128);
    assert_eq!(fee, 0i128);
    assert_eq!(net, 1000i128);
}

#[test]
fn test_query_calculate_fee_max_fee() {
    let env = Env::default();
    let (admin, _user, fee_collector) = create_test_users(&env);
    let (bridge_id, _) = register_all_contracts(&env);
    let bridge = create_bridge_client(&env, &bridge_id);

    bridge.initialize(&admin, &fee_collector, &1000u32);

    let (fee, net) = bridge.query_calculate_fee(&1000i128);
    assert_eq!(fee, 100i128);
    assert_eq!(net, 900i128);
}

/********** cumulative counters tests **********/

#[test]
fn test_query_total_bridged_and_fees_collected() {
    let env = Env::default();
    let (admin, user, fee_collector) = create_test_users(&env);
    let (bridge_id, token_id) = register_all_contracts(&env);
    let bridge = create_bridge_client(&env, &bridge_id);
    init_token(&env, &token_id, &admin);

    bridge.initialize(&admin, &fee_collector, &100u32);
    bridge.add_asset(&token_id);
    mint_tokens(&env, &token_id, &user, 1000i128);

    let target = Address::generate(&env);
    bridge.fund_c_address(&user, &target, &token_id, &500i128);

    let total_bridged = bridge.query_total_bridged(&token_id);
    let total_fees = bridge.query_total_fees_collected(&token_id);

    assert_eq!(total_bridged, 495i128);
    assert_eq!(total_fees, 5i128);
}

#[test]
fn test_query_total_bridged_accumulates() {
    let env = Env::default();
    let (admin, user, fee_collector) = create_test_users(&env);
    let (bridge_id, token_id) = register_all_contracts(&env);
    let bridge = create_bridge_client(&env, &bridge_id);
    init_token(&env, &token_id, &admin);

    bridge.initialize(&admin, &fee_collector, &50u32);
    bridge.add_asset(&token_id);
    mint_tokens(&env, &token_id, &user, 5000i128);

    let target1 = Address::generate(&env);
    let target2 = Address::generate(&env);

    bridge.fund_c_address(&user, &target1, &token_id, &1000i128);
    bridge.fund_c_address(&user, &target2, &token_id, &1000i128);

    let total_bridged = bridge.query_total_bridged(&token_id);
    let total_fees = bridge.query_total_fees_collected(&token_id);

    assert_eq!(total_bridged, 1990i128);
    assert_eq!(total_fees, 10i128);
}

#[test]
fn test_query_total_bridged_batch() {
    let env = Env::default();
    let (admin, user, fee_collector) = create_test_users(&env);
    let (bridge_id, token_id) = register_all_contracts(&env);
    let bridge = create_bridge_client(&env, &bridge_id);
    init_token(&env, &token_id, &admin);

    bridge.initialize(&admin, &fee_collector, &100u32);
    bridge.add_asset(&token_id);
    mint_tokens(&env, &token_id, &user, 3000i128);

    let target1 = Address::generate(&env);
    let target2 = Address::generate(&env);
    let targets = Vec::from_array(&env, [target1, target2]);
    let amounts = Vec::from_array(&env, [1000i128, 500i128]);

    bridge.batch_fund_c_address(&user, &targets, &amounts, &token_id);

    let total_bridged = bridge.query_total_bridged(&token_id);
    let total_fees = bridge.query_total_fees_collected(&token_id);

    assert_eq!(total_bridged, 1485i128);
    assert_eq!(total_fees, 15i128);
}

#[test]
fn test_query_total_bridged_zero() {
    let env = Env::default();
    let (admin, _user, fee_collector) = create_test_users(&env);
    let (bridge_id, token_id) = register_all_contracts(&env);
    let bridge = create_bridge_client(&env, &bridge_id);

    bridge.initialize(&admin, &fee_collector, &50u32);

    let total_bridged = bridge.query_total_bridged(&token_id);
    let total_fees = bridge.query_total_fees_collected(&token_id);

    assert_eq!(total_bridged, 0i128);
    assert_eq!(total_fees, 0i128);
}

/********** admin state change events tests **********/

#[test]
fn test_initialize_emits_event() {
    let env = Env::default();
    let (admin, _user, fee_collector) = create_test_users(&env);
    let (bridge_id, _) = register_all_contracts(&env);
    let bridge = create_bridge_client(&env, &bridge_id);

    bridge.initialize(&admin, &fee_collector, &50u32);

    let events = env.events().all();
    let (contract_id, _topics, _data) = &events.get(events.len() - 1).unwrap();
    assert_eq!(contract_id, &bridge_id);
}

#[test]
fn test_fee_bps_changed_emits_event() {
    let env = Env::default();
    let (admin, _user, fee_collector) = create_test_users(&env);
    let (bridge_id, _) = register_all_contracts(&env);
    let bridge = create_bridge_client(&env, &bridge_id);

    bridge.initialize(&admin, &fee_collector, &50u32);
    bridge.set_fee_bps(&100u32);

    let events = env.events().all();
    let (contract_id, _topics, _data) = &events.get(events.len() - 1).unwrap();
    assert_eq!(contract_id, &bridge_id);
}

#[test]
fn test_fee_collector_changed_emits_event() {
    let env = Env::default();
    let (admin, _user, fee_collector) = create_test_users(&env);
    let (bridge_id, _) = register_all_contracts(&env);
    let bridge = create_bridge_client(&env, &bridge_id);

    bridge.initialize(&admin, &fee_collector, &50u32);
    let new_collector = Address::generate(&env);
    bridge.set_fee_collector(&new_collector);

    let events = env.events().all();
    let (contract_id, _topics, _data) = &events.get(events.len() - 1).unwrap();
    assert_eq!(contract_id, &bridge_id);
}

#[test]
fn test_admin_changed_emits_event() {
    let env = Env::default();
    let (admin, _user, fee_collector) = create_test_users(&env);
    let (bridge_id, _) = register_all_contracts(&env);
    let bridge = create_bridge_client(&env, &bridge_id);

    bridge.initialize(&admin, &fee_collector, &50u32);
    let new_admin = Address::generate(&env);
    bridge.set_admin(&new_admin);

    let events = env.events().all();
    let (contract_id, _topics, _data) = &events.get(events.len() - 1).unwrap();
    assert_eq!(contract_id, &bridge_id);
}

/********** Timelocked Funding Tests **********/

fn setup_timelocked(env: &Env) -> (Address, Address, Address, Address, crate::OnboardingBridgeClient) {
    let (admin, user, fee_collector) = create_test_users(env);
    let (bridge_id, token_id) = register_all_contracts(env);
    let bridge = create_bridge_client(env, &bridge_id);
    init_token(env, &token_id, &admin);
    bridge.initialize(&admin, &fee_collector, &100u32); // 1% fee
    bridge.add_asset(&token_id);
    mint_tokens(env, &token_id, &user, 1000i128);
    (bridge_id, token_id, user, fee_collector, bridge)
}

#[test]
fn test_fund_timelocked_creates_entry() {
    let env = Env::default();
    env.ledger().set_timestamp(1000);
    let (_bridge_id, token_id, user, _fee_collector, bridge) = setup_timelocked(&env);
    let target = Address::generate(&env);

    let id = bridge.fund_c_address_timelocked(&user, &target, &token_id, &500i128, &2000u64, &0u64);

    assert_eq!(id, 0u64);
    let entry = bridge.query_timelocked(&id);
    assert_eq!(entry.amount, 500i128);
    assert_eq!(entry.release_time, 2000u64);
    assert_eq!(entry.cliff_time, 0u64);
    assert!(!entry.claimed);
}

#[test]
fn test_fund_timelocked_holds_tokens_in_contract() {
    let env = Env::default();
    env.ledger().set_timestamp(1000);
    let (bridge_id, token_id, user, _fee_collector, bridge) = setup_timelocked(&env);
    let target = Address::generate(&env);

    bridge.fund_c_address_timelocked(&user, &target, &token_id, &500i128, &2000u64, &0u64);

    // Tokens leave source, sit in bridge (not yet transferred to target)
    assert_eq!(check_balance(&env, &token_id, &user), 500i128);
    assert_eq!(check_balance(&env, &token_id, &target), 0i128);
    assert_eq!(check_balance(&env, &token_id, &bridge_id), 500i128);
}

#[test]
fn test_claim_timelocked_after_release() {
    let env = Env::default();
    env.ledger().set_timestamp(1000);
    let (bridge_id, token_id, user, _fee_collector, bridge) = setup_timelocked(&env);
    let target = Address::generate(&env);

    let id = bridge.fund_c_address_timelocked(&user, &target, &token_id, &500i128, &2000u64, &0u64);

    env.ledger().set_timestamp(2000);
    bridge.claim_timelocked(&id);

    // 1% fee on 500 = 5; net = 495
    assert_eq!(check_balance(&env, &token_id, &target), 495i128);
    assert_eq!(check_balance(&env, &token_id, &bridge_id), 5i128);
}

#[test]
fn test_claim_timelocked_before_release_fails() {
    let env = Env::default();
    env.ledger().set_timestamp(1000);
    let (_bridge_id, token_id, user, _fee_collector, bridge) = setup_timelocked(&env);
    let target = Address::generate(&env);

    let id = bridge.fund_c_address_timelocked(&user, &target, &token_id, &500i128, &2000u64, &0u64);

    env.ledger().set_timestamp(1999);
    assert_eq!(
        bridge.try_claim_timelocked(&id),
        Err(Ok(BridgeError::TimelockNotMatured))
    );
}

#[test]
fn test_claim_timelocked_twice_fails() {
    let env = Env::default();
    env.ledger().set_timestamp(1000);
    let (_bridge_id, token_id, user, _fee_collector, bridge) = setup_timelocked(&env);
    let target = Address::generate(&env);

    let id = bridge.fund_c_address_timelocked(&user, &target, &token_id, &500i128, &2000u64, &0u64);

    env.ledger().set_timestamp(2000);
    bridge.claim_timelocked(&id);

    assert_eq!(
        bridge.try_claim_timelocked(&id),
        Err(Ok(BridgeError::Unauthorized))
    );
}

#[test]
fn test_query_timelocked_not_found() {
    let env = Env::default();
    let (_bridge_id, _token_id, _user, _fee_collector, bridge) = setup_timelocked(&env);

    assert_eq!(
        bridge.try_query_timelocked(&99u64),
        Err(Ok(BridgeError::TimelockNotFound))
    );
}

#[test]
fn test_fund_timelocked_invalid_release_time() {
    let env = Env::default();
    env.ledger().set_timestamp(1000);
    let (_bridge_id, token_id, user, _fee_collector, bridge) = setup_timelocked(&env);
    let target = Address::generate(&env);

    // release_time in the past
    assert_eq!(
        bridge.try_fund_c_address_timelocked(&user, &target, &token_id, &500i128, &999u64, &0u64),
        Err(Ok(BridgeError::InvalidReleaseTime))
    );
}

#[test]
fn test_fund_timelocked_cliff_after_release_fails() {
    let env = Env::default();
    env.ledger().set_timestamp(1000);
    let (_bridge_id, token_id, user, _fee_collector, bridge) = setup_timelocked(&env);
    let target = Address::generate(&env);

    // cliff_time > release_time
    assert_eq!(
        bridge.try_fund_c_address_timelocked(&user, &target, &token_id, &500i128, &2000u64, &3000u64),
        Err(Ok(BridgeError::InvalidReleaseTime))
    );
}

#[test]
fn test_fund_timelocked_with_cliff_stored_correctly() {
    let env = Env::default();
    env.ledger().set_timestamp(1000);
    let (_bridge_id, token_id, user, _fee_collector, bridge) = setup_timelocked(&env);
    let target = Address::generate(&env);

    let id = bridge.fund_c_address_timelocked(&user, &target, &token_id, &500i128, &2000u64, &1500u64);
    let entry = bridge.query_timelocked(&id);
    assert_eq!(entry.cliff_time, 1500u64);
    assert_eq!(entry.release_time, 2000u64);
}

#[test]
fn test_fund_timelocked_increments_id() {
    let env = Env::default();
    env.ledger().set_timestamp(1000);
    let (_bridge_id, token_id, user, _fee_collector, bridge) = setup_timelocked(&env);
    let target = Address::generate(&env);

    let id0 = bridge.fund_c_address_timelocked(&user, &target, &token_id, &100i128, &2000u64, &0u64);
    let id1 = bridge.fund_c_address_timelocked(&user, &target, &token_id, &100i128, &2000u64, &0u64);
    let id2 = bridge.fund_c_address_timelocked(&user, &target, &token_id, &100i128, &2000u64, &0u64);

    assert_eq!(id0, 0u64);
    assert_eq!(id1, 1u64);
    assert_eq!(id2, 2u64);
}

#[test]
fn test_claim_timelocked_updates_counters() {
    let env = Env::default();
    env.ledger().set_timestamp(1000);
    let (_bridge_id, token_id, user, _fee_collector, bridge) = setup_timelocked(&env);
    let target = Address::generate(&env);

    let id = bridge.fund_c_address_timelocked(&user, &target, &token_id, &1000i128, &2000u64, &0u64);
    env.ledger().set_timestamp(2000);
    bridge.claim_timelocked(&id);

    // 1% fee on 1000 = 10; net = 990
    assert_eq!(bridge.query_total_bridged(&token_id), 990i128);
    assert_eq!(bridge.query_total_fees_collected(&token_id), 10i128);
}

/********** Cross-chain Onboarding Tests **********/

#[cfg(test)]
mod crosschain_tests {
    use super::*;
    use crate::{BridgeError, OnboardingBridge, RelayerSig};
    use ed25519_dalek::{Signer, SigningKey};
    use soroban_sdk::{Bytes, BytesN, Env, Vec};

    fn make_signing_key(seed: [u8; 32]) -> SigningKey {
        SigningKey::from_bytes(&seed)
    }

    /// Replicates the contract's payload hash for a given set of call arguments.
    fn build_payload_hash(
        env: &Env,
        chain_id: u32,
        tx_hash: &BytesN<32>,
        target: &soroban_sdk::Address,
        asset: &soroban_sdk::Address,
        amount: i128,
    ) -> BytesN<32> {
        let tx_hash_bytes: Bytes = tx_hash.clone().into();

        // nonce = sha256(chain_id_be4 || tx_hash)
        let mut nonce_pre = Bytes::new(env);
        nonce_pre.extend_from_array(&chain_id.to_be_bytes());
        nonce_pre.append(&tx_hash_bytes);
        let nonce: BytesN<32> = env.crypto().sha256(&nonce_pre).into();

        let target_bytes = target.clone().to_xdr(env);
        let asset_bytes = asset.clone().to_xdr(env);
        let nonce_bytes: Bytes = nonce.into();

        let mut payload = Bytes::new(env);
        payload.extend_from_array(&chain_id.to_be_bytes());
        payload.append(&tx_hash_bytes);
        payload.append(&target_bytes);
        payload.append(&asset_bytes);
        payload.extend_from_array(&amount.to_be_bytes());
        payload.append(&nonce_bytes);

        env.crypto().sha256(&payload).into()
    }

    fn make_relayer_sig(
        env: &Env,
        signing_key: &SigningKey,
        payload_hash: &BytesN<32>,
    ) -> RelayerSig {
        let hash_bytes: Bytes = payload_hash.clone().into();
        let mut hash_arr = [0u8; 32];
        for i in 0..32 {
            hash_arr[i] = hash_bytes.get(i as u32).unwrap();
        }
        let sig = signing_key.sign(&hash_arr);
        RelayerSig {
            pubkey: BytesN::from_array(env, signing_key.verifying_key().as_bytes()),
            signature: BytesN::from_array(env, &sig.to_bytes()),
        }
    }

    fn setup(env: &Env) -> (
        soroban_sdk::Address,
        soroban_sdk::Address,
        soroban_sdk::Address,
        crate::OnboardingBridgeClient,
    ) {
        let bridge_id = env.register(OnboardingBridge, ());
        let token_id = env.register(TestToken, ());
        env.mock_all_auths();

        let admin = soroban_sdk::Address::generate(env);
        let fee_collector = soroban_sdk::Address::generate(env);

        let bridge = crate::OnboardingBridgeClient::new(env, &bridge_id);
        TestTokenClient::new(env, &token_id).initialize(
            &admin,
            &7u32,
            &"Test".into_val(env),
            &"TST".into_val(env),
        );
        bridge.initialize(&admin, &fee_collector, &100u32); // 1% fee
        bridge.add_asset(&token_id);

        // Fund the bridge contract so it can pay out cross-chain claims
        TestTokenClient::new(env, &token_id).mint(&bridge_id, &10_000i128);

        (bridge_id, token_id, admin, bridge)
    }

    #[test]
    fn test_crosschain_happy_path_single_relayer() {
        let env = Env::default();
        let (bridge_id, token_id, _admin, bridge) = setup(&env);

        let sk = make_signing_key([1u8; 32]);
        let pubkey = BytesN::from_array(&env, sk.verifying_key().as_bytes());

        bridge.add_relayer(&pubkey);
        bridge.set_relayer_threshold(&1u32);

        let target = soroban_sdk::Address::generate(&env);
        let tx_hash = BytesN::from_array(&env, &[0xab; 32]);
        let chain_id: u32 = 1;
        let amount: i128 = 1000;

        let payload_hash = build_payload_hash(&env, chain_id, &tx_hash, &target, &token_id, amount);
        let sig = make_relayer_sig(&env, &sk, &payload_hash);
        let sigs = Vec::from_array(&env, [sig]);

        bridge.fund_c_address_crosschain(&chain_id, &tx_hash, &target, &token_id, &amount, &sigs);

        // 1% fee on 1000 = 10; net = 990
        assert_eq!(TestTokenClient::new(&env, &token_id).balance(&target), 990i128);
        assert_eq!(TestTokenClient::new(&env, &token_id).balance(&bridge_id), 10_000 - 990);
    }

    #[test]
    fn test_crosschain_happy_path_2_of_3() {
        let env = Env::default();
        let (_bridge_id, token_id, _admin, bridge) = setup(&env);

        let sk1 = make_signing_key([1u8; 32]);
        let sk2 = make_signing_key([2u8; 32]);
        let sk3 = make_signing_key([3u8; 32]);

        bridge.add_relayer(&BytesN::from_array(&env, sk1.verifying_key().as_bytes()));
        bridge.add_relayer(&BytesN::from_array(&env, sk2.verifying_key().as_bytes()));
        bridge.add_relayer(&BytesN::from_array(&env, sk3.verifying_key().as_bytes()));
        bridge.set_relayer_threshold(&2u32);

        let target = soroban_sdk::Address::generate(&env);
        let tx_hash = BytesN::from_array(&env, &[0xcd; 32]);
        let chain_id: u32 = 101;
        let amount: i128 = 500;

        let payload_hash = build_payload_hash(&env, chain_id, &tx_hash, &target, &token_id, amount);
        let sigs = Vec::from_array(&env, [
            make_relayer_sig(&env, &sk1, &payload_hash),
            make_relayer_sig(&env, &sk2, &payload_hash),
        ]);

        bridge.fund_c_address_crosschain(&chain_id, &tx_hash, &target, &token_id, &amount, &sigs);
        assert_eq!(TestTokenClient::new(&env, &token_id).balance(&target), 495i128);
    }

    #[test]
    fn test_crosschain_replay_rejected() {
        let env = Env::default();
        let (_bridge_id, token_id, _admin, bridge) = setup(&env);

        let sk = make_signing_key([1u8; 32]);
        bridge.add_relayer(&BytesN::from_array(&env, sk.verifying_key().as_bytes()));
        bridge.set_relayer_threshold(&1u32);

        let target = soroban_sdk::Address::generate(&env);
        let tx_hash = BytesN::from_array(&env, &[0xef; 32]);

        let payload_hash = build_payload_hash(&env, 1, &tx_hash, &target, &token_id, 100);
        let sigs = Vec::from_array(&env, [make_relayer_sig(&env, &sk, &payload_hash)]);

        bridge.fund_c_address_crosschain(&1u32, &tx_hash, &target, &token_id, &100i128, &sigs);

        // Second call with same tx_hash must fail
        assert_eq!(
            bridge.try_fund_c_address_crosschain(&1u32, &tx_hash, &target, &token_id, &100i128, &sigs),
            Err(Ok(BridgeError::ReplayedNonce))
        );
    }

    #[test]
    fn test_crosschain_below_threshold_rejected() {
        let env = Env::default();
        let (_bridge_id, token_id, _admin, bridge) = setup(&env);

        let sk1 = make_signing_key([1u8; 32]);
        let sk2 = make_signing_key([2u8; 32]);

        bridge.add_relayer(&BytesN::from_array(&env, sk1.verifying_key().as_bytes()));
        bridge.add_relayer(&BytesN::from_array(&env, sk2.verifying_key().as_bytes()));
        bridge.set_relayer_threshold(&2u32);

        let target = soroban_sdk::Address::generate(&env);
        let tx_hash = BytesN::from_array(&env, &[0x11; 32]);

        let payload_hash = build_payload_hash(&env, 1, &tx_hash, &target, &token_id, 100);
        // Only 1 sig when threshold is 2
        let sigs = Vec::from_array(&env, [make_relayer_sig(&env, &sk1, &payload_hash)]);

        assert_eq!(
            bridge.try_fund_c_address_crosschain(&1u32, &tx_hash, &target, &token_id, &100i128, &sigs),
            Err(Ok(BridgeError::BelowThreshold))
        );
    }

    #[test]
    fn test_crosschain_non_relayer_rejected() {
        let env = Env::default();
        let (_bridge_id, token_id, _admin, bridge) = setup(&env);

        let sk_registered = make_signing_key([1u8; 32]);
        let sk_stranger = make_signing_key([9u8; 32]); // not registered

        bridge.add_relayer(&BytesN::from_array(&env, sk_registered.verifying_key().as_bytes()));
        bridge.set_relayer_threshold(&1u32);

        let target = soroban_sdk::Address::generate(&env);
        let tx_hash = BytesN::from_array(&env, &[0x22; 32]);

        let payload_hash = build_payload_hash(&env, 1, &tx_hash, &target, &token_id, 100);
        let sigs = Vec::from_array(&env, [make_relayer_sig(&env, &sk_stranger, &payload_hash)]);

        assert_eq!(
            bridge.try_fund_c_address_crosschain(&1u32, &tx_hash, &target, &token_id, &100i128, &sigs),
            Err(Ok(BridgeError::NotRelayer))
        );
    }

    #[test]
    fn test_add_remove_relayer_and_threshold() {
        let env = Env::default();
        let (_bridge_id, _token_id, _admin, bridge) = setup(&env);

        let pk = BytesN::from_array(&env, make_signing_key([5u8; 32]).verifying_key().as_bytes());

        bridge.add_relayer(&pk);
        assert!(bridge.query_is_relayer(&pk));

        bridge.set_relayer_threshold(&1u32);
        assert_eq!(bridge.query_relayer_threshold(), 1u32);

        // Can't remove last relayer when it would drop below threshold
        assert_eq!(
            bridge.try_remove_relayer(&pk),
            Err(Ok(BridgeError::BelowThreshold))
        );
    }
}
