use amiquip::{
    Connection,
    ExchangeType,
    ExchangeDeclareOptions,
    QueueDeclareOptions,
    Consumer,
    ConsumerMessage,
    ConsumerOptions,
    Delivery,
    Error,
    FieldTable,
};

use std::{
    env,
    process,
    str,
};

fn msg_callback(consumer: &Consumer, delivery: Delivery) {
    let msg = str::from_utf8(&delivery.body).unwrap();

    eprintln!(" [x] Received msg {{{msg}}}");

    consumer.ack(delivery).unwrap();
}

fn main() -> Result<(), Error> {
    let args: Vec<String> = env::args().collect();

    if args.len() == 1 {
        let program = &args[0];

        eprintln!("Uncorrect args");
        eprintln!("\tUsage: {program} topics");

        process::exit(1);
    }

    let topics = &args[1..];

    let mut connection = Connection::insecure_open("amqp://localhost:5672/")?;
    let channel = connection.open_channel(None)?;
    let exchange = channel.exchange_declare(ExchangeType::Topic, "topic_example", ExchangeDeclareOptions::default())?;
    let queue_ops = QueueDeclareOptions{
        durable: false,
        exclusive: true,
        auto_delete: true,
        arguments: FieldTable::new(),
    };
    let queue = channel.queue_declare("", queue_ops)?;

    for topic in topics {
        channel.queue_bind(queue.name(), exchange.name(), topic, FieldTable::new())?;
    }

    let consumer = queue.consume(ConsumerOptions::default())?;

    for msg in consumer.receiver() {
        match msg {
            ConsumerMessage::Delivery(delivery) => msg_callback(&consumer, delivery),
            other => {
                println!(" [x] Received unmached consumer message {{{other:#?}}}");
                break;
            },
        }
    }

    Ok(())
}