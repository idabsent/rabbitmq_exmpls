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
    fn new(channel: &'a Channel, exchange: &str, route_key: &str) -> Result<FibonacciServerRPC<'a>, Error> {
        let exchange = channel.exchange_declare(ExchangeType::Direct, exchange, ExchangeDeclareOptions::default())?;

        let options = QueueDeclareOptions{
            exclusive: true,
            auto_delete: true,
            durable: false,
            arguments: FieldTable::new(),
        };

        let queue = channel.queue_declare("", options)?;

        channel.queue_bind(queue.name(), exchange.name(), route_key, FieldTable::new())?;

        let exchange = Exchange::direct(&channel);

        Ok(
                Self {
                    exchange,
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

        let props = &delivery.properties;
        let reply_to = match props.reply_to() {
            Some(reply_to) => reply_to.clone(),
            None => return Ok(()),
        };

        let correlation_id = match props.correlation_id() {
            Some(correlation_id) => correlation_id.clone(),
            None => return Ok(()),
        };

        let props = AmqpProperties::default()
            .with_correlation_id(correlation_id);

        let fib = fibonacci(recv_numb).to_string();

        consumer.ack(delivery)?;

        self.exchange.publish(Publish::with_properties(fib.as_bytes(), reply_to, props))
    }
}

fn main() -> Result<(), Error> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        let program = &args[0];

        eprintln!("Uncorrect args");
        eprintln!("\tUsage: {program} exchange route_key");

        process::exit(1);
    }

    let exchange = &args[1];
    let route_key = &args[2];

    let mut connection = Connection::insecure_open("amqp://localhost:5672/")?;
    let channel = connection.open_channel(None)?;

    let server = FibonacciServerRPC::new(&channel, &exchange, &route_key)?;
    server.start_consume()
}