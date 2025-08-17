flowchart TD
    asteroid[Asteroid<br>---<br>id - PK FK<br>current_sector_id - FK<br>size_radius<br>resource_item_id - FK<br>current_resources<br>initial_resources<br>gfx_key]
    faction_chat_message[FactionChatMessage<br>---<br>id - PK<br>player_id - FK<br>faction_id - FK<br>message<br>created_at]
    faction_definition[Faction<br>---<br>id - PK<br>name<br>description<br>capital_station_id - FK?<br>]
    faction_standing[FactionStanding<br>---<br>id - PK<br>faction_one_id - FK<br>faction_two_id - FK<br>reputation_score]
    global_chat_message[GlobalChatMessage<br>---<br>id - PK<br>player_id - FK<br>message<br>created_at]
    global_config[GlobalConfig<br>---<br>id - PK<br>active_players<br>old_gods_defeated<br>version<br>created_at<br>modified_at]
    item_definition[ItemDefinition<br>---<br>id - PK<br>name<br>description<br>category<br>base_value<br>margin_percentage<br>volume_per_unit<br>units_per_stack<br>metadata<br>gfx_key]
    player[Player<br>---<br>id - PK<br>username<br>credits<br>logged_in<br>faction_id - FK?<br>created_at<br>modified_at]
    sector[Sector<br>---<br>id - PK<br>system_id - FK<br>name<br>description<br>controlling_faction_id - FK<br>security_level<br>sunlight<br>anomalous<br>nebula<br>rare_ore<br>x<br>y<br>background_gfx_key]
    sector_chat_message[SectorChatMessage<br>---<br>id - PK<br>player_id - FK<br>sector_id - FK<br>message<br>created_at]
    star_system[StarSystem<br>---<br>id - PK<br>name<br>map_coordinates<br>spectral<br>luminosity<br>controlling_faction_id - FK]
    stellar_object[StellarObject<br>---<br>id - PK<br>kind<br>sector_id - FK]
    faction_definition --> faction_chat_message
    faction_definition --> faction_standing
    faction_definition --> sector
    faction_definition --> star_system
    item_definition --> asteroid
    player --> faction_chat_message
    player --> global_chat_message
    player --> sector_chat_message
    sector --> asteroid
    sector --> sector_chat_message
    sector --> stellar_object
    star_system --> sector
    stellar_object --> asteroid