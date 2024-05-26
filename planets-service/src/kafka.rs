use std::sync::Mutex;
use std::time::Duration;

use lazy_static::lazy_static;
use rdkafka::config::RDKafkaLogLevel;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::util::Timeout;
use rdkafka::ClientConfig;

lazy_static! {
    static ref KAFKA_BROKER: String =
        std::env::var("KAFKA_BROKER").expect("Can't read Kafka broker address");
    static ref KAFKA_TOPIC: String =
        std::env::var("KAFKA_TOPIC").expect("Can't read Kafka topic name");
}

pub fn create_producer() -> FutureProducer {
    ClientConfig::new()
        .set("bootstrap.servers", KAFKA_BROKER.as_str())
        .set("message.timeout.ms", "5000")
        .create()
        .expect("Producer creation failed")
}

pub fn create_consumer(group_id: String) -> StreamConsumer {
    let consumer: StreamConsumer = ClientConfig::new()
        .set("group.id", &group_id)
        .set("bootstrap.servers", KAFKA_BROKER.as_str())
        .set("enable.partition.eof", "false")
        .set("session.timeout.ms", "6000")
        .set("enable.auto.commit", "true")
        .set_log_level(RDKafkaLogLevel::Debug)
        .create()
        .expect("Consumer creation failed");

    consumer
        .subscribe(&[&KAFKA_TOPIC])
        .expect("Can't subscribe to specified topics");

    consumer
}

pub fn get_kafka_consumer_group_id(kafka_consumer_counter: &Mutex<i32>) -> String {
    let mut counter = kafka_consumer_counter.lock().expect("Can't lock counter");
    *counter += 1;
    format!("graphql-group-{}", *counter)
}

// TODO: send without caller blocking
pub async fn send_message(producer: &FutureProducer, message: &str) {
    let delivery_status = producer
        .send(
            FutureRecord::to(&KAFKA_TOPIC)
                .payload(message)
                .key("new_planet"),
            Timeout::After(Duration::from_secs(0)),
        )
        .await;

    match delivery_status {
        Ok(_) => println!("Message was sent"),
        Err(res) => println!("Message wasn't sent: {}", res.0),
    }
}
