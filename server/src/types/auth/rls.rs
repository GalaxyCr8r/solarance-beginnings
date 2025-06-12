use spacetimedb::{client_visibility_filter, Filter};

// You can see your ship object.
#[client_visibility_filter]
const PRIVILAGED_CLIENT_FILTER: Filter = Filter::Sql(
    "SELECT c.* 
     FROM privilaged_client c
     WHERE c.privilage >= (SELECT a.privilage FROM privilaged_client a WHERE a.identity = :sender)"
);