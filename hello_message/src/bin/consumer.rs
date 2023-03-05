use amiquip::{
    Error,
    Connection,
    QueueDeclareOptions,
    ConsumerOptions,
    ConsumerMessage,
    FieldTable,
};

fn main() -> Result<(), Error> {
    let mut connection = Connection::insecure_open("amqp://localhost:5672")?;
    let channel = connection.open_channel(None)?;
    let queue = channel.queue_declare("hello_message", QueueDeclareOptions::default())?;
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
                println!(" [x] Received message {{{}}}", String::from_utf8(delivery.body).unwrap());
            },
            other => {
                println!(" [x] Received unmatched message {other:?}");
                break;
            },
        }
    }

    connection.close()?;

    Ok(())
}