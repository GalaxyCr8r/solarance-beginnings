// use spacetimedb::{client_visibility_filter, Filter};

// // You can see your ship object.
// #[client_visibility_filter]
// const PLAYER_CON_FILTER: Filter = Filter::Sql(
//     "SELECT c.* 
//      FROM player_ship_controller c
//      WHERE c.player_id = :sender"
// );