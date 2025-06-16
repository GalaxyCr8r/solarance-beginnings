use spacetimedb::{client_visibility_filter, Filter};

// You can only see sector objects in your current sector. TODO: In the future, this might be expanded to include anyone in your faction.
#[client_visibility_filter]
const SO_SHIPOBJECT_FILTER: Filter = Filter::Sql(
    "SELECT o.* 
    FROM stellar_object o
    JOIN ship s ON s.sector_id = o.sector_id"
);

/// You can only see your own window
#[client_visibility_filter]
const OWN_WINDOW_FILTER: Filter = Filter::Sql(
    "SELECT w.* 
    FROM sobj_player_window w
    WHERE w.player_id = :sender"
);

/// You can only see high "resolution" transforms within your window.
#[client_visibility_filter]
const HR_OBJECT_FILTER: Filter = Filter::Sql( //// TODO: Add a sector_id check to this which require adding sector_id to sobj_hi_res_transform & player window
    "SELECT o.* 
    FROM sobj_hi_res_transform o
    JOIN sobj_player_window w
    WHERE (o.x > w.tl_x AND 
          o.y > w.tl_y AND 
          o.x < w.br_x AND 
          o.y < w.br_y)"
);

/// You can only see low "resolution" transforms outside your window. 
/// (Might generalize this to ALL low-res transforms to avoid a second heavy "WHERE" clause)
#[client_visibility_filter]
const LR_OBJECT_FILTER: Filter = Filter::Sql( //// TODO: Add a sector_id check to this which require adding sector_id to sobj_hi_res_transform & player window
    "SELECT o.* 
    FROM sobj_low_res_transform o
    JOIN sobj_player_window w
    WHERE (o.x <= w.tl_x OR 
          o.y <= w.tl_y OR 
          o.x >= w.br_x OR 
          o.y >= w.br_y)"
);