#[macro_use]
extern crate serde;

use std::sync::OnceLock;

use neon::prelude::*;
use revolt_database::{Database, DatabaseInfo};

fn js_init(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    static INIT: OnceLock<()> = OnceLock::new();
    if INIT.get().is_none() {
        INIT.get_or_init(|| {
            async_std::task::block_on(async {
                revolt_config::configure!(api);

                match DatabaseInfo::Auto.connect().await {
                    Ok(db) => {
                        let authifier_db = db.clone().to_authifier().await.database;
                        revolt_database::tasks::start_workers(db, authifier_db);
                        Ok(())
                    }
                    Err(err) => Err(err),
                }
            })
            .or_else(|err| cx.throw_error(err))
            .unwrap();
        });
    }

    Ok(cx.undefined())
}

struct DatabaseBinding(Database, Channel);
impl Finalize for DatabaseBinding {}
impl DatabaseBinding {
    fn take(&self) -> (Database, Channel) {
        (self.0.clone(), self.1.clone())
    }
}

fn js_database(mut cx: FunctionContext) -> JsResult<JsBox<DatabaseBinding>> {
    let db = async_std::task::block_on(DatabaseInfo::Auto.connect())
        .or_else(|err| cx.throw_error(err))?;

    let channel = cx.channel();
    Ok(cx.boxed(DatabaseBinding(db, channel)))
}

// Implementations for models
#[derive(Clone)]
#[allow(clippy::large_enum_variant)]
enum Model {
    User(revolt_database::User),
    Error(revolt_result::Error),
}

impl Model {
    fn give(&self) -> Model {
        self.clone()
    }
}

impl Finalize for Model {}

macro_rules! shim_boxed {
    ($cx: ident, $name: ident, $model: ident, $( $variable: ident $type: ident $id: expr )+, $cmd: ident, $( $arg: expr, )+) => {
        fn $name(mut cx: FunctionContext) -> JsResult<JsPromise> {
            $(
                let $variable = cx.argument::<$type>($id)?.value(&mut cx);
            )+

            let (db, channel) = cx.this::<JsBox<DatabaseBinding>>()?.take();
            let (deferred, promise) = cx.promise();

            async_std::task::spawn(async move {
                let result = db.$cmd($($arg,)+).await;
                deferred.settle_with(&channel, move |mut cx| {
                    Ok(cx.boxed(match result {
                        Ok(value) => Model::$model(value),
                        Err(error) => Model::Error(error)
                    }))
                })
            });

            Ok(promise)
        }

        $cx.export_function(stringify!($name), $name)?;
    };
}

fn js_data(mut cx: FunctionContext) -> JsResult<JsValue> {
    match cx.this::<JsBox<Model>>()?.give() {
        Model::Error(_) => neon_serde4::to_value(&mut cx, &None::<()>),
        Model::User(user) => neon_serde4::to_value(&mut cx, &user),
    }
    .or_else(|e| cx.throw_error(e.to_string()))
}

fn js_error(mut cx: FunctionContext) -> JsResult<JsValue> {
    let value = match cx.this::<JsBox<Model>>()?.give() {
        Model::Error(err) => Some(err),
        _ => None,
    };

    neon_serde4::to_value(&mut cx, &value).or_else(|e| cx.throw_error(e.to_string()))
}

// Basic data implementation
#[derive(Serialize, Deserialize)]
struct ResultBinding<T> {
    #[serde(flatten)]
    value: Option<T>,
    error: Option<revolt_result::Error>,
}

macro_rules! shim {
    ($cx: ident, $name: ident, $( $variable: ident $type: ident $id: expr )*, $( $model: ident $modelType: ident $modelId: expr )*, | $db: ident | $closure: expr, $( $arg: expr, )+) => {
        fn $name(mut cx: FunctionContext) -> JsResult<JsPromise> {
            $(
                let $variable = cx.argument::<$type>($id)?.value(&mut cx);
            )*

            $(
                let mut $model = if let Model::$modelType(value) = cx.argument::<JsBox<Model>>($modelId)?.give() {
                    value
                } else {
                    unreachable!()
                };
            )*

            let (db, channel) = cx.this::<JsBox<DatabaseBinding>>()?.take();
            let (deferred, promise) = cx.promise();

            async_std::task::spawn(async move {
                #[allow(clippy::redundant_closure_call)]
                let result = (|$db: $crate::Database| $closure)(db.clone()).await;
                deferred.settle_with(&channel, move |mut cx| {
                    neon_serde4::to_value(
                        &mut cx,
                        &match result {
                            Ok(value) => ResultBinding {
                                value: Some(value),
                                error: None,
                            },
                            Err(error) => ResultBinding {
                                value: None,
                                error: Some(error),
                            },
                        },
                    )
                    .or_else(|e| cx.throw_error(e.to_string()))
                })
            });

            Ok(promise)
        }

        $cx.export_function(stringify!($name), $name)?;
    };
}

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    // initialise required background stuff
    cx.export_function("init", js_init)?;

    // database & model stuff
    cx.export_function("database", js_database)?;
    cx.export_function("model_data", js_data)?;
    cx.export_function("model_error", js_error)?;

    shim_boxed!(
        cx,
        database_fetch_user,
        User,
        user_id JsString 0,
        fetch_user,
        &user_id,
    );

    shim_boxed!(
        cx,
        database_fetch_user_by_username,
        User,
        username JsString 0
        discriminator JsString 1,
        fetch_user_by_username,
        &username, &discriminator,
    );

    // procedure calls
    shim!(
        cx,
        proc_channels_create_dm,
        user_a JsString 0
        user_b JsString 1,
        ,
        |db| async move {
            let user_a = db.fetch_user(&user_a).await?;
            let user_b = db.fetch_user(&user_b).await?;
            revolt_database::Channel::create_dm(&db, &user_a, &user_b).await
        },
        &userA, &userB,
    );

    shim!(
        cx,
        proc_users_suspend,
        duration JsNumber 1
        reason JsString 2,
        user User 0,
        |db| async move {
            let duration = duration as usize;
            user.suspend(&db, if duration == 0 { None } else { Some(duration) }, Some(reason.split('|').map(|x| x.to_owned()).collect())).await
        },
        &user,
    );

    Ok(())
}
