use spacetimedb::{client_visibility_filter, Filter};

// You can only see ship objects in your sector.
// #[client_visibility_filter]
// const SO_SECTOR_FILTER: Filter = Filter::Sql(
//     "SELECT o.* 
//     FROM ship_object o
//     JOIN ship_object s ON s.sector_id = o.sector_id
//     WHERE s.player_id = :sender"
// );

// // You can only see ship instances in your sector.
// #[client_visibility_filter]
// const SI_SECTOR_FILTER: Filter = Filter::Sql(
//     "SELECT ship_instance.* 
//     FROM ship_instance
//     JOIN ship_object ON s.sector_id = i.current_sector_id
//     WHERE s.player_id = :sender"
// ); // "SELECT account.* FROM account JOIN admin WHERE admin.identity = :sender"