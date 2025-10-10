# Implementation Plan

## Combat System Core Infrastructure

- [x] 1. Create combat module structure

  - Create `server/src/types/combat.rs` module file
  - Create `server/src/types/combat/` directory with standard submodules (types.rs, reducers.rs, utility.rs, timers.rs, impls.rs)
  - Add combat module to `server/src/types/mod.rs`
  - _Requirements: 1.1, 2.1, 3.1, 4.1_

- [x] 2. Implement VisualEffect database tables

  - Create `VisualEffect` table with id, source, target, effect_type, created_at fields
  - Create `VisualEffectType` enum with WeaponFire, MissileFire, Explosion variants
  - Create `VisualEffectTimer` scheduled table for cleanup
  - _Requirements: 2.1, 2.2, 2.3_

- [x] 3. Implement weapon and missile type enums

  - Create `WeaponType` enum with Hitscan, Projectile, AreaOfEffect variants
  - Create `MissileType` enum with Dumbfire, Heatseeking variants
  - Add these to combat types module
  - _Requirements: 5.4, 4.1_

## Combat Processing Functions

- [x] 4. Implement core combat utility functions

  - Create `process_weapon_fire` function that handles hitscan damage calculation
  - Create `process_missile_fire` function as placeholder for future missile system
  - Implement damage calculation logic using ItemMetadata from weapon definitions
  - _Requirements: 1.1, 1.2, 1.3, 5.1, 5.2_

- [x] 5. Implement damage calculation system

  - Create damage calculation function that extracts BaseDamage, ShieldDamageMod, KineticDamageMod from weapon ItemMetadata
  - Apply damage to target shields first, then hull health
  - Handle target destruction when hull health reaches zero
  - _Requirements: 1.2, 1.3, 1.6_

- [x] 6. Implement energy consumption system

  - Extract EnergyConsumption from weapon ItemMetadata
  - Validate sufficient energy before allowing weapon fire
  - Deduct energy from ship's current energy reserves
  - _Requirements: 1.4, 1.5, 6.1, 6.2, 6.3, 6.5_

## Visual Effects System

- [x] 7. Implement visual effect creation and cleanup

  - Create visual effects when weapons are fired with source and target positions
  - Schedule automatic cleanup after 10 milliseconds using VisualEffectTimer
  - Implement `cleanup_visual_effect` scheduled reducer
  - _Requirements: 2.1, 2.2, 2.3_

## Combat Reducers Integration

- [x] 8. Create combat processing reducer

  - Implement reducer that processes PlayerShipController fire_weapons and fire_missiles flags
  - Reset fire_weapons and fire_missiles to false after processing
  - Validate target is valid Ship or Station class before firing
  - _Requirements: 1.1, 5.1_

- [x] 9. Integrate combat with player ship controller updates
  - Modify existing `update_player_controller` reducer to call combat processing
  - Ensure combat actions are processed when fire_weapons or fire_missiles are set
  - Add server-side validation for combat actions
  - _Requirements: 1.1, 5.1_

## Ship Status Extensions

- [ ] 10. Add weapon cooldown fields to ShipStatus
  - Add `weapon_cooldown_ms: u32` field to ShipStatus table
  - Add `missile_cooldown_ms: u32` field to ShipStatus table
  - Implement cooldown checking in combat processing
  - _Requirements: 1.1, 5.1_

## NPC Combat Preparation

- [ ] 11. Create NpcShipController placeholder structure
  - Create `NpcShipController` table with same combat fields as PlayerShipController
  - Create `NpcBehavior` enum with Idle, Patrol, Attack, Flee variants
  - Ensure combat functions accept both player and NPC controllers
  - _Requirements: 4.1, 4.2, 4.3, 4.4_

## Client-Side Combat Mode

- [ ] 12. Implement client-side combat mode state

  - Create combat mode toggle functionality for Q key
  - Implement spacebar for fire_weapons in combat mode only
  - Implement Left Control for fire_missiles in combat mode only
  - Prevent weapon firing in utility mode
  - _Requirements: 7.1, 7.2, 7.3, 7.4_

- [ ] 13. Add client-side visual effect handling
  - Create firing effect rendering system for visual feedback
  - Handle VisualEffect database entries to trigger client-side effects
  - Implement configurable effect duration on client
  - _Requirements: 2.4_

## Error Handling and Validation

- [ ] 14. Implement combat error handling
  - Create `CombatError` enum with InsufficientEnergy, InvalidTarget, WeaponNotEquipped, OutOfRange variants
  - Add proper error messages for combat failures
  - Implement server-side validation for all combat actions
  - _Requirements: 1.5, 6.3_

## Testing and Integration

- [ ] 15. Create weapon item definitions for testing

  - Add sample weapon ItemDefinitions with combat-related ItemMetadata
  - Include BaseDamage, ShieldDamageMod, KineticDamageMod, EnergyConsumption metadata
  - Ensure weapons can be equipped in ship weapon slots
  - _Requirements: 1.2, 6.1_

- [ ] 16. Test combat system integration
  - Verify weapon firing consumes energy and applies damage
  - Test visual effects creation and cleanup
  - Validate combat mode switching prevents firing in utility mode
  - Test target validation and error handling
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 1.6, 2.1, 2.2, 2.3, 6.1, 6.2, 6.3, 7.1, 7.2, 7.3_
