use amiquip::{
    Connection,
    Consumer,
    ConsumerOptions,
    Delivery,
    ConsumerMessage,
    QueueDeclareOptions,
    FieldTable,
    Error,
};

use std::{
    thread,
    time,
};

fn msg_callback(consumer: &Consumer, del: Delivery) {
    let point_count = del.body
            .iter()
            .filter(|byte| **byte as char == '.')
            .count();

    println!(" [x] Processing message...");
    thread::sleep(time::Duration::from_secs(point_count as u64));
    println!(" [x] Message received {{{}}}", String::from_utf8(del.body.clone()).unwrap());

    consumer.ack(del).unwrap();
}

fn main() -> Result<(), Error> {
    let mut connection = Connection::insecure_open("amqp://localhost:5672/")?;
    let channel = connection.open_channel(None)?;
    let queue = channel.queue_declare(
            "full_workers_queue",
            QueueDeclareOptions{
                durable: true,
                exclusive: false,
                auto_delete: false,
                arguments: FieldTable::new(),
            },
    )?;
    channel.qos(0, 1, false)?;
    let consumer = queue.consume(ConsumerOptions::default())?;

    for msg in consumer.receiver() {
        match msg {
            ConsumerMessage::Delivery(del) => {
                msg_callback(&consumer, del)
            },
            other => {
                println!(" [x] Received unmached consumer message {other:#?}");
                break;
            },
        }
    }

    connection.close()?;

    Ok(())
}