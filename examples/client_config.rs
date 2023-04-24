use mqtt_channel_client::{
    paho_mqtt::{
        connect_options::ConnectOptionsBuilder, create_options::CreateOptionsBuilder, Message,
        PersistenceType,
    },
    Client, ClientConfigBuilder, Event, SubscriptionBuilder,
};
use prometheus_client::{encoding::text::encode, registry::Registry};
use std::time::Duration;

#[tokio::main]
async fn main() {
    env_logger::init();

    // Create the client
    let client = Client::new(
        CreateOptionsBuilder::new()
            .server_uri("tcp://localhost:1883")
            .client_id("demo")
            .persistence(PersistenceType::None)
            .finalize(),
        ClientConfigBuilder::default()
            .metrics_prefix("another_mqtt".into())
            .build()
            .unwrap(),
    )
    .unwrap();

    let mut registry = Registry::default();
    client.register_metrics(&mut registry);

    // Start a task to print metrics
    let metrics_print_task = tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(5));

        loop {
            let mut encoded = String::new();
            encode(&mut encoded, &registry).unwrap();
            println!("Metrics:\n{}", encoded);

            interval.tick().await;
        }
    });

    // Start a task to reply to pings
    let tx = client.tx_channel();
    let mut rx = client.rx_channel();
    let pong_task = tokio::spawn(async move {
        loop {
            if let Ok(Event::Rx(msg)) = rx.recv().await {
                if msg.topic().starts_with("ping/") {
                    let topic = format!("pong/{}", msg.topic().strip_prefix("ping/").unwrap());
                    tx.send(Event::Tx(Message::new(topic, msg.payload(), msg.qos())))
                        .unwrap();
                }
            }
        }
    });

    // Add a subscription
    client.subscribe(
        SubscriptionBuilder::default()
            .topic("ping/+".into())
            .build()
            .unwrap(),
    );

    // Connect to the broker
    client
        .start(
            ConnectOptionsBuilder::new()
                .clean_session(true)
                .automatic_reconnect(Duration::from_secs(1), Duration::from_secs(5))
                .keep_alive_interval(Duration::from_secs(5))
                .user_name("me")
                .password("my_password")
                .finalize(),
        )
        .await
        .unwrap();

    // Wait for an exit signal
    tokio::signal::ctrl_c().await.unwrap();
    println!("Exiting...");

    // Disconnect from the broker
    client.stop().await.unwrap();

    // Exit tasks
    pong_task.abort();
    metrics_print_task.abort();
}
