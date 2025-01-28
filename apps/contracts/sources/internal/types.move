module windfall::types {
    use std::string::String;
    
    friend windfall::storage;
    friend windfall::position;
    friend windfall::governance;

    /// Core position data structure
    struct Position has store {
        id: u64,
        asset_symbol: String,
        size: u64,
        entry_price: u64,
        is_long: bool,
        is_active: bool,
        total_shares: u64,
    }

    /// Share allocation data
    struct ShareAllocation has store {
        user: address,
        shares: u64,
        entry_time: u64,
    }

    /// Proposal data with minimal exposure
    struct ProposalData has store {
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

    #[view]
    public fun create_position(
        id: u64,
        asset_symbol: String,
        size: u64,
        price: u64,
        is_long: bool
    ): Position {
        Position {
            id,
            asset_symbol,
            size,
            entry_price: price,
            is_long,
            is_active: true,
            total_shares: 0,
        }
    }

    #[view]
    public fun create_share_allocation(
        user: address,
        shares: u64,
        entry_time: u64
    ): ShareAllocation {
        ShareAllocation {
            user,
            shares,
            entry_time,
        }
    }
} 