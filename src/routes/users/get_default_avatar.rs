use rocket::{Request, Response};
use rocket::response::{self, Responder};
use rocket::fs::NamedFile;
use std::path::Path;

use crate::database::Ref;

pub struct CachedFile(NamedFile);

pub static CACHE_CONTROL: &'static str = "public, max-age=31536000, immutable";

impl<'r> Responder<'r, 'static> for CachedFile {
    fn respond_to(self, req: &'r Request<'_>) -> response::Result<'static> {
        Response::build_from(self.0.respond_to(req)?)
            .raw_header("Cache-control", CACHE_CONTROL)
            .ok()
    }
}

#[get("/<target>/default_avatar")]
pub async fn req(target: Ref) -> Option<CachedFile> {
    match target.id.chars().nth(25).unwrap() {
        '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' => {
            NamedFile::open(Path::new("assets/user_red.png")).await.ok().map(|n| CachedFile(n))
        }
        '8' | '9' | 'A' | 'C' | 'B' | 'D' | 'E' | 'F' => {
            NamedFile::open(Path::new("assets/user_green.png"))
                .await
                .ok().map(|n| CachedFile(n))
        }
        'G' | 'H' | 'J' | 'K' | 'M' | 'N' | 'P' | 'Q' => {
            NamedFile::open(Path::new("assets/user_blue.png"))
                .await
                .ok().map(|n| CachedFile(n))
        }
        'R' | 'S' | 'T' | 'V' | 'W' | 'X' | 'Y' | 'Z' => {
            NamedFile::open(Path::new("assets/user_yellow.png"))
                .await
                .ok().map(|n| CachedFile(n))
        }
        _ => unreachable!(),
    }
}
