use rocket::local::asynchronous::Client;
use std::ops::Deref;

pub struct TestHarness {
    client: Client,
}

impl TestHarness {
    pub async fn new() -> TestHarness {
        dotenv::dotenv().ok();

        let client = Client::tracked(crate::web().await)
            .await
            .expect("valid rocket instance");

        TestHarness { client }
    }
}

impl Deref for TestHarness {
    type Target = Client;

    fn deref(&self) -> &Self::Target {
        &self.client
    }
}
