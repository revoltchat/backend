#[cfg(feature = "serde")]
#[macro_use]
extern crate serde;

#[macro_use]
extern crate async_trait;

#[macro_use]
extern crate log;

macro_rules! auto_derived {
    ( $( $item:item )+ ) => {
        $(
            #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
            #[derive(Debug, Clone)]
            $item
        )+
    };
}

mod admin_migrations;

pub use admin_migrations::*;

pub struct Database(pub revolt_database::Database);

pub trait AbstractDatabase: Sync + Send + admin_migrations::AbstractMigrations {}
impl AbstractDatabase for revolt_database::DummyDb {}
impl AbstractDatabase for revolt_database::MongoDb {}

impl std::ops::Deref for Database {
    type Target = dyn AbstractDatabase;

    fn deref(&self) -> &Self::Target {
        match &self.0 {
            revolt_database::Database::Dummy(dummy) => dummy,
            revolt_database::Database::MongoDb(mongo) => mongo,
        }
    }
}
