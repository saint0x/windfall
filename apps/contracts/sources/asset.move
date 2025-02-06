module windfall::asset {
    use std::error;
    use std::signer;
    use std::string::{Self, String};
    use std::vector;
    use aptos_framework::account;
    use aptos_framework::coin;
    use aptos_framework::event::{Self, EventHandle};
    use aptos_framework::timestamp;
    use aptos_std::table::{Self, Table};
    use windfall::registry;
    use aptos_framework::security;

    /// Error codes
    const ENOT_INITIALIZED: u64 = 1;
    const EALREADY_INITIALIZED: u64 = 2;
    const ENOT_AUTHORIZED: u64 = 3;
    const ENOT_ACTUATOR: u64 = 4;
    const EASSET_ALREADY_EXISTS: u64 = 5;
    const EASSET_NOT_FOUND: u64 = 6;
    const EINSUFFICIENT_BALANCE: u64 = 7;
    const EINVALID_AMOUNT: u64 = 8;
    const EUSER_NOT_VERIFIED: u64 = 9;
    const EUSER_NOT_MEMBER: u64 = 10;
    const EFUND_NOT_FOUND: u64 = 11;
    const EINVALID_SHARE_TOTAL: u64 = 12;

    /// Minimum verification level required for asset operations
    const MIN_VERIFICATION_LEVEL: u8 = 1;

    const MODULE_ID: u8 = 1; // Unique identifier for the asset module

    struct Asset has store {
        symbol: String,
        name: String,
        decimals: u8,
        total_supply: u64,
        is_active: bool,
    }

    struct Balance has store {
        amount: u64,
        last_updated: u64,
    }

    struct AssetEvents has key {
        creation_events: EventHandle<AssetCreationEvent>,
        transfer_events: EventHandle<TransferEvent>,
        mint_events: EventHandle<MintEvent>,
        burn_events: EventHandle<BurnEvent>,
    }

    struct AssetData has key {
        admin: address,
        assets: Table<String, Asset>,
        balances: Table<address, Table<String, Balance>>,
    }

    struct AssetCreationEvent has drop, store {
        symbol: String,
        name: String,
        decimals: u8,
        total_supply: u64,
        creation_time: u64,
    }

    struct TransferEvent has drop, store {
        symbol: String,
        from: address,
        to: address,
        amount: u64,
        transfer_time: u64,
    }

    struct MintEvent has drop, store {
        symbol: String,
        to: address,
        amount: u64,
        mint_time: u64,
    }

    struct BurnEvent has drop, store {
        symbol: String,
        from: address,
        amount: u64,
        burn_time: u64,
    }

    struct Fund has store {
        name: String,
        description: String,
        executor: address,
        members: vector<address>,
        created_at: u64,
        metadata: Table<String, String>,
    }

    struct FundStore has key {
        funds: Table<u64, Fund>,
        fund_count: u64,
    }

    struct FundWallet has key {
        fund_id: u64,
        actuator: address,
        members: vector<Member>,
        balance: u64,
    }

    struct Member has store {
        address: address,
        ownership_share: u64,  // Basis points (1/10000)
        joined_at: u64,
    }

    struct FundEvents has key {
        investment_events: EventHandle<InvestmentEvent>,
        withdrawal_events: EventHandle<WithdrawalEvent>,
        member_update_events: EventHandle<MemberUpdateEvent>,
    }

    struct InvestmentEvent has drop, store {
        fund_id: u64,
        target_address: address,
        amount: u64,
        timestamp: u64,
    }

    struct WithdrawalEvent has drop, store {
        fund_id: u64,
        to_address: address,
        amount: u64,
        timestamp: u64,
    }

    struct MemberUpdateEvent has drop, store {
        fund_id: u64,
        member_address: address,
        new_share: u64,
        timestamp: u64,
    }

    public fun initialize(admin: &signer) {
        let admin_address = signer::address_of(admin);
        
        assert!(!exists<AssetData>(admin_address), error::already_exists(EALREADY_INITIALIZED));
        
        move_to(admin, AssetData {
            admin: admin_address,
            assets: table::new(),
            balances: table::new(),
        });

        move_to(admin, AssetEvents {
            creation_events: account::new_event_handle<AssetCreationEvent>(admin),
            transfer_events: account::new_event_handle<TransferEvent>(admin),
            mint_events: account::new_event_handle<MintEvent>(admin),
            burn_events: account::new_event_handle<BurnEvent>(admin),
        });
    }

    public entry fun create_asset(
        admin: &signer,
        symbol: String,
        name: String,
        decimals: u8,
        initial_supply: u64
    ) acquires AssetData, AssetEvents {
        let admin_address = signer::address_of(admin);
        let asset_data = borrow_global_mut<AssetData>(@windfall);
        
        assert!(admin_address == asset_data.admin, error::permission_denied(ENOT_AUTHORIZED));
        assert!(!table::contains(&asset_data.assets, symbol), 
            error::already_exists(EASSET_ALREADY_EXISTS));

        // Create new asset
        table::add(&mut asset_data.assets, symbol, Asset {
            symbol: symbol,
            name: name,
            decimals: decimals,
            total_supply: initial_supply,
            is_active: true,
        });

        // Initialize admin balance
        if (!table::contains(&asset_data.balances, admin_address)) {
            table::add(&mut asset_data.balances, admin_address, table::new());
        };
        let admin_balances = table::borrow_mut(&mut asset_data.balances, admin_address);
        table::add(admin_balances, symbol, Balance {
            amount: initial_supply,
            last_updated: timestamp::now_microseconds(),
        });

        // Emit creation event
        let events = borrow_global_mut<AssetEvents>(@windfall);
        event::emit_event(&mut events.creation_events, AssetCreationEvent {
            symbol: symbol,
            name: name,
            decimals: decimals,
            total_supply: initial_supply,
            creation_time: timestamp::now_microseconds(),
        });
    }

    public entry fun transfer(
        from: &signer,
        to: address,
        symbol: String,
        amount: u64
    ) acquires AssetData, AssetEvents {
        // Security checks
        security::assert_not_paused(MODULE_ID);
        security::start_reentrancy_protection();
        
        let from_address = signer::address_of(from);
        
        // Verify user
        assert!(registry::get_verification_level(from_address) >= MIN_VERIFICATION_LEVEL,
            error::permission_denied(EUSER_NOT_VERIFIED));
        assert!(registry::get_verification_level(to) >= MIN_VERIFICATION_LEVEL,
            error::permission_denied(EUSER_NOT_VERIFIED));

        let asset_data = borrow_global_mut<AssetData>(@windfall);
        
        // Verify asset exists and is active
        assert!(table::contains(&asset_data.assets, symbol),
            error::not_found(EASSET_NOT_FOUND));
        let asset = table::borrow(&asset_data.assets, symbol);
        assert!(asset.is_active, error::invalid_state(EASSET_NOT_FOUND));

        // Check and update sender balance
        assert!(table::contains(&asset_data.balances, from_address),
            error::not_found(EINSUFFICIENT_BALANCE));
        let from_balances = table::borrow_mut(&mut asset_data.balances, from_address);
        assert!(table::contains(from_balances, symbol),
            error::not_found(EINSUFFICIENT_BALANCE));
        let from_balance = table::borrow_mut(from_balances, symbol);
        assert!(from_balance.amount >= amount, error::invalid_argument(EINSUFFICIENT_BALANCE));
        
        // Update sender balance
        from_balance.amount = from_balance.amount - amount;
        from_balance.last_updated = timestamp::now_microseconds();

        // Initialize receiver balance table if needed
        if (!table::contains(&asset_data.balances, to)) {
            table::add(&mut asset_data.balances, to, table::new());
        };
        let to_balances = table::borrow_mut(&mut asset_data.balances, to);
        
        // Initialize or update receiver balance
        if (!table::contains(to_balances, symbol)) {
            table::add(to_balances, symbol, Balance {
                amount: amount,
                last_updated: timestamp::now_microseconds(),
            });
        } else {
            let to_balance = table::borrow_mut(to_balances, symbol);
            to_balance.amount = to_balance.amount + amount;
            to_balance.last_updated = timestamp::now_microseconds();
        };

        // Emit transfer event
        let events = borrow_global_mut<AssetEvents>(@windfall);
        event::emit_event(&mut events.transfer_events, TransferEvent {
            symbol: symbol,
            from: from_address,
            to: to,
            amount: amount,
            transfer_time: timestamp::now_microseconds(),
        });

        // End reentrancy protection
        security::end_reentrancy_protection();
    }

    public entry fun mint(
        admin: &signer,
        to: address,
        symbol: String,
        amount: u64
    ) acquires AssetData, AssetEvents {
        let admin_address = signer::address_of(admin);
        let asset_data = borrow_global_mut<AssetData>(@windfall);
        
        assert!(admin_address == asset_data.admin, error::permission_denied(ENOT_AUTHORIZED));
        assert!(table::contains(&asset_data.assets, symbol),
            error::not_found(EASSET_NOT_FOUND));

        // Update total supply
        let asset = table::borrow_mut(&mut asset_data.assets, symbol);
        asset.total_supply = asset.total_supply + amount;

        // Initialize receiver balance table if needed
        if (!table::contains(&asset_data.balances, to)) {
            table::add(&mut asset_data.balances, to, table::new());
        };
        let to_balances = table::borrow_mut(&mut asset_data.balances, to);
        
        // Initialize or update receiver balance
        if (!table::contains(to_balances, symbol)) {
            table::add(to_balances, symbol, Balance {
                amount: amount,
                last_updated: timestamp::now_microseconds(),
            });
        } else {
            let to_balance = table::borrow_mut(to_balances, symbol);
            to_balance.amount = to_balance.amount + amount;
            to_balance.last_updated = timestamp::now_microseconds();
        };

        // Emit mint event
        let events = borrow_global_mut<AssetEvents>(@windfall);
        event::emit_event(&mut events.mint_events, MintEvent {
            symbol: symbol,
            to: to,
            amount: amount,
            mint_time: timestamp::now_microseconds(),
        });
    }

    public entry fun burn(
        admin: &signer,
        from: address,
        symbol: String,
        amount: u64
    ) acquires AssetData, AssetEvents {
        let admin_address = signer::address_of(admin);
        let asset_data = borrow_global_mut<AssetData>(@windfall);
        
        assert!(admin_address == asset_data.admin, error::permission_denied(ENOT_AUTHORIZED));
        assert!(table::contains(&asset_data.assets, symbol),
            error::not_found(EASSET_NOT_FOUND));

        // Check and update balance
        assert!(table::contains(&asset_data.balances, from),
            error::not_found(EINSUFFICIENT_BALANCE));
        let from_balances = table::borrow_mut(&mut asset_data.balances, from);
        assert!(table::contains(from_balances, symbol),
            error::not_found(EINSUFFICIENT_BALANCE));
        let from_balance = table::borrow_mut(from_balances, symbol);
        assert!(from_balance.amount >= amount, error::invalid_argument(EINSUFFICIENT_BALANCE));

        // Update balance and total supply
        from_balance.amount = from_balance.amount - amount;
        from_balance.last_updated = timestamp::now_microseconds();

        let asset = table::borrow_mut(&mut asset_data.assets, symbol);
        asset.total_supply = asset.total_supply - amount;

        // Emit burn event
        let events = borrow_global_mut<AssetEvents>(@windfall);
        event::emit_event(&mut events.burn_events, BurnEvent {
            symbol: symbol,
            from: from,
            amount: amount,
            burn_time: timestamp::now_microseconds(),
        });
    }

    #[view]
    public fun get_balance(account: address, symbol: String): u64 acquires AssetData {
        let asset_data = borrow_global<AssetData>(@windfall);
        
        if (!table::contains(&asset_data.balances, account)) {
            return 0
        };
        
        let account_balances = table::borrow(&asset_data.balances, account);
        if (!table::contains(account_balances, symbol)) {
            return 0
        };
        
        let balance = table::borrow(account_balances, symbol);
        balance.amount
    }

    #[view]
    public fun get_asset_info(symbol: String): (String, String, u8, u64, bool) acquires AssetData {
        let asset_data = borrow_global<AssetData>(@windfall);
        assert!(table::contains(&asset_data.assets, symbol),
            error::not_found(EASSET_NOT_FOUND));
        
        let asset = table::borrow(&asset_data.assets, symbol);
        (
            asset.symbol,
            asset.name,
            asset.decimals,
            asset.total_supply,
            asset.is_active
        )
    }

    public fun initialize_fund_store(admin: &signer) {
        let admin_address = signer::address_of(admin);
        assert!(!exists<FundStore>(admin_address), error::already_exists(EALREADY_INITIALIZED));
        
        move_to(admin, FundStore {
            funds: table::new(),
            fund_count: 0,
        });
    }

    public entry fun create_fund(
        admin: &signer,
        name: String,
        description: String,
        executor: address,
        initial_members: vector<address>,
        metadata_keys: vector<String>,
        metadata_values: vector<String>,
    ) acquires FundStore {
        let admin_address = signer::address_of(admin);
        let fund_store = borrow_global_mut<FundStore>(@windfall);
        
        assert!(admin_address == @windfall, error::permission_denied(ENOT_AUTHORIZED));
        assert!(vector::length(&metadata_keys) == vector::length(&metadata_values), error::invalid_argument(EINVALID_AMOUNT));
        
        let fund_id = fund_store.fund_count + 1;
        let metadata = table::new();
        
        // Add all metadata key-value pairs
        let i = 0;
        let len = vector::length(&metadata_keys);
        while (i < len) {
            let key = vector::borrow(&metadata_keys, i);
            let value = vector::borrow(&metadata_values, i);
            table::add(&mut metadata, *key, *value);
            i = i + 1;
        };
        
        table::add(&mut fund_store.funds, fund_id, Fund {
            name,
            description,
            executor,
            members: initial_members,
            created_at: timestamp::now_microseconds(),
            metadata,
        });
        
        fund_store.fund_count = fund_id;
    }

    #[view]
    public fun get_fund_info(fund_id: u64): (String, String, address, vector<address>, u64, vector<String>, vector<String>) 
    acquires FundStore {
        let fund_store = borrow_global<FundStore>(@windfall);
        assert!(table::contains(&fund_store.funds, fund_id), error::not_found(EFUND_NOT_FOUND));
        
        let fund = table::borrow(&fund_store.funds, fund_id);
        let metadata_keys = table::keys(&fund.metadata);
        let metadata_values = vector::empty();
        
        let i = 0;
        let len = vector::length(&metadata_keys);
        while (i < len) {
            let key = vector::borrow(&metadata_keys, i);
            let value = table::borrow(&fund.metadata, *key);
            vector::push_back(&mut metadata_values, *value);
            i = i + 1;
        };
        
        (
            fund.name,
            fund.description,
            fund.executor,
            fund.members,
            fund.created_at,
            metadata_keys,
            metadata_values
        )
    }

    public entry fun execute_transaction(
        executor: &signer,
        fund_id: u64,
        // Add transaction parameters as needed
    ) acquires FundStore {
        let executor_address = signer::address_of(executor);
        let fund_store = borrow_global<FundStore>(@windfall);
        
        assert!(table::contains(&fund_store.funds, fund_id), error::not_found(EFUND_NOT_FOUND));
        let fund = table::borrow(&fund_store.funds, fund_id);
        assert!(fund.executor == executor_address, error::permission_denied(ENOT_AUTHORIZED));
        
        // Add transaction execution logic here
    }

    public fun initialize_fund_wallet(
        creator: &signer,
        fund_id: u64,
        actuator: address,
        initial_members: vector<address>,
        initial_shares: vector<u64>
    ) {
        let creator_addr = signer::address_of(creator);
        assert!(!exists<FundWallet>(creator_addr), error::already_exists(EALREADY_INITIALIZED));
        assert!(vector::length(&initial_members) == vector::length(&initial_shares), error::invalid_argument(EINVALID_SHARE_TOTAL));
        
        // Validate total shares equal 10000 (100%)
        let total_shares = 0u64;
        let i = 0;
        let len = vector::length(&initial_shares);
        while (i < len) {
            total_shares = total_shares + *vector::borrow(&initial_shares, i);
            i = i + 1;
        };
        assert!(total_shares == 10000, error::invalid_argument(EINVALID_SHARE_TOTAL));

        // Create member list
        let members = vector::empty<Member>();
        let i = 0;
        while (i < len) {
            vector::push_back(&mut members, Member {
                address: *vector::borrow(&initial_members, i),
                ownership_share: *vector::borrow(&initial_shares, i),
                joined_at: timestamp::now_microseconds(),
            });
            i = i + 1;
        };

        // Initialize wallet
        move_to(creator, FundWallet {
            fund_id,
            actuator,
            members,
            balance: 0,
        });

        // Initialize events
        move_to(creator, FundEvents {
            investment_events: account::new_event_handle<InvestmentEvent>(creator),
            withdrawal_events: account::new_event_handle<WithdrawalEvent>(creator),
            member_update_events: account::new_event_handle<MemberUpdateEvent>(creator),
        });
    }

    public entry fun invest(
        actuator: &signer,
        fund_addr: address,
        target: address,
        amount: u64
    ) acquires FundWallet, FundEvents {
        let actuator_addr = signer::address_of(actuator);
        let fund = borrow_global_mut<FundWallet>(fund_addr);
        
        // Verify actuator
        assert!(actuator_addr == fund.actuator, error::permission_denied(ENOT_ACTUATOR));
        assert!(fund.balance >= amount, error::invalid_argument(EINVALID_AMOUNT));

        // Execute investment
        coin::transfer<AptosCoin>(actuator, target, amount);
        fund.balance = fund.balance - amount;

        // Emit event
        let events = borrow_global_mut<FundEvents>(fund_addr);
        event::emit_event(&mut events.investment_events, InvestmentEvent {
            fund_id: fund.fund_id,
            target_address: target,
            amount,
            timestamp: timestamp::now_microseconds(),
        });
    }

    public entry fun withdraw_profits(
        actuator: &signer,
        fund_addr: address,
        amount: u64
    ) acquires FundWallet, FundEvents {
        let actuator_addr = signer::address_of(actuator);
        let fund = borrow_global_mut<FundWallet>(fund_addr);
        
        // Verify actuator
        assert!(actuator_addr == fund.actuator, error::permission_denied(ENOT_ACTUATOR));
        assert!(fund.balance >= amount, error::invalid_argument(EINVALID_AMOUNT));

        // Calculate and distribute profits
        let i = 0;
        let len = vector::length(&fund.members);
        while (i < len) {
            let member = vector::borrow(&fund.members, i);
            let member_share = (amount * member.ownership_share) / 10000;
            if (member_share > 0) {
                coin::transfer<AptosCoin>(actuator, member.address, member_share);
            };
            i = i + 1;
        };

        fund.balance = fund.balance - amount;

        // Emit event
        let events = borrow_global_mut<FundEvents>(fund_addr);
        event::emit_event(&mut events.withdrawal_events, WithdrawalEvent {
            fund_id: fund.fund_id,
            to_address: actuator_addr,
            amount,
            timestamp: timestamp::now_microseconds(),
        });
    }

    public entry fun update_member_share(
        actuator: &signer,
        fund_addr: address,
        member_addr: address,
        new_share: u64
    ) acquires FundWallet, FundEvents {
        let actuator_addr = signer::address_of(actuator);
        let fund = borrow_global_mut<FundWallet>(fund_addr);
        
        // Verify actuator
        assert!(actuator_addr == fund.actuator, error::permission_denied(ENOT_ACTUATOR));

        // Update member share
        let i = 0;
        let len = vector::length(&fund.members);
        let found = false;
        let total_shares = 0u64;

        while (i < len) {
            let member = vector::borrow_mut(&mut fund.members, i);
            if (member.address == member_addr) {
                member.ownership_share = new_share;
                found = true;
            };
            total_shares = total_shares + member.ownership_share;
            i = i + 1;
        };

        assert!(found, error::not_found(EUSER_NOT_MEMBER));
        assert!(total_shares == 10000, error::invalid_argument(EINVALID_SHARE_TOTAL));

        // Emit event
        let events = borrow_global_mut<FundEvents>(fund_addr);
        event::emit_event(&mut events.member_update_events, MemberUpdateEvent {
            fund_id: fund.fund_id,
            member_address: member_addr,
            new_share,
            timestamp: timestamp::now_microseconds(),
        });
    }

    #[view]
    public fun get_member_share(fund_addr: address, member_addr: address): u64 acquires FundWallet {
        let fund = borrow_global<FundWallet>(fund_addr);
        let i = 0;
        let len = vector::length(&fund.members);
        
        while (i < len) {
            let member = vector::borrow(&fund.members, i);
            if (member.address == member_addr) {
                return member.ownership_share
            };
            i = i + 1;
        };
        0
    }

    #[view]
    public fun get_fund_balance(fund_addr: address): u64 acquires FundWallet {
        borrow_global<FundWallet>(fund_addr).balance
    }
} 
