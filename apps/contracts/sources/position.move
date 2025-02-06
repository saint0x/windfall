module windfall::position {
    use std::error;
    use std::signer;
    use std::string::String;
    use aptos_framework::account;
    use aptos_framework::event::{Self, EventHandle};
    use aptos_framework::timestamp;
    use aptos_std::table::{Self, Table};
    use windfall::security;

    /// Error codes
    const ENOT_INITIALIZED: u64 = 1;
    const EALREADY_INITIALIZED: u64 = 2;
    const ENOT_AUTHORIZED: u64 = 3;
    const EPOSITION_NOT_FOUND: u64 = 4;
    const EINVALID_SHARE_AMOUNT: u64 = 5;
    const EINVALID_POSITION_SIZE: u64 = 6;
    const EINSUFFICIENT_SHARES: u64 = 7;
    const EINVALID_AMOUNT: u64 = 8;
    const EINVALID_PRICE: u64 = 9;
    const EINSUFFICIENT_BALANCE: u64 = 10;
    const EPOSITION_CLOSED: u64 = 11;

    const MODULE_ID: u8 = 3; // Unique identifier for position module

    struct Position has store {
        id: u64,
        asset_symbol: String,
        total_size: u64,
        entry_price: u64,
        entry_timestamp: u64,
        is_active: bool,
        total_shares: u64,  // Total shares (fixed point with 6 decimals)
    }

    struct UserShare has store {
        shares: u64,        // User's share amount (fixed point with 6 decimals)
        entry_timestamp: u64,
        last_updated: u64,
    }

    struct PositionData has key {
        admin: address,
        actuator: address,  // Address authorized to execute trades
        positions: Table<u64, Position>,
        user_shares: Table<address, Table<u64, UserShare>>,  // User -> Position ID -> Share
        next_position_id: u64,
    }

    struct PositionEvents has key {
        position_opened_events: EventHandle<PositionOpenedEvent>,
        position_closed_events: EventHandle<PositionClosedEvent>,
        share_allocation_events: EventHandle<ShareAllocationEvent>,
        share_transfer_events: EventHandle<ShareTransferEvent>,
    }

    struct PositionOpenedEvent has drop, store {
        position_id: u64,
        asset_symbol: String,
        size: u64,
        entry_price: u64,
        timestamp: u64,
    }

    struct PositionClosedEvent has drop, store {
        position_id: u64,
        exit_price: u64,
        pnl: u64,
        timestamp: u64,
    }

    struct ShareAllocationEvent has drop, store {
        position_id: u64,
        user_address: address,
        shares: u64,
        timestamp: u64,
    }

    struct ShareTransferEvent has drop, store {
        position_id: u64,
        from_address: address,
        to_address: address,
        shares: u64,
        timestamp: u64,
    }

    public fun initialize(admin: &signer) {
        let admin_address = signer::address_of(admin);
        
        assert!(!exists<PositionData>(admin_address), error::already_exists(EALREADY_INITIALIZED));
        
        move_to(admin, PositionData {
            admin: admin_address,
            actuator: admin_address,  // Initially set to admin
            positions: table::new(),
            user_shares: table::new(),
            next_position_id: 0,
        });

        move_to(admin, PositionEvents {
            position_opened_events: account::new_event_handle<PositionOpenedEvent>(admin),
            position_closed_events: account::new_event_handle<PositionClosedEvent>(admin),
            share_allocation_events: account::new_event_handle<ShareAllocationEvent>(admin),
            share_transfer_events: account::new_event_handle<ShareTransferEvent>(admin),
        });
    }

    public entry fun set_actuator(
        admin: &signer,
        new_actuator: address
    ) acquires PositionData {
        let position_data = borrow_global_mut<PositionData>(@windfall);
        assert!(signer::address_of(admin) == position_data.admin, error::permission_denied(ENOT_AUTHORIZED));
        position_data.actuator = new_actuator;
    }

    public entry fun open_position(
        trader: &signer,
        asset_id: u64,
        size: u64,
        entry_price: u64,
        is_long: bool
    ) acquires PositionData, PositionEvents {
        // Security checks
        security::assert_not_paused(MODULE_ID);
        security::start_reentrancy_protection();

        let position_data = borrow_global_mut<PositionData>(@windfall);
        assert!(signer::address_of(trader) == position_data.actuator, 
            error::permission_denied(ENOT_AUTHORIZED));

        let position_id = position_data.next_position_id;
        let current_time = timestamp::now_microseconds();

        table::add(&mut position_data.positions, position_id, Position {
            id: position_id,
            asset_symbol: asset_id.to_string(),
            total_size: size,
            entry_price: entry_price,
            entry_timestamp: current_time,
            is_active: true,
            total_shares: 1000000, // 1.0 in fixed point (6 decimals)
        });

        position_data.next_position_id = position_id + 1;

        // Emit position opened event
        let events = borrow_global_mut<PositionEvents>(@windfall);
        event::emit_event(&mut events.position_opened_events, PositionOpenedEvent {
            position_id,
            asset_symbol: asset_id.to_string(),
            size,
            entry_price,
            timestamp: current_time,
        });

        security::end_reentrancy_protection();
    }

    public entry fun allocate_shares(
        actuator: &signer,
        position_id: u64,
        user_address: address,
        shares: u64
    ) acquires PositionData, PositionEvents {
        let position_data = borrow_global_mut<PositionData>(@windfall);
        assert!(signer::address_of(actuator) == position_data.actuator,
            error::permission_denied(ENOT_AUTHORIZED));
        
        let position = table::borrow(&position_data.positions, position_id);
        assert!(position.is_active, error::invalid_state(EPOSITION_NOT_FOUND));
        assert!(shares <= position.total_shares, error::invalid_argument(EINVALID_SHARE_AMOUNT));

        let current_time = timestamp::now_microseconds();

        // Initialize user shares table if needed
        if (!table::contains(&position_data.user_shares, user_address)) {
            table::add(&mut position_data.user_shares, user_address, table::new());
        };

        let user_positions = table::borrow_mut(&mut position_data.user_shares, user_address);
        assert!(!table::contains(user_positions, position_id),
            error::already_exists(EINVALID_SHARE_AMOUNT));

        table::add(user_positions, position_id, UserShare {
            shares,
            entry_timestamp: current_time,
            last_updated: current_time,
        });

        // Emit share allocation event
        let events = borrow_global_mut<PositionEvents>(@windfall);
        event::emit_event(&mut events.share_allocation_events, ShareAllocationEvent {
            position_id,
            user_address,
            shares,
            timestamp: current_time,
        });
    }

    public entry fun transfer_shares(
        from: &signer,
        to: address,
        position_id: u64,
        shares: u64
    ) acquires PositionData, PositionEvents {
        let from_address = signer::address_of(from);
        let position_data = borrow_global_mut<PositionData>(@windfall);
        
        // Verify position exists and is active
        let position = table::borrow(&position_data.positions, position_id);
        assert!(position.is_active, error::invalid_state(EPOSITION_NOT_FOUND));

        // Verify and update sender shares
        assert!(table::contains(&position_data.user_shares, from_address),
            error::not_found(EINSUFFICIENT_SHARES));
        let from_positions = table::borrow_mut(&mut position_data.user_shares, from_address);
        assert!(table::contains(from_positions, position_id),
            error::not_found(EINSUFFICIENT_SHARES));
        
        let from_share = table::borrow_mut(from_positions, position_id);
        assert!(from_share.shares >= shares, error::invalid_argument(EINSUFFICIENT_SHARES));
        
        let current_time = timestamp::now_microseconds();
        from_share.shares = from_share.shares - shares;
        from_share.last_updated = current_time;

        // Initialize or update receiver shares
        if (!table::contains(&position_data.user_shares, to)) {
            table::add(&mut position_data.user_shares, to, table::new());
        };
        let to_positions = table::borrow_mut(&mut position_data.user_shares, to);
        
        if (!table::contains(to_positions, position_id)) {
            table::add(to_positions, position_id, UserShare {
                shares,
                entry_timestamp: current_time,
                last_updated: current_time,
            });
        } else {
            let to_share = table::borrow_mut(to_positions, position_id);
            to_share.shares = to_share.shares + shares;
            to_share.last_updated = current_time;
        };

        // Emit transfer event
        let events = borrow_global_mut<PositionEvents>(@windfall);
        event::emit_event(&mut events.share_transfer_events, ShareTransferEvent {
            position_id,
            from_address,
            to_address: to,
            shares,
            timestamp: current_time,
        });
    }

    #[view]
    public fun get_position_info(position_id: u64): (String, u64, u64, u64, bool) acquires PositionData {
        let position_data = borrow_global<PositionData>(@windfall);
        assert!(table::contains(&position_data.positions, position_id),
            error::not_found(EPOSITION_NOT_FOUND));
        
        let position = table::borrow(&position_data.positions, position_id);
        (
            position.asset_symbol,
            position.total_size,
            position.entry_price,
            position.total_shares,
            position.is_active
        )
    }

    #[view]
    public fun get_user_shares(user_address: address, position_id: u64): u64 acquires PositionData {
        let position_data = borrow_global<PositionData>(@windfall);
        
        if (!table::contains(&position_data.user_shares, user_address)) {
            return 0
        };
        
        let user_positions = table::borrow(&position_data.user_shares, user_address);
        if (!table::contains(user_positions, position_id)) {
            return 0
        };
        
        let share = table::borrow(user_positions, position_id);
        share.shares
    }

    public entry fun close_position(
        trader: &signer,
        position_id: u64,
        exit_price: u64
    ) acquires PositionData, PositionEvents {
        // Security checks
        security::assert_not_paused(MODULE_ID);
        security::start_reentrancy_protection();

        let position_data = borrow_global_mut<PositionData>(@windfall);
        assert!(signer::address_of(trader) == position_data.actuator, 
            error::permission_denied(ENOT_AUTHORIZED));

        let position = table::borrow(&position_data.positions, position_id);
        assert!(position.is_active, error::invalid_state(EPOSITION_CLOSED));

        let current_time = timestamp::now_microseconds();

        let pnl = (exit_price - position.entry_price) * position.total_size / position.entry_price;

        table::remove(&mut position_data.positions, position_id);

        // Emit position closed event
        let events = borrow_global_mut<PositionEvents>(@windfall);
        event::emit_event(&mut events.position_closed_events, PositionClosedEvent {
            position_id,
            exit_price,
            pnl,
            timestamp: current_time,
        });

        security::end_reentrancy_protection();
    }

    public entry fun modify_position(
        trader: &signer,
        position_id: u64,
        new_size: u64,
        new_price: u64
    ) acquires PositionData, PositionEvents {
        // Security checks
        security::assert_not_paused(MODULE_ID);
        security::start_reentrancy_protection();

        let position_data = borrow_global_mut<PositionData>(@windfall);
        assert!(signer::address_of(trader) == position_data.actuator, 
            error::permission_denied(ENOT_AUTHORIZED));

        let position = table::borrow(&position_data.positions, position_id);
        assert!(position.is_active, error::invalid_state(EPOSITION_CLOSED));

        let current_time = timestamp::now_microseconds();

        position.total_size = new_size;
        position.entry_price = new_price;
        position.entry_timestamp = current_time;

        // Emit position modified event
        let events = borrow_global_mut<PositionEvents>(@windfall);
        event::emit_event(&mut events.position_closed_events, PositionClosedEvent {
            position_id,
            exit_price: new_price,
            pnl: 0, // Assuming pnl is not provided in the new_price
            timestamp: current_time,
        });

        security::end_reentrancy_protection();
    }

    public entry fun liquidate_position(
        liquidator: &signer,
        position_id: u64,
        liquidation_price: u64
    ) acquires PositionData, PositionEvents {
        // Security checks
        security::assert_not_paused(MODULE_ID);
        security::start_reentrancy_protection();

        let position_data = borrow_global_mut<PositionData>(@windfall);
        assert!(signer::address_of(liquidator) == position_data.actuator, 
            error::permission_denied(ENOT_AUTHORIZED));

        let position = table::borrow(&position_data.positions, position_id);
        assert!(position.is_active, error::invalid_state(EPOSITION_CLOSED));

        let current_time = timestamp::now_microseconds();

        let pnl = (liquidation_price - position.entry_price) * position.total_size / position.entry_price;

        table::remove(&mut position_data.positions, position_id);

        // Emit position liquidated event
        let events = borrow_global_mut<PositionEvents>(@windfall);
        event::emit_event(&mut events.position_closed_events, PositionClosedEvent {
            position_id,
            exit_price: liquidation_price,
            pnl,
            timestamp: current_time,
        });

        security::end_reentrancy_protection();
    }
} 