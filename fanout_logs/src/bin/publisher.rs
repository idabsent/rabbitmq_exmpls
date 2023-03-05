use amiquip::{
    Connection,
    Publish,
    ExchangeType,
    ExchangeDeclareOptions,
    Error,
};

fn main() -> Result<(), Error> {
    let mut connection = Connection::insecure_open("amqp://localhost:5672/")?;
    let channel = connection.open_channel(None)?;
    let exchange = channel.exchange_declare(ExchangeType::Fanout, "logs", ExchangeDeclareOptions::default())?;

    exchange.publish(Publish::new("New log msg...".as_bytes(), ""))?;

    connection.close()?;

    Ok(())
}