use amiquip::{
    Connection,
    ExchangeType,
    ExchangeDeclareOptions,
    Publish,
    Error,
};

use std::{
    env,
    process,
};

fn main() -> Result<(), Error> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        let program = &args[0];

        eprintln!("Uncorrect arguments");
        eprintln!("\tUsage: {program} topic");

        process::exit(1);
    }

    let topic = &args[1];

    let mut connection = Connection::insecure_open("amqp://localhost:5672/")?;
    let channel = connection.open_channel(None)?;
    let exchange = channel.exchange_declare(ExchangeType::Topic, "topic_example", ExchangeDeclareOptions::default())?;

    exchange.publish(Publish::new("Example msg to topic...".as_bytes(), topic))?;

    connection.close()?;

    Ok(())
}