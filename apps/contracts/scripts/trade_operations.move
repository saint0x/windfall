script {
    use std::string::String;
    use windfall::position;
    use windfall::governance;

    /// Called by backend when a trade proposal is approved and ready to execute
    public entry fun execute_trade(
        actuator: &signer,
        proposal_id: u64,
        asset_symbol: String,
        size: u64,
        price: u64,
        is_entry: bool
    ) {
        // If it's an entry, open new position
        if (is_entry) {
            position::open_position(
                actuator,
                asset_symbol,
                size,
                price,
                true // is_long (we can add short later)
            );
        } else {
            // Close position logic will be added
            // This will include PnL calculation and distribution
        };
    }

    /// Called by backend to update position shares after execution
    public entry fun update_shares(
        actuator: &signer,
        position_id: u64,
        user: address,
        shares: u64
    ) {
        position::allocate_shares(actuator, position_id, user, shares);
    }

    /// Called by backend to submit trade proposals
    public entry fun propose_trade(
        proposer: &signer,
        asset_symbol: String,
        size: u64,
        price: u64,
        is_entry: bool,
        description: String
    ) {
        governance::create_trade_proposal(
            proposer,
            asset_symbol,
            size,
            price,
            is_entry,
            description
        );
    }
} 