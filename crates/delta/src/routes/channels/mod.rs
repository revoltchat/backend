use revolt_rocket_okapi::revolt_okapi::openapi3::OpenApi;
use rocket::Route;

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
mod message_react;
mod message_search;
mod message_send;
mod message_unreact;
mod permissions_set;
mod permissions_set_default;
mod voice_join;
mod webhook_create;
mod webhook_fetch_all;

pub fn routes() -> (Vec<Route>, OpenApi) {
    openapi_get_routes_spec![
        channel_ack::ack,
        channel_fetch::fetch,
        members_fetch::fetch_members,
        channel_delete::delete,
        channel_edit::edit,
        invite_create::create_invite,
        message_send::message_send,
        message_query::req,
        message_search::req,
        message_fetch::fetch,
        message_edit::edit,
        message_bulk_delete::bulk_delete_messages,
        message_delete::delete,
        group_create::create_group,
        group_add_member::add_member,
        group_remove_member::remove_member,
        voice_join::call,
        permissions_set::req,
        permissions_set_default::req,
        message_react::react_message,
        message_unreact::unreact_message,
        message_clear_reactions::clear_reactions,
        webhook_create::create_webhook,
        webhook_fetch_all::fetch_webhooks,
    ]
}
