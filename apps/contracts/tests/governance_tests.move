#[test_only]
module windfall::governance_tests {
    use std::signer;
    use std::string::utf8;
    use aptos_framework::account;
    use aptos_framework::timestamp;
    use windfall::governance;
    use windfall::registry;
    use windfall::position;

    fun setup_test(aptos_framework: &signer, admin: &signer) {
        timestamp::set_time_has_started_for_testing(aptos_framework);
        let admin_addr = signer::address_of(admin);
        account::create_account_for_test(admin_addr);
        registry::initialize(admin);
        position::initialize(admin);
        governance::initialize(admin);
    }

    #[test(aptos_framework = @0x1, admin = @0x123, member1 = @0x234, member2 = @0x345)]
    public entry fun test_trade_proposal_flow(
        aptos_framework: &signer,
        admin: &signer,
        member1: &signer,
        member2: &signer
    ) {
        setup_test(aptos_framework, admin);
        
        // Setup members
        let member1_addr = signer::address_of(member1);
        let member2_addr = signer::address_of(member2);
        account::create_account_for_test(member1_addr);
        account::create_account_for_test(member2_addr);
        registry::register(member1);
        registry::register(member2);

        // Create trade proposal
        governance::create_trade_proposal(
            member1,
            utf8(b"BTC/USD"),
            100000000, // 1 BTC
            4000000,  // $40,000
            true,     // entry
            utf8(b"Long BTC at 40k")
        );

        // Check proposal exists
        let (proposal_type, votes_yes, votes_no, _, executed) = governance::get_proposal_info(0);
        assert!(proposal_type == 2, 0); // PROPOSAL_TYPE_TRADE = 2
        assert!(votes_yes == 0, 1);
        assert!(votes_no == 0, 2);
        assert!(!executed, 3);

        // Vote on proposal
        governance::vote(member1, 0, true);  // Yes vote
        governance::vote(member2, 0, true);  // Yes vote

        // Verify votes
        let (_, votes_yes, votes_no, _, _) = governance::get_proposal_info(0);
        assert!(votes_yes == 2, 4);
        assert!(votes_no == 0, 5);
        assert!(governance::has_voted(0, member1_addr), 6);
        assert!(governance::has_voted(0, member2_addr), 7);
    }

    #[test(aptos_framework = @0x1, admin = @0x123, member1 = @0x234, member2 = @0x345)]
    #[expected_failure(abort_code = 9)] // EINSUFFICIENT_VOTES
    public entry fun test_emergency_veto(
        aptos_framework: &signer,
        admin: &signer,
        member1: &signer,
        member2: &signer
    ) {
        setup_test(aptos_framework, admin);
        
        // Setup members
        let member1_addr = signer::address_of(member1);
        let member2_addr = signer::address_of(member2);
        account::create_account_for_test(member1_addr);
        account::create_account_for_test(member2_addr);
        registry::register(member1);
        registry::register(member2);

        // Create proposal
        governance::create_trade_proposal(
            member1,
            utf8(b"BTC/USD"),
            100000000,
            4000000,
            true,
            utf8(b"Long BTC at 40k")
        );

        // Try emergency veto without sufficient votes
        governance::emergency_veto(member2, 0);
    }
} 