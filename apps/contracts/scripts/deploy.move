script {
    use std::signer;
    use windfall::registry;
    use windfall::asset;
    use windfall::position;
    use windfall::governance;

    fun initialize_contracts(admin: &signer) {
        // Initialize registry first as other contracts depend on it
        registry::initialize(admin);

        // Initialize asset management
        asset::initialize(admin);

        // Initialize position tracking
        position::initialize(admin);

        // Initialize governance last as it depends on other contracts
        governance::initialize(admin);
    }

    public entry fun deploy(admin: &signer) {
        initialize_contracts(admin);
    }
} 