use rocket::Route;
use rocket_okapi::okapi::openapi3::OpenApi;

mod channel_ack;
mod channel_delete;
mod channel_edit;
mod channel_fetch;
mod group_add_member;
mod group_create;
mod group_remove_member;
mod invite_create;
mod members_fetch;
mod message_bulk_delete;
mod message_clear_reactions;
mod message_delete;
mod message_edit;
mod message_fetch;
mod message_query;
mod message_query_stale;
mod message_react;
mod message_search;
mod message_send;
mod message_unreact;
mod permissions_set;
mod permissions_set_default;
mod voice_join;

pub fn routes() -> (Vec<Route>, OpenApi) {
    openapi_get_routes_spec![
        channel_ack::req,
        channel_fetch::req,
        members_fetch::req,
        channel_delete::req,
        channel_edit::req,
        invite_create::req,
        message_send::message_send,
        message_query::req,
        message_search::req,
        message_query_stale::req,
        message_fetch::req,
        message_edit::req,
        message_bulk_delete::req,
        message_delete::req,
        group_create::req,
        group_add_member::req,
        group_remove_member::req,
        voice_join::req,
        permissions_set::req,
        permissions_set_default::req,
        message_react::react_message,
        message_unreact::unreact_message,
        message_clear_reactions::clear_reactions
    ]
}
