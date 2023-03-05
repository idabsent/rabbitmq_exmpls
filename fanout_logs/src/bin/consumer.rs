use amiquip::{
    Connection,
    ExchangeDeclareOptions,
    QueueDeclareOptions,
    ExchangeType,
    Error,
    ConsumerMessage,
    ConsumerOptions,
    Delivery,
    FieldTable,
};

use std::str;

fn msg_callback(del: Delivery) {
    let msg = str::from_utf8(&del.body).unwrap();

    println!(" [x] Received msg {{{msg}}}");
}

fn main() -> Result<(), Error> {
    let mut connection = Connection::insecure_open("amqp://localhost:5672/")?;
    let channel = connection.open_channel(None)?;

    let exchange = channel.exchange_declare(ExchangeType::Fanout, "logs", ExchangeDeclareOptions::default())?;

    let queue_ops = QueueDeclareOptions{
        exclusive: true,
        auto_delete: false,
        durable: false,
        arguments: FieldTable::new(),
    };

    let queue = channel.queue_declare("", queue_ops)?;
    channel.queue_bind(queue.name(), exchange.name(), "", FieldTable::new())?;

    let consumer = queue.consume(ConsumerOptions::default())?;

    for msg in consumer.receiver() {
        match msg {
            ConsumerMessage::Delivery(del) => msg_callback(del),
            other => {
                println!(" [x] Unmatched consumer message received {{{other:#?}}}");
                break;
            }
        }
    }

    connection.close()?;

    Ok(())
}