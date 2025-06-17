use spacetimedb::{client_visibility_filter, Filter};

// You can only see CargoCrates in your sector... somehow
#[client_visibility_filter]
const CRATE_SECTOR_FILTER: Filter = Filter::Sql(
     "SELECT c.* 
      FROM cargo_crate c
      JOIN ship o ON o.sector_id = c.current_sector_id"
 );