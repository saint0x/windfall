script {
    use std::string;
    use windfall::governance;

    const OPERATION_VETO: u8 = 0;
    const OPERATION_TRADE: u8 = 1;

    fun main(
        caller: &signer,
        operation: u8,
        proposal_id: u64,
        asset_symbol: vector<u8>,
        size: u64,
        price: u64,
        is_entry: bool,
        description: vector<u8>
    ) {
        if (operation == OPERATION_VETO) {
            governance::emergency_veto(caller, proposal_id);
        } else if (operation == OPERATION_TRADE) {
            governance::create_trade_proposal(
                caller,
                string::utf8(asset_symbol),
                size,
                price,
                is_entry,
                string::utf8(description)
            );
        };
    }
} 