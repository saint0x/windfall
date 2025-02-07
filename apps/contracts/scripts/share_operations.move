script {
    use windfall::position;

    const DESCRIPTION: vector<u8> = b"Share allocation";

    fun main(
        actuator: &signer,
        position_id: u64,
        user_address: address,
        shares: u64
    ) {
        position::allocate_shares(
            actuator,
            position_id,
            user_address,
            shares
        );
    }
} 