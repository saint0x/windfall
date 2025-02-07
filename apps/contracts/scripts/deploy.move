module windfall::deploy {
    use windfall::security;
    use windfall::registry;
    use windfall::position;
    use windfall::governance;

    public entry fun deploy(admin: &signer) {
        // Initialize modules in order
        security::initialize(admin);
        registry::initialize(admin);
        position::initialize(admin);
        governance::initialize(admin);
    }
} 