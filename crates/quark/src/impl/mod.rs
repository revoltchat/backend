mod dummy;
mod generic;
mod mongo;

#[cfg(feature = "rocket_impl")]
mod rocket;

pub use self::generic::users::user_settings::UserSettingsImpl;
pub use dummy::DummyDb;
pub use mongo::MongoDb;
