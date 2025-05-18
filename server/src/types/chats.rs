use log::info;
use spacetimedb::{table, Identity, ReducerContext, Timestamp};
use spacetimedsl::dsl;

use crate::types::players::get_username;

#[dsl(plural_name = global_chat_messages)]
#[table(name = global_chat_message, public)]
pub struct GlobalChatMessage {
    #[primary_key]
    #[auto_inc]
    #[wrap]
    pub id: u64,

    pub identity: Identity, // FK to Player

    pub message: String,

    created_at: Timestamp,
}

//// Impls ///

impl GlobalChatMessage {
    //
}

//// Reducers ///

#[spacetimedb::reducer]
pub fn send_global_chat(ctx: &ReducerContext, chat_message: String) -> Result<(), String> {
    let dsl = dsl(ctx);

    // If ctx.sender is a valid, unbanned, unmuted player
    info!("Message received: [{}]: {}", get_username(ctx, ctx.sender), chat_message);

    dsl.create_global_chat_message(ctx.sender, &chat_message)?;
    Ok(())
}
