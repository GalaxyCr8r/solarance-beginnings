flowchart TD
    asteroid[Asteroid<br>---<br>id - PK FK<br>current_sector_id - FK<br>size_radius<br>resource_item_id - FK<br>current_resources<br>initial_resources<br>gfx_key]
    asteroid_sector[AsteroidSector<br>---<br>id - PK FK<br>sparseness<br>rarity<br>cluster_extent<br>cluster_inner<br>]
    cargo_crate[CargoCrate<br>---<br>id - PK<br>current_sector_id - FK<br>sobj_id - FK<br>item_id - FK<br>quantity<br>despawn_ts<br>gfx_key]
    docked_ship[DockedShip<br>---<br>id - PK FK<br>shiptype_id - FK<br>station_id - FK<br>sector_id - FK<br>player_id - FK<br>faction_id - FK]
    faction_chat_message[FactionChatMessage<br>---<br>id - PK<br>player_id - FK<br>faction_id - FK<br>message<br>created_at]
    faction_definition[Faction<br>---<br>id - PK<br>name<br>description<br>capital_station_id - FK?<br>]
    faction_standing[FactionStanding<br>---<br>id - PK<br>faction_one_id - FK<br>faction_two_id - FK<br>reputation_score]
    global_chat_message[GlobalChatMessage<br>---<br>id - PK<br>player_id - FK<br>message<br>created_at]
    global_config[GlobalConfig<br>---<br>id - PK<br>active_players<br>old_gods_defeated<br>version<br>created_at<br>modified_at]
    item_definition[ItemDefinition<br>---<br>id - PK<br>name<br>description<br>category<br>base_value<br>margin_percentage<br>volume_per_unit<br>units_per_stack<br>metadata<br>gfx_key]
    jump_gate[JumpGate<br>---<br>id - PK FK<br>current_sector_id - FK<br>target_sector_id - FK<br>target_gate_arrival_pos<br>gfx_key<br>is_active]
    player[Player<br>---<br>id - PK<br>username<br>credits<br>logged_in<br>faction_id - FK?<br>created_at<br>modified_at]
    player_faction_standing[PlayerFactionStanding<br>---<br>id - PK<br>player_identity - FK<br>faction_id - FK<br>reputation_score<br>]
    player_ship_controller[PlayerShipController<br>---<br>id - PK FK<br>stellar_object_id - FK<br>up<br>down<br>left<br>right<br>current_action<br>activate_jump_drive<br>tractor_beam_on<br>mining_laser_on<br>cargo_bay_open<br>dock<br>undock<br>shield_boost<br>fire_weapons<br>fire_missle<br>targetted_sobj_id - FK?]
    sector[Sector<br>---<br>id - PK<br>system_id - FK<br>name<br>description<br>controlling_faction_id - FK<br>security_level<br>sunlight<br>anomalous<br>nebula<br>rare_ore<br>x<br>y<br>background_gfx_key]
    sector_chat_message[SectorChatMessage<br>---<br>id - PK<br>player_id - FK<br>sector_id - FK<br>message<br>created_at]
    server_message[ServerMessage<br>---<br>id - PK<br>message<br>message_type<br>group_name<br>sender_context<br>created_at]
    server_message_recipient[ServerMessageRecipient<br>---<br>id - PK<br>server_message_id - FK<br>player_id - FK<br>read_at<br>delivered_at]
    ship[Ship<br>---<br>id - PK FK<br>shiptype_id - FK<br>sobj_id - FK<br>sector_id - FK<br>player_id - FK<br>faction_id - FK]
    ship_cargo_item[ShipCargoItem<br>---<br>id - PK<br>ship_id - FK<br>item_id - FK<br>quantity]
    ship_equipment_slot[ShipEquipmentSlot<br>---<br>id - PK<br>ship_id - FK<br>slot_type<br>slot_index<br>item_id - FK]
    ship_global[ShipGlobal<br>---<br>id - PK]
    ship_status[ShipStatus<br>---<br>id - PK<br>sector_id - FK?<br>player_id - FK?<br>health<br>shields<br>energy<br>used_cargo_capacity<br>max_cargo_capacity<br>ai_state]
    ship_type_definition[ShipTypeDefinition<br>---<br>id - PK<br>name<br>description<br>class<br>max_health<br>max_shields<br>max_energy<br>base_speed<br>base_acceleration<br>base_turn_rate<br>cargo_capacity<br>num_weapon_slots<br>num_large_weapon_slots<br>num_turret_slots<br>num_large_turret_slots<br>num_shield_slots<br>num_engine_slots<br>num_mining_laser_slots<br>num_special_slots<br>gfx_key]
    sobj_hi_res_transform[StellarObjectTransformHiRes<br>---<br>id - PK FK<br>x<br>y<br>rotation_radians]
    sobj_internal_transform[StellarObjectTransformInternal<br>---<br>id - PK FK<br>x<br>y<br>rotation_radians]
    sobj_low_res_transform[StellarObjectTransformLowRes<br>---<br>id - PK FK<br>x<br>y<br>rotation_radians]
    sobj_player_window[StellarObjectPlayerWindow<br>---<br>id - PK FK<br>sobj_id - FK<br>window<br>margin<br>tl_x<br>tl_y<br>br_x<br>br_y]
    sobj_turn_left_controller[StellarObjectControllerTurnLeft<br>---<br>id - PK FK]
    sobj_velocity[StellarObjectVelocity<br>---<br>id - PK FK<br>x<br>y<br>rotation_radians<br>auto_dampen]
    station[Station<br>---<br>id - PK<br>size<br>sector_id - FK<br>sobj_id - FK<br>owner_faction_id - FK<br>name<br>gfx_key]
    station_module[StationModule<br>---<br>id - PK<br>station_id - FK<br>blueprint - FK<br>station_slot_identifier<br>is_operational<br>built_at_timestamp<br>last_status_update_timestamp]
    station_module_blueprint[StationModuleBlueprint<br>---<br>id - PK<br>name<br>description<br>category<br>specific_type<br>build_cost_resources<br>build_time_seconds<br>power_consumption_mw_operational<br>power_consumption_mw_idle<br>cpu_load_flops<br>required_station_tech_level<br>max_internal_storage_slots<br>max_internal_storage_volume_per_slot_m3<br>provides_station_morale_boost<br>icon_asset_id<br>construction_hp<br>operational_hp]
    station_module_inventory_item[StationModuleInventoryItem<br>---<br>id - PK<br>module_id - FK<br>resource_item_id - FK<br>quantity<br>max_quantity<br>storage_purpose_tag<br>cached_price]    
    station_module_under_construction[StationModuleUnderConstruction<br>---<br>id - PK FK<br>is_operational<br>construction_progress_percentage]
    station_status[StationStatus<br>---<br>id - PK FK<br>health<br>shields<br>energy]
    station_under_construction[StationUnderConstruction<br>---<br>id - PK FK<br>is_operational<br>construction_progress_percentage]
    star_system[StarSystem<br>---<br>id - PK<br>name<br>map_coordinates<br>spectral<br>luminosity<br>controlling_faction_id - FK]
    star_system_object[StarSystemObject<br>---<br>id - PK<br>system_id - FK<br>kind<br>orbit_au<br>rotation_or_width_km<br>gfx_key]
    stellar_object[StellarObject<br>---<br>id - PK<br>kind<br>sector_id - FK]
    faction_definition --> docked_ship
    faction_definition --> faction_chat_message
    faction_definition --> faction_standing
    faction_definition --> player_faction_standing
    faction_definition --> sector
    faction_definition --> ship
    faction_definition --> star_system
    faction_definition --> station
    item_definition --> asteroid
    item_definition --> cargo_crate
    item_definition --> ship_cargo_item
    item_definition --> ship_equipment_slot
    item_definition --> station_module_inventory_item
    player --> docked_ship
    player --> faction_chat_message
    player --> global_chat_message
    player --> player_faction_standing
    player --> player_ship_controller
    player --> sector_chat_message
    player --> server_message_recipient
    player --> ship
    player --> sobj_player_window
    sector --> asteroid
    sector --> asteroid_sector
    sector --> cargo_crate
    sector --> docked_ship
    sector --> jump_gate
    sector --> sector_chat_message
    sector --> ship
    sector --> station
    sector --> stellar_object
    server_message --> server_message_recipient
    ship_global --> docked_ship
    ship_global --> ship
    ship_global --> ship_cargo_item
    ship_global --> ship_equipment_slot
    ship_type_definition --> docked_ship
    ship_type_definition --> ship
    station --> docked_ship
    station --> station_module
    station --> station_module_under_construction
    station --> station_status
    station --> station_under_construction
    station_module --> station_module_inventory_item
    station_module_blueprint --> station_module
    star_system --> sector
    star_system --> star_system_object
    stellar_object --> asteroid
    stellar_object --> cargo_crate
    stellar_object --> jump_gate
    stellar_object --> player_ship_controller
    stellar_object --> ship
    stellar_object --> sobj_hi_res_transform
    stellar_object --> sobj_internal_transform
    stellar_object --> sobj_low_res_transform
    stellar_object --> sobj_player_window
    stellar_object --> sobj_turn_left_controller
    stellar_object --> sobj_velocity
    stellar_object --> station