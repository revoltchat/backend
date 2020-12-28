use crate::util::result::Result;

#[get("/<id>")]
pub async fn req(id: String) -> Result<String> {
    println!("{}", id);
    Ok("LETS FUCKING GOOOO".to_string())
}
