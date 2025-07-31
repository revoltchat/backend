use revolt_rocket_okapi::{
    r#gen::OpenApiGenerator,
    request::OpenApiFromData,
    response::OpenApiResponderInner,
    revolt_okapi::{openapi3::{RequestBody, Responses}},
};
use rocket::{data::{Data, FromData, Outcome}, form};
use rocket::request::Request;
use schemars::JsonSchema;
use revolt_result::{create_error, Error};

pub struct Validate<T>(pub T);

impl<T> Validate<T> {
    pub fn into_inner(self) -> T {
        self.0
    }
}

#[async_trait]
impl<'r, T: FromData<'r, Error = Error> + validator::Validate> FromData<'r> for Validate<T> {
    type Error = Error;

    async fn from_data(req: &'r Request<'_>, data: Data<'r>) -> Outcome<'r, Self> {
        T::from_data(req, data).await.and_then(|inner| {
            if let Err(e) = inner.validate() {
                let error = create_error!(FailedValidation { error: e.to_string() });
                req.local_cache(|| Some(error.clone()));

                return Outcome::Error((error.rocket_status(), error))
            };

            Outcome::Success(Self(inner))
        })
    }
}

#[async_trait]
impl<'r, T: form::FromForm<'r> + JsonSchema> form::FromForm<'r> for Validate<T> {
    type Context = T::Context;

    fn init(opts: form::Options) -> Self::Context {
        <T as form::FromForm<'r>>::init(opts)
    }

    fn push_value(ctxt: &mut Self::Context, field: form::ValueField<'r>) {
        <T as form::FromForm<'r>>::push_value(ctxt, field);
    }

    async fn push_data(ctxt: &mut Self::Context, field: form::DataField<'r, '_>) {
        <T as form::FromForm<'r>>::push_data(ctxt, field).await;
    }

    fn finalize(ctxt: Self::Context) -> form::Result<'r, Self>  {
        <T as form::FromForm<'r>>::finalize(ctxt).map(Self)
    }
}

impl<'r, T: FromData<'r, Error = Error> + OpenApiFromData<'r> + validator::Validate> OpenApiFromData<'r> for Validate<T> {
    fn request_body(gen: &mut OpenApiGenerator) -> revolt_rocket_okapi::Result<RequestBody> {
        T::request_body(gen)
    }
}

impl<T: JsonSchema> JsonSchema for Validate<T> {
    fn schema_name() -> String {
        T::schema_name()
    }

    fn json_schema(generator: &mut schemars::r#gen::SchemaGenerator) -> schemars::schema::Schema {
        T::json_schema(generator)
    }
}

impl<T: OpenApiResponderInner> OpenApiResponderInner for Validate<T> {
    fn responses(gen: &mut OpenApiGenerator) -> revolt_rocket_okapi::Result<Responses> {
        T::responses(gen)
    }
}

impl<T> std::ops::Deref for Validate<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
