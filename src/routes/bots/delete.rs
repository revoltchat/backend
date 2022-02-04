use rauth::util::EmptyResponse;
use revolt_quark::Result;

#[delete("/<target>")]
pub async fn delete_bot(/*user: UserRef, target: Ref*/ target: String,) -> Result<EmptyResponse> {
    todo!()
}
