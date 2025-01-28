module windfall::storage {
    use std::error;
    use aptos_std::table::{Self, Table};
    use windfall::types::{Position, ShareAllocation, ProposalData};

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

    public(friend) fun initialize_position_store(admin: &signer) {
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
        let position_id = position.id;
        
        assert!(!table::contains(&position_store.positions, position_id),
            error::already_exists(0));
            
        table::add(&mut position_store.positions, position_id, position);
        table::add(&mut position_store.shares, position_id, table::new());
    }

    public(friend) fun get_position(position_id: u64): Position acquires PositionStore {
        let position_store = borrow_global<PositionStore>(@windfall);
        assert!(table::contains(&position_store.positions, position_id),
            error::not_found(0));
        *table::borrow(&position_store.positions, position_id)
    }

    public(friend) fun update_position(position: Position) acquires PositionStore {
        let position_store = borrow_global_mut<PositionStore>(@windfall);
        assert!(table::contains(&position_store.positions, position.id),
            error::not_found(0));
        *table::borrow_mut(&mut position_store.positions, position.id) = position;
    }

    public(friend) fun store_share_allocation(
        position_id: u64,
        allocation: ShareAllocation
    ) acquires PositionStore {
        let position_store = borrow_global_mut<PositionStore>(@windfall);
        let shares = table::borrow_mut(&mut position_store.shares, position_id);
        table::add(shares, allocation.user, allocation);
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