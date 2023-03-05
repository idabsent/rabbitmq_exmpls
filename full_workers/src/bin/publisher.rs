use amiquip::{
    Connection,
    Exchange,
    Publish,
    AmqpProperties,
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

        eprintln!("Uncorrect usage");
        eprintln!("\tUsage: {program} point_count");

        process::exit(1);
    }

    let point_count: usize = args[1].trim().parse().unwrap();
    let msg = format!("Hello msg{}", ".".repeat(point_count));

    let mut connection = Connection::insecure_open("amqp://localhost:5672/")?;
    let channel = connection.open_channel(None)?;
    let exchange = Exchange::direct(&channel);
    let pub_props = AmqpProperties::default()
        .with_delivery_mode(2);

    exchange.publish(Publish::with_properties(msg.as_bytes(), "full_workers_queue", pub_props))?;

    connection.close()?;

    Ok(())
}