module windfall::security {
    use std::error;
    use std::signer;
    use aptos_framework::account;
    use aptos_framework::event::{Self, EventHandle};
    use aptos_framework::timestamp;

    /// Error codes
    const ENOT_ADMIN: u64 = 1;
    const ECONTRACT_PAUSED: u64 = 2;
    const EREENTRANCY: u64 = 3;
    const ENOT_INITIALIZED: u64 = 4;
    const EALREADY_INITIALIZED: u64 = 5;

    /// Stores the pause state and admin controls
    struct SecurityConfig has key {
        admin: address,
        paused: bool,
        pause_guards: vector<u8>, // Module identifiers that are paused
    }

    /// Reentrancy guard for specific operations
    struct ReentrancyGuard has key {
        entered: bool,
    }

    /// Events for security-related actions
    struct SecurityEvents has key {
        pause_events: EventHandle<PauseEvent>,
        admin_events: EventHandle<AdminEvent>,
    }

    struct PauseEvent has drop, store {
        paused: bool,
        module_id: u8,
        admin: address,
        timestamp: u64,
    }

    struct AdminEvent has drop, store {
        previous_admin: address,
        new_admin: address,
        timestamp: u64,
    }

    /// Initialize security configuration
    public fun initialize(admin: &signer) {
        let admin_address = signer::address_of(admin);
        
        assert!(!exists<SecurityConfig>(admin_address), error::already_exists(EALREADY_INITIALIZED));
        
        move_to(admin, SecurityConfig {
            admin: admin_address,
            paused: false,
            pause_guards: vector::empty(),
        });

        move_to(admin, ReentrancyGuard {
            entered: false,
        });

        move_to(admin, SecurityEvents {
            pause_events: account::new_event_handle<PauseEvent>(admin),
            admin_events: account::new_event_handle<AdminEvent>(admin),
        });
    }

    /// Check if a specific module is paused
    public fun is_paused(module_id: u8): bool acquires SecurityConfig {
        let config = borrow_global<SecurityConfig>(@windfall);
        config.paused || vector::contains(&config.pause_guards, &module_id)
    }

    /// Pause a specific module
    public entry fun pause_module(
        admin: &signer,
        module_id: u8
    ) acquires SecurityConfig, SecurityEvents {
        let admin_address = signer::address_of(admin);
        let config = borrow_global_mut<SecurityConfig>(@windfall);
        
        assert!(admin_address == config.admin, error::permission_denied(ENOT_ADMIN));
        
        if (!vector::contains(&config.pause_guards, &module_id)) {
            vector::push_back(&mut config.pause_guards, module_id);
            
            // Emit pause event
            let events = borrow_global_mut<SecurityEvents>(@windfall);
            event::emit_event(&mut events.pause_events, PauseEvent {
                paused: true,
                module_id,
                admin: admin_address,
                timestamp: timestamp::now_microseconds(),
            });
        };
    }

    /// Unpause a specific module
    public entry fun unpause_module(
        admin: &signer,
        module_id: u8
    ) acquires SecurityConfig, SecurityEvents {
        let admin_address = signer::address_of(admin);
        let config = borrow_global_mut<SecurityConfig>(@windfall);
        
        assert!(admin_address == config.admin, error::permission_denied(ENOT_ADMIN));
        
        let (found, index) = vector::index_of(&config.pause_guards, &module_id);
        if (found) {
            vector::remove(&mut config.pause_guards, index);
            
            // Emit unpause event
            let events = borrow_global_mut<SecurityEvents>(@windfall);
            event::emit_event(&mut events.pause_events, PauseEvent {
                paused: false,
                module_id,
                admin: admin_address,
                timestamp: timestamp::now_microseconds(),
            });
        };
    }

    /// Pause all modules
    public entry fun pause_all(admin: &signer) acquires SecurityConfig, SecurityEvents {
        let admin_address = signer::address_of(admin);
        let config = borrow_global_mut<SecurityConfig>(@windfall);
        
        assert!(admin_address == config.admin, error::permission_denied(ENOT_ADMIN));
        
        config.paused = true;
        
        // Emit pause event
        let events = borrow_global_mut<SecurityEvents>(@windfall);
        event::emit_event(&mut events.pause_events, PauseEvent {
            paused: true,
            module_id: 0, // 0 indicates all modules
            admin: admin_address,
            timestamp: timestamp::now_microseconds(),
        });
    }

    /// Unpause all modules
    public entry fun unpause_all(admin: &signer) acquires SecurityConfig, SecurityEvents {
        let admin_address = signer::address_of(admin);
        let config = borrow_global_mut<SecurityConfig>(@windfall);
        
        assert!(admin_address == config.admin, error::permission_denied(ENOT_ADMIN));
        
        config.paused = false;
        
        // Emit unpause event
        let events = borrow_global_mut<SecurityEvents>(@windfall);
        event::emit_event(&mut events.pause_events, PauseEvent {
            paused: false,
            module_id: 0, // 0 indicates all modules
            admin: admin_address,
            timestamp: timestamp::now_microseconds(),
        });
    }

    /// Transfer admin rights to a new address
    public entry fun transfer_admin(
        current_admin: &signer,
        new_admin: address
    ) acquires SecurityConfig, SecurityEvents {
        let admin_address = signer::address_of(current_admin);
        let config = borrow_global_mut<SecurityConfig>(@windfall);
        
        assert!(admin_address == config.admin, error::permission_denied(ENOT_ADMIN));
        
        let old_admin = config.admin;
        config.admin = new_admin;
        
        // Emit admin change event
        let events = borrow_global_mut<SecurityEvents>(@windfall);
        event::emit_event(&mut events.admin_events, AdminEvent {
            previous_admin: old_admin,
            new_admin,
            timestamp: timestamp::now_microseconds(),
        });
    }

    /// Start reentrancy protection
    public fun start_reentrancy_protection() acquires ReentrancyGuard {
        let guard = borrow_global_mut<ReentrancyGuard>(@windfall);
        assert!(!guard.entered, error::invalid_state(EREENTRANCY));
        guard.entered = true;
    }

    /// End reentrancy protection
    public fun end_reentrancy_protection() acquires ReentrancyGuard {
        let guard = borrow_global_mut<ReentrancyGuard>(@windfall);
        guard.entered = false;
    }

    /// Check if a function is being reentered
    public fun is_reentrant(): bool acquires ReentrancyGuard {
        borrow_global<ReentrancyGuard>(@windfall).entered
    }

    /// Modifier-style function to check if a module is paused
    public fun assert_not_paused(module_id: u8) acquires SecurityConfig {
        assert!(!is_paused(module_id), error::invalid_state(ECONTRACT_PAUSED));
    }

    /// Modifier-style function to check if caller is admin
    public fun assert_admin(caller: address) acquires SecurityConfig {
        let config = borrow_global<SecurityConfig>(@windfall);
        assert!(caller == config.admin, error::permission_denied(ENOT_ADMIN));
    }
} 