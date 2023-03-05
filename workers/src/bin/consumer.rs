use amiquip::{
    Connection,
    ConsumerOptions,
    QueueDeclareOptions,
    Delivery,
    ConsumerMessage,
    Error,
    FieldTable,
};

use std::{
    char,
    thread,
    time,
};

fn delivery_callback(delivery: Delivery) {
    let point_count = delivery.body
        .iter()
        .filter(|byte| **byte as char == '.')
        .count();

    println!(" [x] Processing msg...");
    thread::sleep(time::Duration::from_secs(point_count as u64));
    println!(" [x] Received msg {{{}}}", String::from_utf8(delivery.body).unwrap());
}

fn main() -> Result<(), Error> {
    let mut connection = Connection::insecure_open("amqp://localhost:5672/")?;
    let channel = connection.open_channel(None)?;
    let queue = channel.queue_declare("workers_queue", QueueDeclareOptions::default())?;
    let consumer = queue.consume(
            ConsumerOptions{
                no_ack: true,
                no_local: false,
                exclusive: false,
                arguments: FieldTable::new(),
            }
    )?;

    for msg in consumer.receiver() {
        match msg {
            ConsumerMessage::Delivery(delivery) => {
                delivery_callback(delivery)
            },
            other => {
                println!(" [x] Unmached consumer message received... Msg [{other:#?}]");
                break
            }
        }
    }

    connection.close()?;

    Ok(())
}