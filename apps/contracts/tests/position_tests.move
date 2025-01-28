#[test_only]
module windfall::position_tests {
    use std::signer;
    use aptos_framework::account;
    use aptos_framework::timestamp;
    use windfall::position;
    use windfall::registry;

    const ASSET_SYMBOL: vector<u8> = b"BTC/USD";
    const POSITION_SIZE: u64 = 100000000; // 1 BTC
    const ENTRY_PRICE: u64 = 4000000; // $40,000

    fun setup_test(aptos_framework: &signer, admin: &signer) {
        timestamp::set_time_has_started_for_testing(aptos_framework);
        let admin_addr = signer::address_of(admin);
        account::create_account_for_test(admin_addr);
        registry::initialize(admin);
        position::initialize(admin);
    }

    #[test(aptos_framework = @0x1, admin = @0x123, actuator = @0x234)]
    public entry fun test_set_actuator(
        aptos_framework: &signer,
        admin: &signer,
        actuator: &signer
    ) {
        setup_test(aptos_framework, admin);
        let actuator_addr = signer::address_of(actuator);
        account::create_account_for_test(actuator_addr);
        registry::register(actuator);
        position::set_actuator(admin, actuator_addr);
        assert!(position::get_actuator() == actuator_addr, 0);
    }

    #[test(aptos_framework = @0x1, admin = @0x123, actuator = @0x234, user = @0x345)]
    public entry fun test_open_position(
        aptos_framework: &signer,
        admin: &signer,
        actuator: &signer,
        user: &signer
    ) {
        setup_test(aptos_framework, admin);
        
        // Setup actuator and user
        let actuator_addr = signer::address_of(actuator);
        let user_addr = signer::address_of(user);
        account::create_account_for_test(actuator_addr);
        account::create_account_for_test(user_addr);
        registry::register(actuator);
        registry::register(user);
        position::set_actuator(admin, actuator_addr);

        // Open position
        position::open_position(
            actuator,
            ASSET_SYMBOL,
            POSITION_SIZE,
            ENTRY_PRICE,
            true // is_long
        );

        // Verify position
        let (size, price, is_long) = position::get_position_info(0);
        assert!(size == POSITION_SIZE, 1);
        assert!(price == ENTRY_PRICE, 2);
        assert!(is_long == true, 3);
    }

    #[test(aptos_framework = @0x1, admin = @0x123, actuator = @0x234, user1 = @0x345, user2 = @0x456)]
    public entry fun test_share_allocation(
        aptos_framework: &signer,
        admin: &signer,
        actuator: &signer,
        user1: &signer,
        user2: &signer
    ) {
        setup_test(aptos_framework, admin);
        
        // Setup accounts
        let actuator_addr = signer::address_of(actuator);
        let user1_addr = signer::address_of(user1);
        let user2_addr = signer::address_of(user2);
        account::create_account_for_test(actuator_addr);
        account::create_account_for_test(user1_addr);
        account::create_account_for_test(user2_addr);
        registry::register(actuator);
        registry::register(user1);
        registry::register(user2);
        position::set_actuator(admin, actuator_addr);

        // Open position
        position::open_position(
            actuator,
            ASSET_SYMBOL,
            POSITION_SIZE,
            ENTRY_PRICE,
            true
        );

        // Allocate shares
        position::allocate_shares(actuator, 0, user1_addr, 6000); // 60%
        position::allocate_shares(actuator, 0, user2_addr, 4000); // 40%

        // Verify shares
        assert!(position::get_user_shares(0, user1_addr) == 6000, 4);
        assert!(position::get_user_shares(0, user2_addr) == 4000, 5);
    }
} 