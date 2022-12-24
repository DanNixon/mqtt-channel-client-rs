use mqtt_channel_client::{Client, ClientConfig, Event, SubscriptionBuilder};
use paho_mqtt::{
    connect_options::ConnectOptionsBuilder, create_options::CreateOptionsBuilder, Message,
    PersistenceType,
};
use std::time::Duration;

#[tokio::main]
async fn main() {
    env_logger::init();

    // Create the client
    let mut client = Client::new(
        CreateOptionsBuilder::new()
            .server_uri("tcp://localhost:1883")
            .client_id("demo")
            .persistence(PersistenceType::None)
            .finalize(),
        ClientConfig::default(),
    )
    .unwrap();

    // Start a task to process events
    let tx = client.tx_channel();
    let c2 = client.clone();
    let processing_task = tokio::spawn(async move {
        let mut rx = tx.subscribe();
        loop {
            let event = rx.recv().await;
            println!("Event: {:?}", event);

            if let Ok(Event::Rx(msg)) = event {
                // Subscribe to a topic
                if msg.topic() == "subscribe_to" {
                    c2.subscribe(
                        SubscriptionBuilder::default()
                            .topic(msg.payload_str().to_string())
                            .build()
                            .unwrap(),
                    );
                }

                // Send messages
                tx.send(Event::Tx(Message::new(
                    format!("received/{}", msg.topic()),
                    msg.payload(),
                    msg.qos(),
                )))
                .unwrap();
            }
        }
    });

    // Add a subscription
    client.subscribe(
        SubscriptionBuilder::default()
            .topic("subscribe_to".into())
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

    // Exit event processing task
    processing_task.abort();
}
