use spacetimedb::{client_visibility_filter, Filter};

// You can see your ship object.
// #[client_visibility_filter]
// const SO_PLAYER_FILTER: Filter = Filter::Sql(
//     "SELECT o.* 
//      FROM ship_object o
//      WHERE o.player_id = :sender" // This doesn't matter unless the sector filter will work... so for now the client will have to limit it.
// );

// You can see your ship object.
// #[client_visibility_filter]
// const SO_SECTOR_FILTER: Filter = Filter::Sql(
//     "SELECT o.* 
//      FROM ship_object o
//      JOIN ship_object s ON s.sector_id = o.sector_id" // WILL NOT WORK until they fix RLS.
// );

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