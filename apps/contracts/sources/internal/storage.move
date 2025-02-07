module windfall::storage {
    use std::error;
    use aptos_std::table::{Self, Table};
    use windfall::types::{Self, Position, ShareAllocation, ProposalData};

    /// Error codes
    const ENOT_FOUND: u64 = 1;
    const EALREADY_EXISTS: u64 = 2;
    const ENOT_AUTHORIZED: u64 = 3;
    const EINVALID_STATE: u64 = 4;

    friend windfall::position;
    friend windfall::governance;

    /// Core storage for positions
    struct PositionStore has key {
        positions: Table<u64, Position>,
        shares: Table<u64, Table<address, ShareAllocation>>,
        next_position_id: u64,
    }

    /// Core storage for governance
    struct GovernanceStore has key {
        proposals: Table<u64, ProposalData>,
        votes: Table<u64, Table<address, bool>>,
        next_proposal_id: u64,
    }

    public fun initialize(admin: &signer) {
        move_to(admin, PositionStore {
            positions: table::new(),
            shares: table::new(),
            next_position_id: 0,
        });
    }

    public(friend) fun initialize_governance_store(admin: &signer) {
        move_to(admin, GovernanceStore {
            proposals: table::new(),
            votes: table::new(),
            next_proposal_id: 0,
        });
    }

    public(friend) fun store_position(position: Position) acquires PositionStore {
        let position_store = borrow_global_mut<PositionStore>(@windfall);
        let position_id = types::get_position_id(&position);
        
        assert!(!table::contains(&position_store.positions, position_id),
            error::already_exists(EALREADY_EXISTS));
        
        table::add(&mut position_store.positions, position_id, position);
        
        // Initialize shares table for this position
        table::add(&mut position_store.shares, position_id, table::new());
    }

    public(friend) fun update_position(position: Position) acquires PositionStore {
        let position_store = borrow_global_mut<PositionStore>(@windfall);
        let position_id = types::get_position_id(&position);
        
        assert!(table::contains(&position_store.positions, position_id),
            error::not_found(ENOT_FOUND));
        
        *table::borrow_mut(&mut position_store.positions, position_id) = position;
    }

    public(friend) fun store_share_allocation(
        position_id: u64,
        allocation: ShareAllocation
    ) acquires PositionStore {
        let position_store = borrow_global_mut<PositionStore>(@windfall);
        
        assert!(table::contains(&position_store.shares, position_id),
            error::not_found(ENOT_FOUND));
        
        let shares = table::borrow_mut(&mut position_store.shares, position_id);
        let user = types::get_share_allocation_user(&allocation);
        
        assert!(!table::contains(shares, user),
            error::already_exists(EALREADY_EXISTS));
        
        table::add(shares, user, allocation);
    }

    #[view]
    public fun get_position(position_id: u64): Position acquires PositionStore {
        let position_store = borrow_global<PositionStore>(@windfall);
        assert!(table::contains(&position_store.positions, position_id),
            error::not_found(ENOT_FOUND));
        *table::borrow(&position_store.positions, position_id)
    }

    #[view]
    public fun get_share_allocation(
        position_id: u64,
        user: address
    ): ShareAllocation acquires PositionStore {
        let position_store = borrow_global<PositionStore>(@windfall);
        
        assert!(table::contains(&position_store.shares, position_id),
            error::not_found(ENOT_FOUND));
        
        let shares = table::borrow(&position_store.shares, position_id);
        assert!(table::contains(shares, user),
            error::not_found(ENOT_FOUND));
        
        *table::borrow(shares, user)
    }

    public(friend) fun get_next_position_id(): u64 acquires PositionStore {
        let position_store = borrow_global_mut<PositionStore>(@windfall);
        let id = position_store.next_position_id;
        position_store.next_position_id = id + 1;
        id
    }

    public(friend) fun get_next_proposal_id(): u64 acquires GovernanceStore {
        let governance_store = borrow_global_mut<GovernanceStore>(@windfall);
        let id = governance_store.next_proposal_id;
        governance_store.next_proposal_id = id + 1;
        id
    }
} 