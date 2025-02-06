module windfall::governance {
    use std::error;
    use std::signer;
    use std::string::String;
    use aptos_framework::account;
    use aptos_framework::event::{Self, EventHandle};
    use aptos_framework::timestamp;
    use aptos_std::table::{Self, Table};
    use windfall::registry;
    use windfall::security;

    /// Error codes
    const ENOT_INITIALIZED: u64 = 1;
    const EALREADY_INITIALIZED: u64 = 2;
    const ENOT_AUTHORIZED: u64 = 3;
    const ENOT_MEMBER: u64 = 4;
    const EALREADY_VOTED: u64 = 5;
    const EPROPOSAL_NOT_FOUND: u64 = 6;
    const EPROPOSAL_EXPIRED: u64 = 7;
    const EPROPOSAL_ALREADY_EXECUTED: u64 = 8;
    const EINSUFFICIENT_VOTES: u64 = 9;
    const EINVALID_QUORUM: u64 = 10;

    const PROPOSAL_DURATION: u64 = 86400000000; // 24 hours in microseconds
    const EMERGENCY_VETO_THRESHOLD: u64 = 30;   // 30% for emergency vetoes

    const MODULE_ID: u8 = 2; // Unique identifier for governance module

    struct ProposalType has store, copy, drop {
        code: u8,
    }

    const PROPOSAL_TYPE_ACTUATOR: u8 = 1;
    const PROPOSAL_TYPE_TRADE: u8 = 2;
    const PROPOSAL_TYPE_VETO: u8 = 3;
    const PROPOSAL_TYPE_PARAMETER: u8 = 4;

    struct Proposal has store {
        id: u64,
        proposer: address,
        proposal_type: ProposalType,
        description: String,
        start_time: u64,
        end_time: u64,
        executed: bool,
        votes_yes: u64,
        votes_no: u64,
        total_eligible_votes: u64,
        payload: vector<u8>,  // Serialized data for execution
    }

    struct Vote has store {
        voted: bool,
        vote: bool,  // true = yes, false = no
        time: u64,
    }

    struct GovernanceConfig has store {
        quorum_threshold: u64,     // Percentage (1-100) needed for proposal to pass
        veto_threshold: u64,       // Percentage needed for veto
        min_voting_period: u64,    // Minimum time in microseconds
    }

    struct GovernanceData has key {
        admin: address,
        config: GovernanceConfig,
        proposals: Table<u64, Proposal>,
        votes: Table<u64, Table<address, Vote>>,  // proposal_id -> voter -> vote
        next_proposal_id: u64,
    }

    struct GovernanceEvents has key {
        proposal_created_events: EventHandle<ProposalCreatedEvent>,
        vote_events: EventHandle<VoteEvent>,
        proposal_executed_events: EventHandle<ProposalExecutedEvent>,
        emergency_veto_events: EventHandle<EmergencyVetoEvent>,
    }

    struct ProposalCreatedEvent has drop, store {
        proposal_id: u64,
        proposer: address,
        proposal_type: u8,
        timestamp: u64,
    }

    struct VoteEvent has drop, store {
        proposal_id: u64,
        voter: address,
        vote: bool,
        timestamp: u64,
    }

    struct ProposalExecutedEvent has drop, store {
        proposal_id: u64,
        executed_by: address,
        success: bool,
        timestamp: u64,
    }

    struct EmergencyVetoEvent has drop, store {
        proposal_id: u64,
        initiator: address,
        timestamp: u64,
    }

    struct TradeInfo has store, drop {
        asset_symbol: String,
        size: u64,
        price: u64,
        is_entry: bool,  // true = entry, false = exit
    }

    public fun initialize(admin: &signer) {
        let admin_address = signer::address_of(admin);
        
        assert!(!exists<GovernanceData>(admin_address), error::already_exists(EALREADY_INITIALIZED));
        
        move_to(admin, GovernanceData {
            admin: admin_address,
            config: GovernanceConfig {
                quorum_threshold: 51,  // 51% for normal proposals
                veto_threshold: 30,    // 30% for vetoes
                min_voting_period: 86400000000, // 24 hours
            },
            proposals: table::new(),
            votes: table::new(),
            next_proposal_id: 0,
        });

        move_to(admin, GovernanceEvents {
            proposal_created_events: account::new_event_handle<ProposalCreatedEvent>(admin),
            vote_events: account::new_event_handle<VoteEvent>(admin),
            proposal_executed_events: account::new_event_handle<ProposalExecutedEvent>(admin),
            emergency_veto_events: account::new_event_handle<EmergencyVetoEvent>(admin),
        });
    }

    public entry fun create_actuator_proposal(
        proposer: &signer,
        new_actuator: address,
        description: String
    ) acquires GovernanceData, GovernanceEvents {
        // Security checks
        security::assert_not_paused(MODULE_ID);
        security::start_reentrancy_protection();

        let proposer_address = signer::address_of(proposer);
        assert!(registry::is_active(proposer_address), error::permission_denied(ENOT_MEMBER));

        let governance_data = borrow_global_mut<GovernanceData>(@windfall);
        let proposal_id = governance_data.next_proposal_id;
        let current_time = timestamp::now_microseconds();

        let proposal = Proposal {
            id: proposal_id,
            proposer: proposer_address,
            proposal_type: ProposalType { code: PROPOSAL_TYPE_ACTUATOR },
            description,
            start_time: current_time,
            end_time: current_time + PROPOSAL_DURATION,
            executed: false,
            votes_yes: 0,
            votes_no: 0,
            total_eligible_votes: registry::get_active_users(),
            payload: std::bcs::to_bytes(&new_actuator),
        };

        table::add(&mut governance_data.proposals, proposal_id, proposal);
        table::add(&mut governance_data.votes, proposal_id, table::new());
        governance_data.next_proposal_id = proposal_id + 1;

        // Emit event
        let events = borrow_global_mut<GovernanceEvents>(@windfall);
        event::emit_event(&mut events.proposal_created_events, ProposalCreatedEvent {
            proposal_id,
            proposer: proposer_address,
            proposal_type: PROPOSAL_TYPE_ACTUATOR,
            timestamp: current_time,
        });

        security::end_reentrancy_protection();
    }

    public entry fun vote(
        voter: &signer,
        proposal_id: u64,
        vote: bool
    ) acquires GovernanceData, GovernanceEvents {
        // Security checks
        security::assert_not_paused(MODULE_ID);
        security::start_reentrancy_protection();

        let voter_address = signer::address_of(voter);
        assert!(registry::is_active(voter_address), error::permission_denied(ENOT_MEMBER));

        let governance_data = borrow_global_mut<GovernanceData>(@windfall);
        assert!(table::contains(&governance_data.proposals, proposal_id), 
            error::not_found(EPROPOSAL_NOT_FOUND));

        let proposal = table::borrow_mut(&mut governance_data.proposals, proposal_id);
        let current_time = timestamp::now_microseconds();
        
        assert!(current_time <= proposal.end_time, error::invalid_state(EPROPOSAL_EXPIRED));
        assert!(!proposal.executed, error::invalid_state(EPROPOSAL_ALREADY_EXECUTED));

        let votes = table::borrow_mut(&mut governance_data.votes, proposal_id);
        assert!(!table::contains(votes, voter_address), error::invalid_state(EALREADY_VOTED));

        table::add(votes, voter_address, Vote {
            voted: true,
            vote,
            time: current_time,
        });

        if (vote) {
            proposal.votes_yes = proposal.votes_yes + 1;
        } else {
            proposal.votes_no = proposal.votes_no + 1;
        };

        // Emit vote event
        let events = borrow_global_mut<GovernanceEvents>(@windfall);
        event::emit_event(&mut events.vote_events, VoteEvent {
            proposal_id,
            voter: voter_address,
            vote,
            timestamp: current_time,
        });

        security::end_reentrancy_protection();
    }

    public entry fun emergency_veto(
        member: &signer,
        proposal_id: u64
    ) acquires GovernanceData, GovernanceEvents {
        // Security checks
        security::assert_not_paused(MODULE_ID);
        security::start_reentrancy_protection();

        let member_address = signer::address_of(member);
        assert!(registry::is_active(member_address), error::permission_denied(ENOT_MEMBER));

        let governance_data = borrow_global_mut<GovernanceData>(@windfall);
        assert!(table::contains(&governance_data.proposals, proposal_id),
            error::not_found(EPROPOSAL_NOT_FOUND));

        let proposal = table::borrow_mut(&mut governance_data.proposals, proposal_id);
        assert!(!proposal.executed, error::invalid_state(EPROPOSAL_ALREADY_EXECUTED));

        let votes = table::borrow(&governance_data.votes, proposal_id);
        let total_votes = proposal.votes_yes + proposal.votes_no;
        
        // Check if veto threshold is met
        if (proposal.votes_no * 100 >= total_votes * EMERGENCY_VETO_THRESHOLD) {
            proposal.executed = true; // Mark as executed to prevent further voting
            
            // Emit veto event
            let events = borrow_global_mut<GovernanceEvents>(@windfall);
            event::emit_event(&mut events.emergency_veto_events, EmergencyVetoEvent {
                proposal_id,
                initiator: member_address,
                timestamp: timestamp::now_microseconds(),
            });
        } else {
            abort error::invalid_state(EINSUFFICIENT_VOTES)
        };

        security::end_reentrancy_protection();
    }

    public entry fun create_trade_proposal(
        proposer: &signer,
        asset_symbol: String,
        size: u64,
        price: u64,
        is_entry: bool,
        description: String
    ) acquires GovernanceData, GovernanceEvents {
        let proposer_address = signer::address_of(proposer);
        assert!(registry::is_active(proposer_address), error::permission_denied(ENOT_MEMBER));

        let governance_data = borrow_global_mut<GovernanceData>(@windfall);
        let proposal_id = governance_data.next_proposal_id;
        let current_time = timestamp::now_microseconds();

        let trade_info = TradeInfo {
            asset_symbol,
            size,
            price,
            is_entry,
        };

        let proposal = Proposal {
            id: proposal_id,
            proposer: proposer_address,
            proposal_type: ProposalType { code: PROPOSAL_TYPE_TRADE },
            description,
            start_time: current_time,
            end_time: current_time + PROPOSAL_DURATION,
            executed: false,
            votes_yes: 0,
            votes_no: 0,
            total_eligible_votes: registry::get_active_users(),
            payload: std::bcs::to_bytes(&trade_info),
        };

        table::add(&mut governance_data.proposals, proposal_id, proposal);
        table::add(&mut governance_data.votes, proposal_id, table::new());
        governance_data.next_proposal_id = proposal_id + 1;

        // Emit event
        let events = borrow_global_mut<GovernanceEvents>(@windfall);
        event::emit_event(&mut events.proposal_created_events, ProposalCreatedEvent {
            proposal_id,
            proposer: proposer_address,
            proposal_type: PROPOSAL_TYPE_TRADE,
            timestamp: current_time,
        });
    }

    public entry fun execute_proposal(
        executor: &signer,
        proposal_id: u64
    ) acquires GovernanceData, GovernanceEvents {
        // Security checks
        security::assert_not_paused(MODULE_ID);
        security::start_reentrancy_protection();

        let executor_address = signer::address_of(executor);
        assert!(registry::is_active(executor_address), error::permission_denied(ENOT_MEMBER));

        let governance_data = borrow_global_mut<GovernanceData>(@windfall);
        assert!(table::contains(&governance_data.proposals, proposal_id),
            error::not_found(EPROPOSAL_NOT_FOUND));

        let proposal = table::borrow_mut(&mut governance_data.proposals, proposal_id);
        let current_time = timestamp::now_microseconds();
        
        assert!(current_time > proposal.end_time, error::invalid_state(EPROPOSAL_EXPIRED));
        assert!(!proposal.executed, error::invalid_state(EPROPOSAL_ALREADY_EXECUTED));

        let total_votes = proposal.votes_yes + proposal.votes_no;
        assert!(total_votes > 0, error::invalid_state(EINSUFFICIENT_VOTES));

        // Check if proposal passed
        let success = proposal.votes_yes * 100 > total_votes * governance_data.config.quorum_threshold;
        
        if (success) {
            // Mark as executed
            proposal.executed = true;

            // Handle different proposal types
            if (proposal.proposal_type.code == PROPOSAL_TYPE_ACTUATOR) {
                // Update actuator in position contract
                let new_actuator: address = std::bcs::from_bytes(&proposal.payload);
                windfall::position::set_actuator(executor, new_actuator);
            };
            // Note: Trade execution will be handled by the actuator off-chain
        };

        // Emit execution event
        let events = borrow_global_mut<GovernanceEvents>(@windfall);
        event::emit_event(&mut events.proposal_executed_events, ProposalExecutedEvent {
            proposal_id,
            executed_by: executor_address,
            success,
            timestamp: current_time,
        });

        security::end_reentrancy_protection();
    }

    #[view]
    public fun get_proposal_info(proposal_id: u64): (u8, u64, u64, u64, bool) acquires GovernanceData {
        let governance_data = borrow_global<GovernanceData>(@windfall);
        assert!(table::contains(&governance_data.proposals, proposal_id),
            error::not_found(EPROPOSAL_NOT_FOUND));
        
        let proposal = table::borrow(&governance_data.proposals, proposal_id);
        (
            proposal.proposal_type.code,
            proposal.votes_yes,
            proposal.votes_no,
            proposal.end_time,
            proposal.executed
        )
    }

    #[view]
    public fun has_voted(proposal_id: u64, voter: address): bool acquires GovernanceData {
        let governance_data = borrow_global<GovernanceData>(@windfall);
        if (!table::contains(&governance_data.votes, proposal_id)) {
            return false
        };
        
        let votes = table::borrow(&governance_data.votes, proposal_id);
        table::contains(votes, voter)
    }
} 
