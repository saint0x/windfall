module windfall::types {
    use std::string::String;
    
    friend windfall::storage;
    friend windfall::position;
    friend windfall::governance;

    /// Core position data structure
    struct Position has store, copy, drop {
        id: u64,
        asset_symbol: String,
        total_size: u64,
        entry_price: u64,
        entry_timestamp: u64,
        is_active: bool,
        total_shares: u64,
    }

    /// Share allocation data
    struct ShareAllocation has store, copy, drop {
        user: address,
        shares: u64,
        entry_timestamp: u64,
        last_updated: u64,
    }

    /// Proposal data with minimal exposure
    struct ProposalData has store, copy, drop {
        id: u64,
        proposer: address,
        proposal_type: u8,
        status: u8,
        creation_time: u64,
    }

    // Error codes
    const INVALID_POSITION: u64 = 1;
    const INVALID_SHARES: u64 = 2;
    const UNAUTHORIZED: u64 = 3;
    const INVALID_STATE: u64 = 4;

    // Getter functions for Position
    public fun get_position_id(position: &Position): u64 {
        position.id
    }

    public fun get_position_asset_symbol(position: &Position): String {
        position.asset_symbol
    }

    public fun get_position_size(position: &Position): u64 {
        position.total_size
    }

    public fun get_position_entry_price(position: &Position): u64 {
        position.entry_price
    }

    public fun get_position_entry_timestamp(position: &Position): u64 {
        position.entry_timestamp
    }

    public fun get_position_is_active(position: &Position): bool {
        position.is_active
    }

    public fun get_position_total_shares(position: &Position): u64 {
        position.total_shares
    }

    // Getter functions for ShareAllocation
    public fun get_share_allocation_user(allocation: &ShareAllocation): address {
        allocation.user
    }

    public fun get_share_allocation_shares(allocation: &ShareAllocation): u64 {
        allocation.shares
    }

    public fun get_share_allocation_entry_timestamp(allocation: &ShareAllocation): u64 {
        allocation.entry_timestamp
    }

    public fun get_share_allocation_last_updated(allocation: &ShareAllocation): u64 {
        allocation.last_updated
    }

    // Constructor for Position
    public fun create_position(
        id: u64,
        asset_symbol: String,
        total_size: u64,
        entry_price: u64,
        entry_timestamp: u64,
        is_active: bool,
        total_shares: u64
    ): Position {
        Position {
            id,
            asset_symbol,
            total_size,
            entry_price,
            entry_timestamp,
            is_active,
            total_shares,
        }
    }

    // Constructor for ShareAllocation
    public fun create_share_allocation(
        user: address,
        shares: u64,
        entry_timestamp: u64,
        last_updated: u64
    ): ShareAllocation {
        ShareAllocation {
            user,
            shares,
            entry_timestamp,
            last_updated,
        }
    }
} 