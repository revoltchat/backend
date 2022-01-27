use revolt_quark::Result;
use rauth::util::EmptyResponse;

#[delete("/<target>")]
pub async fn delete_bot(/*user: UserRef, target: Ref*/ target: String) -> Result<EmptyResponse> {
    todo!()
}
