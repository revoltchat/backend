use rocket::{catch, Catcher, Request};
use revolt_result::{create_error, Error, Result};

#[catch(404)]
pub fn not_found() -> Result<()> {
    Err(create_error!(NotFound))
}

#[catch(422)]
pub fn unprocessable_entity(req: &Request) -> Result<()> {
    match req.local_cache(|| None::<Error>) {
        Some(e) => Err(e.clone()),
        None => Err(create_error!(UnprocessableEntity))
    }
}

pub fn all_catchers() -> Vec<Catcher> {
    catchers![not_found, unprocessable_entity]
}