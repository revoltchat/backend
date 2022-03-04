use rocket::http::ContentType;
use rocket::response::{self, Responder};
use rocket::{Request, Response};

pub struct CachedFile((ContentType, Vec<u8>));

pub static CACHE_CONTROL: &str = "public, max-age=31536000, immutable";

impl<'r> Responder<'r, 'static> for CachedFile {
    fn respond_to(self, req: &'r Request<'_>) -> response::Result<'static> {
        Response::build_from(self.0.respond_to(req)?)
            .raw_header("Cache-Control", CACHE_CONTROL)
            .ok()
    }
}

// Charset: 0123456789ABCDEFGHJKMNPQRSTVWXYZ

#[get("/<target>/default_avatar")]
pub async fn req(target: String) -> CachedFile {
    CachedFile((
        ContentType::PNG,
        match target.chars().last().unwrap() {
            '0' | '1' | '2' | '3' | 'S' | 'Z' => {
                include_bytes!("../../../assets/user/2.png").to_vec()
            }
            '4' | '5' | '6' | '7' | 'T' => include_bytes!("../../../assets/user/3.png").to_vec(),
            '8' | '9' | 'A' | 'B' => include_bytes!("../../../assets/user/4.png").to_vec(),
            'C' | 'D' | 'E' | 'F' | 'V' => include_bytes!("../../../assets/user/5.png").to_vec(),
            'G' | 'H' | 'J' | 'K' | 'W' => include_bytes!("../../../assets/user/6.png").to_vec(),
            'M' | 'N' | 'P' | 'Q' | 'X' => include_bytes!("../../../assets/user/7.png").to_vec(),
            /*'0' | '1' | '2' | '3' | 'R' | 'Y'*/
            _ => include_bytes!("../../../assets/user/1.png").to_vec(),
        },
    ))
}
