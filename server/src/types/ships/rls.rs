use spacetimedb::{client_visibility_filter, Filter};

///You can only see ship objects in your sector.
#[client_visibility_filter]
const SO_SECTOR_FILTER: Filter = Filter::Sql(
    "SELECT o.* 
     FROM ship_object o
     WHERE o.player_id = :sender"
);

// You can only see ship instances of ship objects you can see.
#[client_visibility_filter]
const SI_SECTOR_FILTER: Filter = Filter::Sql(
     "SELECT i.* 
      FROM ship_instance i
      JOIN ship_object o
      WHERE o.ship_id = i.id"
 );

// You can only see ship cargo items of ships you can see.
#[client_visibility_filter]
const SI_CARGO_FILTER: Filter = Filter::Sql(
     "SELECT ship_cargo_item.* 
      FROM ship_cargo_item
      JOIN ship_object o ON ship_cargo_item.ship_id = o.ship_id"
 );