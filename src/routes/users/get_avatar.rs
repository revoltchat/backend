use rocket::response::NamedFile;
use std::path::Path;

use crate::database::Ref;

#[get("/<target>/avatar")]
pub async fn req(target: Ref) -> Option<NamedFile> {
    match target.id.chars().nth(25).unwrap() {
        '0' |
        '1' |
        '2' |
        '3' |
        '4' |
        '5' |
        '6' |
        '7' => NamedFile::open(Path::new("assets/user_red.png")).await.ok(),
        '8' |
        '9' |
        'A' |
        'C' |
        'B' |
        'D' |
        'E' |
        'F' => NamedFile::open(Path::new("assets/user_green.png")).await.ok(),
        'G' |
        'H' |
        'J' |
        'K' |
        'M' |
        'N' |
        'P' |
        'Q' => NamedFile::open(Path::new("assets/user_blue.png")).await.ok(),
        'R' |
        'S' |
        'T' |
        'V' |
        'W' |
        'X' |
        'Y' |
        'Z' => NamedFile::open(Path::new("assets/user_yellow.png")).await.ok(),
        _ => unreachable!()
    }
}
