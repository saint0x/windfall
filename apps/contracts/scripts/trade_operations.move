script {
    use windfall::position;

    const DESCRIPTION: vector<u8> = b"Position operations";

    fun main(
        trader: &signer,
        asset_id: u64,
        size: u64,
        price: u64,
        is_entry: bool,
        position_id: u64
    ) {
        if (is_entry) {
            position::open_position(
                trader,
                asset_id,
                size,
                price,
                true // is_long (we can add short later)
            );
        } else {
            position::close_position(
                trader,
                position_id,
                price
            );
        }
    }
}