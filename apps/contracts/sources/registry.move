module windfall::registry {
    use std::error;
    use std::signer;
    use aptos_framework::account;
    use aptos_framework::event::{Self, EventHandle};
    use aptos_framework::timestamp;
    use aptos_std::table::{Self, Table};
    use windfall::security;

    /// Error codes
    const ENOT_INITIALIZED: u64 = 1;
    const EALREADY_INITIALIZED: u64 = 2;
    const ENOT_AUTHORIZED: u64 = 3;
    const EUSER_NOT_FOUND: u64 = 4;
    const EINVALID_VERIFICATION_LEVEL: u64 = 5;
    const EUSER_ALREADY_REGISTERED: u64 = 6;
    const ESECURITY_NOT_INITIALIZED: u64 = 7;

    const MODULE_ID: u8 = 4; // Unique identifier for registry module

    struct UserProfile has store {
        registration_time: u64,
        last_updated: u64,
        is_active: bool,
    }

    struct RegistryEvents has key {
        registration_events: EventHandle<RegistrationEvent>,
        deactivation_events: EventHandle<DeactivationEvent>,
    }

    struct RegistryData has key {
        admin: address,
        users: Table<address, UserProfile>,
        total_users: u64,
        active_users: u64,
    }

    struct RegistrationEvent has drop, store {
        user_address: address,
        registration_time: u64,
    }

    struct DeactivationEvent has drop, store {
        user_address: address,
        deactivation_time: u64,
    }

    public fun initialize(admin: &signer) {
        let admin_address = signer::address_of(admin);
        
        assert!(!exists<RegistryData>(admin_address), error::already_exists(EALREADY_INITIALIZED));
        
        move_to(admin, RegistryData {
            admin: admin_address,
            users: table::new(),
            total_users: 0,
            active_users: 0,
        });

        move_to(admin, RegistryEvents {
            registration_events: account::new_event_handle<RegistrationEvent>(admin),
            deactivation_events: account::new_event_handle<DeactivationEvent>(admin),
        });
    }

    public entry fun register_user(
        _admin: &signer,  // Prefixed with underscore as it will be used for authorization in future
        user_address: address,
        _verification_level: u8  // Prefixed with underscore as it will be used in future
    ) acquires RegistryData, RegistryEvents {
        // Security checks
        security::assert_not_paused(MODULE_ID);
        security::start_reentrancy_protection();
        
        let registry_data = borrow_global_mut<RegistryData>(@windfall);
        
        assert!(!table::contains(&registry_data.users, user_address), 
            error::already_exists(EUSER_ALREADY_REGISTERED));

        let current_time = timestamp::now_microseconds();
        
        table::add(&mut registry_data.users, user_address, UserProfile {
            registration_time: current_time,
            last_updated: current_time,
            is_active: true,
        });

        registry_data.total_users = registry_data.total_users + 1;
        registry_data.active_users = registry_data.active_users + 1;

        // Emit registration event
        let events = borrow_global_mut<RegistryEvents>(@windfall);
        event::emit_event(&mut events.registration_events, RegistrationEvent {
            user_address,
            registration_time: current_time,
        });
    }

    public entry fun update_verification_level(
        admin: &signer,
        user_address: address,
        _new_level: u8  // Prefixed with underscore as it will be used in future
    ) acquires RegistryData, RegistryEvents {
        // Security checks
        security::assert_not_paused(MODULE_ID);
        
        let registry_data = borrow_global_mut<RegistryData>(@windfall);
        
        assert!(signer::address_of(admin) == registry_data.admin,
            error::permission_denied(ENOT_AUTHORIZED));
        assert!(table::contains(&registry_data.users, user_address),
            error::not_found(EUSER_NOT_FOUND));

        let user_profile = table::borrow_mut(&mut registry_data.users, user_address);
        
        if (user_profile.is_active) {
            user_profile.is_active = false;
            user_profile.last_updated = timestamp::now_microseconds();
            registry_data.active_users = registry_data.active_users - 1;

            // Emit deactivation event
            let events = borrow_global_mut<RegistryEvents>(@windfall);
            event::emit_event(&mut events.deactivation_events, DeactivationEvent {
                user_address,
                deactivation_time: timestamp::now_microseconds(),
            });
        }
    }

    public entry fun deactivate_user(
        admin: &signer,
        user_address: address
    ) acquires RegistryData, RegistryEvents {
        // Security checks
        security::assert_not_paused(MODULE_ID);
        
        let registry_data = borrow_global_mut<RegistryData>(@windfall);
        
        assert!(signer::address_of(admin) == registry_data.admin,
            error::permission_denied(ENOT_AUTHORIZED));
        assert!(table::contains(&registry_data.users, user_address),
            error::not_found(EUSER_NOT_FOUND));

        let user_profile = table::borrow_mut(&mut registry_data.users, user_address);
        
        if (user_profile.is_active) {
            user_profile.is_active = false;
            user_profile.last_updated = timestamp::now_microseconds();
            registry_data.active_users = registry_data.active_users - 1;

            // Emit deactivation event
            let events = borrow_global_mut<RegistryEvents>(@windfall);
            event::emit_event(&mut events.deactivation_events, DeactivationEvent {
                user_address,
                deactivation_time: timestamp::now_microseconds(),
            });
        }
    }

    public entry fun reactivate_user(
        admin: &signer,
        user_address: address
    ) acquires RegistryData, RegistryEvents {
        // Security checks
        security::assert_not_paused(MODULE_ID);
        
        let registry_data = borrow_global_mut<RegistryData>(@windfall);
        
        assert!(signer::address_of(admin) == registry_data.admin,
            error::permission_denied(ENOT_AUTHORIZED));
        assert!(table::contains(&registry_data.users, user_address),
            error::not_found(EUSER_NOT_FOUND));

        let user_profile = table::borrow_mut(&mut registry_data.users, user_address);
        
        if (!user_profile.is_active) {
            user_profile.is_active = true;
            user_profile.last_updated = timestamp::now_microseconds();
            registry_data.active_users = registry_data.active_users + 1;

            // Emit reactivation event
            let events = borrow_global_mut<RegistryEvents>(@windfall);
            event::emit_event(&mut events.registration_events, RegistrationEvent {
                user_address,
                registration_time: timestamp::now_microseconds(),
            });
        }
    }

    #[view]
    public fun is_registered(user_address: address): bool acquires RegistryData {
        let registry_data = borrow_global<RegistryData>(@windfall);
        table::contains(&registry_data.users, user_address)
    }

    #[view]
    public fun is_active(user_address: address): bool acquires RegistryData {
        let registry_data = borrow_global<RegistryData>(@windfall);
        assert!(table::contains(&registry_data.users, user_address),
            error::not_found(EUSER_NOT_FOUND));
        
        let user_profile = table::borrow(&registry_data.users, user_address);
        user_profile.is_active
    }

    #[view]
    public fun get_total_users(): u64 acquires RegistryData {
        borrow_global<RegistryData>(@windfall).total_users
    }

    #[view]
    public fun get_active_users(): u64 acquires RegistryData {
        borrow_global<RegistryData>(@windfall).active_users
    }

    #[view]
    public fun get_verification_level(user_address: address): u8 acquires RegistryData {
        let registry_data = borrow_global<RegistryData>(@windfall);
        if (!table::contains(&registry_data.users, user_address)) {
            return 0
        };
        
        let user_profile = table::borrow(&registry_data.users, user_address);
        if (user_profile.is_active) {
            1 // For now, all active users have level 1
        } else {
            0
        }
    }
} 
