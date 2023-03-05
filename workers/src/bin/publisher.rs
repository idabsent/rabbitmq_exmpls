use amiquip::{
    Connection,
    Publish,
    Exchange,
    Error,
};

use std::{
    process,
    env,
};

fn main() -> Result<(), Error> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Uncorrect usage");
        eprintln!("\tUsage: {} point_count", args[0]);
        process::exit(1);
    }

    let point_count: usize = args[1].trim().parse().unwrap();
    let msg = format!("Hello msg{}", String::from(".").repeat(point_count));

    let mut connection = Connection::insecure_open("amqp://localhost:5672")?;
    let channel = connection.open_channel(None)?;
    let exchange = Exchange::direct(&channel);

    exchange.publish(Publish::new(msg.as_bytes(), "workers_queue"))?;

    connection.close()?;

    Ok(())
}