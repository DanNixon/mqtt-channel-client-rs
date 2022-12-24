#[cfg(feature = "metrics")]
use crate::metrics::{ConnectionEventLabels, MessageLabelsBuilder, MetricCollection};
use crate::{
    events::{Event, StatusEvent},
    ClientConfig, Subscription,
};
use paho_mqtt::{AsyncClient, ConnectOptions, CreateOptions, Message};
#[cfg(feature = "metrics")]
use prometheus_client::registry::Registry;
use std::sync::{Arc, Mutex};
use tokio::{
    sync::broadcast::{self, Receiver, Sender},
    task::JoinHandle,
};

/// Channel based MQTT client.
#[derive(Clone)]
pub struct Client {
    client: AsyncClient,
    #[allow(dead_code)]
    config: ClientConfig,
    subscriptions: Arc<Mutex<Vec<Subscription>>>,

    tx_channel: Sender<Event>,
    handle: Arc<tokio::sync::Mutex<Option<JoinHandle<()>>>>,

    #[cfg(feature = "metrics")]
    metrics: MetricCollection,
}

impl Client {
    /// Create a new client using the supplied options.
    pub fn new(options: CreateOptions, config: ClientConfig) -> Result<Self, crate::Error> {
        let (tx, _) = broadcast::channel::<Event>(config.channel_size);

        Ok(Self {
            client: AsyncClient::new(options)?,
            config,
            subscriptions: Default::default(),

            tx_channel: tx,
            handle: Default::default(),

            #[cfg(feature = "metrics")]
            metrics: Default::default(),
        })
    }

    /// Get a sending channel for sending events to the client.
    pub fn tx_channel(&self) -> Sender<Event> {
        self.tx_channel.clone()
    }

    /// Get a receiving channel for consuming events from the client.
    pub fn rx_channel(&self) -> Receiver<Event> {
        self.tx_channel.subscribe()
    }

    /// Send a message.
    pub fn send(&self, msg: Message) -> crate::Result<()> {
        self.tx_channel.send(Event::Tx(msg))?;
        Ok(())
    }

    /// Add a new subscription.
    ///
    /// If the client is currently connected then the subscription takes effect for the connected
    /// session.
    ///
    /// In all cases the subscription is added to the cache to be subscribed on reconnect.
    pub fn subscribe(&self, subscription: Subscription) {
        // Add to the cached list of subscriptions
        self.subscriptions
            .lock()
            .unwrap()
            .push(subscription.clone());

        // Subscribe now if the client is connected
        if self.client.is_connected() {
            log::debug!(
                "Adding subscription to active client: {}",
                subscription.topic
            );
            self.client.subscribe(subscription.topic, subscription.qos);
        }
    }

    /// Register metrics with a registry.
    #[cfg(feature = "metrics")]
    pub fn register_metrics(&self, registry: &mut Registry) {
        let registry = registry.sub_registry_with_prefix(&self.config.metrics_prefix);

        registry.register(
            "messages",
            "MQTT messages processed",
            Box::new(self.metrics.messages.clone()),
        );

        registry.register(
            "connection_events",
            "MQTT broker connection change events",
            Box::new(self.metrics.connection_events.clone()),
        );
    }

    /// Start the client using the supplied connection options.
    pub async fn start(&mut self, options: ConnectOptions) -> crate::Result<()> {
        if self.handle.lock().await.is_some() {
            return Err(crate::Error::ClientAlreadyStarted);
        }

        let client = self.client.clone();

        let tx_channel = self.tx_channel.clone();
        let subscriptions = self.subscriptions.clone();
        #[cfg(feature = "metrics")]
        let metrics = self.metrics.clone();
        client.set_connected_callback(move |c| {
            log::debug!("Connected");

            #[cfg(feature = "metrics")]
            metrics
                .connection_events
                .get_or_create(&ConnectionEventLabels::connected())
                .inc();

            if let Err(e) = tx_channel.send(Event::Status(StatusEvent::Connected)) {
                log::error!("Failed to send event: {}", e);
            }

            for s in &*subscriptions.lock().unwrap() {
                c.subscribe(&s.topic, s.qos);
            }
        });

        let tx_channel = self.tx_channel.clone();
        #[cfg(feature = "metrics")]
        let metrics = self.metrics.clone();
        client.set_disconnected_callback(move |_c, _p, _r| {
            log::debug!("Disconnected");

            #[cfg(feature = "metrics")]
            metrics
                .connection_events
                .get_or_create(&ConnectionEventLabels::disconnected())
                .inc();

            if let Err(e) = tx_channel.send(Event::Status(StatusEvent::Disconnected)) {
                log::error!("Failed to send event: {}", e);
            }
        });

        let tx_channel = self.tx_channel.clone();
        #[cfg(feature = "metrics")]
        let metrics = self.metrics.clone();
        client.set_connection_lost_callback(move |_c| {
            log::debug!("Connection lost");

            #[cfg(feature = "metrics")]
            metrics
                .connection_events
                .get_or_create(&ConnectionEventLabels::lost())
                .inc();

            if let Err(e) = tx_channel.send(Event::Status(StatusEvent::Disconnected)) {
                log::error!("Failed to send event: {}", e);
            }
        });

        let tx_channel = self.tx_channel.clone();
        #[cfg(feature = "metrics")]
        let metrics = self.metrics.clone();
        client.set_message_callback(move |_c, msg| {
            if let Some(msg) = msg {
                log::debug!("Received message on topic \"{}\"", msg.topic());

                #[cfg(feature = "metrics")]
                metrics
                    .messages
                    .get_or_create(
                        &MessageLabelsBuilder::default()
                            .received()
                            .topic(msg.topic().into())
                            .build()
                            .unwrap(),
                    )
                    .inc();

                if let Err(e) = tx_channel.send(Event::Rx(msg)) {
                    log::error!("Failed to send event: {}", e);
                }
            }
        });

        let response = client.connect(Some(options)).wait()?;
        log::debug!(
            "Using MQTT version {}",
            response.connect_response().unwrap().mqtt_version
        );

        let tx_channel = self.tx_channel.clone();
        let mut rx_channel = tx_channel.subscribe();
        #[cfg(feature = "metrics")]
        let metrics = self.metrics.clone();
        *self.handle.lock().await = Some(tokio::spawn(async move {
            loop {
                match rx_channel.recv().await {
                    // Send any messages that are available
                    Ok(Event::Tx(msg)) => {
                        log::debug!("Sending message on topic \"{}\"", msg.topic());

                        #[cfg(feature = "metrics")]
                        let topic = msg.topic().to_string();

                        match client.try_publish(msg) {
                            Ok(delivery_token) => {
                                #[cfg(feature = "metrics")]
                                metrics
                                    .messages
                                    .get_or_create(
                                        &MessageLabelsBuilder::default()
                                            .sent()
                                            .topic(topic)
                                            .build()
                                            .unwrap(),
                                    )
                                    .inc();

                                if let Err(e) = delivery_token.wait() {
                                    log::error!("Error sending message: {}", e);
                                }
                            }
                            Err(e) => {
                                #[cfg(feature = "metrics")]
                                metrics
                                    .messages
                                    .get_or_create(
                                        &MessageLabelsBuilder::default()
                                            .sent()
                                            .topic(topic)
                                            .failure()
                                            .build()
                                            .unwrap(),
                                    )
                                    .inc();

                                log::error!("Error creating/queuing the message: {}", e)
                            }
                        }
                    }
                    // Exit if requested
                    Ok(Event::Stop) => {
                        log::debug!("Stopped");
                        return;
                    }
                    Err(e) => {
                        log::warn!("Receive error: {}", e);
                    }
                    _ => {}
                }
            }
        }));

        Ok(())
    }

    /// Request for the client to be stopped and wait for it to terminate.
    pub async fn stop(&mut self) -> crate::Result<()> {
        log::trace!("Stopping client");

        // Send termination request
        self.tx_channel.send(Event::Stop)?;

        // Wait for task to exit
        match self.handle.lock().await.take() {
            Some(handle) => Ok(handle.await?),
            None => Err(crate::Error::ClientAlreadyStopped),
        }
    }
}
