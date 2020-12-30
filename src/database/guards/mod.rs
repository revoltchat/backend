pub mod reference;
pub mod user;

/*
// ! FIXME
impl<'r> FromParam<'r> for User {
    type Error = &'r RawStr;

    fn from_param(param: &'r RawStr) -> Result<Self, Self::Error> {
        Err(param)
        /*if let Ok(result) = fetch_channel(param).await {
            if let Some(channel) = result {
                Ok(channel)
            } else {
                Err(param)
            }
        } else {
            Err(param)
        }*/
    }
}
*/
