use amiquip::{
    Connection,
    AmqpProperties,
    Channel,
    Exchange,
    ExchangeType,
    ExchangeDeclareOptions,
    Queue,
    QueueDeclareOptions,
    Consumer,
    ConsumerMessage,
    ConsumerOptions,
    Delivery,
    Error,
    FieldTable,
    Publish,
};

use std::{
    str,
    process,
    env,
};

struct FibonacciServerRPC<'a> {
    exchange: Exchange<'a>,
    connection: Connection,
    channel: Channel,
    queue: Queue<'a>,
}

fn fibonacci(num: i64) -> i64 {
    if num == 1 {
        return 1;
    };

    if num == 0 {
        return 0;
    }

    fibonacci(num - 2) + fibonacci(num - 1)
}

impl<'a> FibonacciServerRPC<'a> {
    fn new(uri: &str, exchange: &str) -> Result<FibonacciServerRPC<'a>, Error> {
        let mut connection = Connection::insecure_open(uri)?;
        let channel = connection.open_channel(None)?;

        let exchange = channel.exchange_declare(ExchangeType::Direct, exchange, ExchangeDeclareOptions::default())?;

        let options = QueueDeclareOptions{
            exclusive: true,
            auto_delete: true,
            durable: false,
            arguments: FieldTable::new(),
        };

        let queue = channel.queue_declare("", options)?;

        channel.queue_bind(queue.name(), exchange.name(), "", FieldTable::new())?;

        let exchange = Exchange::direct(&channel);

        Ok(
                Self {
                    exchange,
                    connection,
                    channel,
                    queue,
                }
        )
    }

    fn start_consume(&self) -> Result<(), Error> {
        let consumer = self.queue.consume(ConsumerOptions::default())?;

        for msg in consumer.receiver() {
            match msg {
                ConsumerMessage::Delivery(delivery) => self.parse_delivery(&consumer, delivery)?,
                other => {
                    println!(" [x] Received unmatched consumer message {{{other:#?}}}");
                    break;
                },
            }
        }

        Ok(())
    }

    fn parse_delivery(&self, consumer: &Consumer, delivery: Delivery) -> Result<(), Error> {
        let msg = match str::from_utf8(&delivery.body) {
            Ok(msg) => msg,
            Err(_) => return Ok(()),
        };

        let recv_numb: i64 = match msg.trim().parse() {
            Ok(num) => num,
            Err(_) => return Ok(()),
        };

        let props = delivery.properties;
        let reply_to = match props.reply_to() {
            Some(reply_to) => reply_to.to_string(),
            None => return Ok(()),
        };

        let correlation_id = match props.correlation_id() {
            Some(correlation_id) => correlation_id.to_string(),
            None => return Ok(()),
        };

        let props = AmqpProperties::default()
            .with_correlation_id(correlation_id);

        let fib = fibonacci(recv_numb).to_string();

        self.exchange.publish(Publish::with_properties(fib.as_bytes(), reply_to, props))
    }
}

fn main() -> Result<(), Error> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        let program = &args[0];

        eprintln!("Uncorrect args");
        eprintln!("\tUsage: {program} exchange");

        process::exit(1);
    }

    let exchange = &args[1];

    let server = FibonacciServerRPC::new("amqp://localhost:5672/", &exchange)?;
    server.start_consume()
}