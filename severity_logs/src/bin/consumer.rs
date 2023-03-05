use amiquip::{
    Connection,
    ExchangeDeclareOptions,
    ExchangeType,
    QueueDeclareOptions,
    FieldTable,
    Delivery,
    Consumer,
    ConsumerMessage,
    ConsumerOptions,
    Error,
};

use std::{
    str,
    env,
    process,
};

use severity_logs::SUPPORTED_SEVERITIES;

fn msg_callback(consumer: &Consumer, delivery: Delivery) {
    let msg = str::from_utf8(&delivery.body).unwrap();

    println!(" [x] Log received {{{msg}}}");

    consumer.ack(delivery).unwrap();
}

pub fn show_usage(program_name: &str) {
    let mut usage_msg = String::from(program_name);
    for severity in SUPPORTED_SEVERITIES {
        usage_msg.push_str(&format!(" {severity} "));
    }

    eprintln!("Uncorrect arguments");
    eprintln!("\tUsage: {usage_msg}");

    process::exit(1);
}

fn main() -> Result<(), Error> {
    let args: Vec<String> = env::args().collect();

    if args.len() == 1 {
        show_usage(&args[0]);
    }

    let severities = &args[1..];
    for severity in severities {
        if !SUPPORTED_SEVERITIES.contains(&severity.as_str()) {
            show_usage(&args[0]);
        }
    }

    let mut connection = Connection::insecure_open("amqp://localhost:5672/")?;
    let channel = connection.open_channel(None)?;
    let exchange = channel.exchange_declare(ExchangeType::Direct, "severity_logs", ExchangeDeclareOptions::default())?;

    let queue_ops = QueueDeclareOptions{
        auto_delete: true,
        exclusive: true,
        durable: false,
        arguments: FieldTable::new(),
    };
    let queue = channel.queue_declare("", queue_ops)?;

    for severity in severities {
        channel.queue_bind(queue.name(), exchange.name(), severity, FieldTable::new())?;
    }

    let consumer = queue.consume(ConsumerOptions::default())?;
    for msg in consumer.receiver() {
        match msg {
            ConsumerMessage::Delivery(delivery) => msg_callback(&consumer, delivery),
            other => {
                println!(" [x] Received unmatched consumer message {other:#?}");
                break;
            }
        }
    }

    Ok(())
}