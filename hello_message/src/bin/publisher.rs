use amiquip::{
    Exchange,
    Error,
    Publish,
    Connection,
};

fn main() -> Result<(), Error> {
    let mut connection = Connection::insecure_open("amqp://localhost:5672/")?;
    let channel = connection.open_channel(None)?;
    let exchange = Exchange::direct(&channel);

    let message = "Hello fucing bich".as_bytes();

    exchange.publish(Publish::new(message, "hello_message"))?;

    connection.close()?;

    Ok(())
}
