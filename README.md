# mqtt-channel-client

[![CI](https://github.com/DanNixon/mqtt-channel-client-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/DanNixon/mqtt-channel-client-rs/actions/workflows/ci.yml)
[![dependency status](https://deps.rs/repo/github/dannixon/mqtt-channel-client-rs/status.svg)](https://deps.rs/repo/github/dannixon/mqtt-channel-client-rs)

MQTT client that communicates over Tokio channels.

I found this to be a very common pattern I repeated in several event driven applications that involved communication over MQTT.

This library aims to abstract common MQTT functionality and provide a simple event based topic/message interface to an MQTT broker.
