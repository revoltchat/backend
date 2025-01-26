use amqprs::{
    channel::{BasicPublishArguments, Channel},
    connection::{Connection, OpenConnectionArguments},
    BasicProperties,
};
use log::{debug, warn};

pub(crate) trait Channeled {
    #[allow(unused)]
    fn get_connection(&self) -> Option<&Connection>;
    fn get_channel(&self) -> Option<&Channel>;
    fn set_connection(&mut self, conn: Connection);
    fn set_channel(&mut self, channel: Channel);
}

pub(crate) async fn make_channel<T: Channeled>(consumer: &mut T) {
    let config = revolt_config::config().await;

    let args = OpenConnectionArguments::new(
        &config.rabbit.host,
        config.rabbit.port,
        &config.rabbit.username,
        &config.rabbit.password,
    );
    let conn = amqprs::connection::Connection::open(&args).await.unwrap();

    let channel = conn.open_channel(None).await.unwrap();

    consumer.set_connection(conn);
    consumer.set_channel(channel);
}

pub(crate) async fn publish_message<T: Channeled>(
    consumer: &mut T,
    payload: Vec<u8>,
    args: BasicPublishArguments,
) {
    let routing_key = &args.routing_key.clone();
    let mut channel = consumer.get_channel();
    if channel.is_none() {
        make_channel(consumer).await;
        channel = consumer.get_channel();
    }

    if let Some(chnl) = channel {
        chnl.basic_publish(BasicProperties::default(), payload.clone(), args.clone())
            .await
            .unwrap();
        debug!("Sent message to queue for target {}", routing_key);
    } else {
        warn!("Failed to unwrap channel (including attempt to make a channel)!")
    }
}
