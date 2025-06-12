use spacetimedb::{client_visibility_filter, Filter};

// You can see your ship object.
#[client_visibility_filter]
const PRIVILEGED_CLIENT_FILTER: Filter = Filter::Sql(
    "SELECT c.* 
     FROM privileged_client c
     WHERE c.privilage >= (SELECT a.privilage FROM privileged_client a WHERE a.identity = :sender)"
);