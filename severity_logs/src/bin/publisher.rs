use amiquip::{
    Connection,
    ExchangeType,
    ExchangeDeclareOptions,
    Publish,
    Error,
};

use severity_logs::SUPPORTED_SEVERITIES;

use std::{
    env,
    process,
};

fn main() -> Result<(), Error> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        let mut usage_msg = format!("{} ", args[0]);

        for (i, &severity) in SUPPORTED_SEVERITIES.iter().enumerate() {
            let delim = if i == (SUPPORTED_SEVERITIES.len() - 1) { "" } else { " | " };
            usage_msg.push_str(&format!("{severity}{delim}"));
        }

        eprintln!("Missing log severity");
        eprintln!("\tUsage: {usage_msg}");
        process::exit(1);
    }

    let severity = &args[1];

    let mut connection = Connection::insecure_open("amqp://localhost:5672")?;
    let channel = connection.open_channel(None)?;
    let exchange = channel.exchange_declare(ExchangeType::Direct, "severity_logs", ExchangeDeclareOptions::default())?;

    exchange.publish(Publish::new(format!("[{severity}] Example msg...").as_bytes(), severity))?;

    Ok(())
}