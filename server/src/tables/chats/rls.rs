//use spacetimedb::{client_visibility_filter, Filter};

// // You can only see sector chat messages in your current sector.
// #[client_visibility_filter]
// const CHAT_SECTOR_FILTER: Filter = Filter::Sql(
//      "SELECT c.* 
//       FROM sector_chat_message c
//       JOIN ship o
//       WHERE o.sector_id = c.sector_id"
//  );

//  // TODO: Add faction chat filter