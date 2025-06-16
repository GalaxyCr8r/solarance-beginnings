use spacetimedb::{client_visibility_filter, Filter};

// You can only see sector chat messages in your current sector.
#[client_visibility_filter]
const ASTEROID_SECTOR_FILTER: Filter = Filter::Sql(
     "SELECT a.* 
      FROM asteroid a
      JOIN ship o ON o.sector_id = a.current_sector_id"
 );