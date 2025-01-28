#[test_only]
module windfall::registry_tests {
    use std::signer;
    use aptos_framework::account;
    use aptos_framework::timestamp;
    use windfall::registry;

    fun setup_test(aptos_framework: &signer) {
        timestamp::set_time_has_started_for_testing(aptos_framework);
    }

    #[test(aptos_framework = @0x1, admin = @0x123)]
    public entry fun test_initialize_registry(aptos_framework: &signer, admin: &signer) {
        setup_test(aptos_framework);
        let admin_addr = signer::address_of(admin);
        account::create_account_for_test(admin_addr);
        registry::initialize(admin);
        assert!(registry::is_initialized(), 0);
    }

    #[test(aptos_framework = @0x1, admin = @0x123, user = @0x456)]
    public entry fun test_register_user(aptos_framework: &signer, admin: &signer, user: &signer) {
        setup_test(aptos_framework);
        let admin_addr = signer::address_of(admin);
        let user_addr = signer::address_of(user);
        account::create_account_for_test(admin_addr);
        account::create_account_for_test(user_addr);
        registry::initialize(admin);
        registry::register(user);
        assert!(registry::is_active(user_addr), 1);
    }
} 