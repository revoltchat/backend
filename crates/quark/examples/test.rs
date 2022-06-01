use revolt_quark::models::user::PartialUser;
use revolt_quark::presence::{
    presence_create_session, presence_delete_session, presence_filter_online, presence_is_online,
};
use revolt_quark::*;

#[async_std::main]
async fn main() {
    let db = DatabaseInfo::Dummy.connect().await.unwrap();

    let sus = PartialUser {
        username: Some("neat".into()),
        ..Default::default()
    };

    db.update_user("user id", &sus, vec![]).await.unwrap();

    dbg!(presence_create_session("entry", 0).await);
    dbg!(presence_is_online("entry").await);
    dbg!(presence_filter_online(&["a".into(), "b".into(), "entry".into()]).await);

    dbg!(presence_delete_session("entry", 0).await);
    dbg!(presence_is_online("entry").await);

    dbg!(presence_create_session("entry", 0).await);
    dbg!(presence_create_session("entry", 0).await);
    dbg!(presence_delete_session("entry", 0).await);
    dbg!(presence_is_online("entry").await);
    dbg!(presence_delete_session("entry", 1).await);
    dbg!(presence_is_online("entry").await);

    // __set_key("dietz", vec![0xFF]).await;
    // dbg!(presence_filter_online(&["dietz".into(), "nuts".into()]).await);
}
