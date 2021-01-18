use crate::database::*;
use crate::util::result::{Error, Result};

use mongodb::bson::doc;

#[delete("/<target>")]
pub async fn req(user: User, target: Ref) -> Result<()> {
    let target = target.fetch_channel().await?;

    let perm = permissions::channel::calculate(&user, &target).await;
    if !perm.get_view() {
        Err(Error::LabelMe)?
    }

    match target {
        Channel::SavedMessages { .. } => Err(Error::NoEffect),
        Channel::DirectMessage { .. } => {
            get_collection("channels")
                .update_one(
                    doc! {
                        "_id": target.id()
                    },
                    doc! {
                        "$set": {
                            "active": false
                        }
                    },
                    None,
                )
                .await
                .map_err(|_| Error::DatabaseError {
                    operation: "update_one",
                    with: "channel",
                })?;

            Ok(())
        }
        _ => unimplemented!(),
    }
}
