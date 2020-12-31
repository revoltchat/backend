use rocket::response::NamedFile;
use std::path::Path;

#[get("/<_target>/avatar")]
pub async fn req(_target: String) -> Option<NamedFile> {
    NamedFile::open(Path::new("avatar.png")).await.ok()
}
