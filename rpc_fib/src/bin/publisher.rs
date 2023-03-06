use amiquip::{
    Connection,
    AmqpProperties,
    Channel,
    Exchange,
    ExchangeType,
    ExchangeDeclareOptions,
    Consumer,
    ConsumerOptions,
    ConsumerMessage,
    Queue,
    QueueDeclareOptions,
    Publish,
    Error,
    FieldTable,
};

use std::{
    env,
    process,
    str,
    time,
};

use uuid::Uuid;

struct FibonacciClientRPC<'a> {
    exchange: Exchange<'a>,
    route_key: &'a str,
    consumer: Consumer<'a>,
    queue: Queue<'a>,
}

impl<'a> FibonacciClientRPC<'a> {
    fn new(channel: &'a Channel, exchange: &str, route_key: &'a str, local_route: &str) -> Result<FibonacciClientRPC<'a>, Error> {
        let exchange = channel.exchange_declare(ExchangeType::Direct, exchange, ExchangeDeclareOptions::default())?;
        let queue_ops = QueueDeclareOptions {
            durable: false,
            auto_delete: true,
            exclusive: true,
            arguments: FieldTable::new(),
        };

        let queue = channel.queue_declare(local_route, queue_ops)?;

        let consumer_options = ConsumerOptions{
            no_local: true,
            no_ack: false,
            exclusive: false,
            arguments: FieldTable::new(),
        };
        let consumer = queue.consume(consumer_options)?;

        Ok(
                Self {
                    exchange,
                    queue,
                    consumer,
                    route_key,
                }
        )
    }

    fn call<T>(&self, num: T) -> Result<u64, Error>
    where T: Into<u64>
    {
        let local_correlation_id = Uuid::new_v4().to_string();
        let correlation_id = local_correlation_id.clone();
        let reply_to = String::from(self.queue.name());

        let msg_props = AmqpProperties::default()
            .with_correlation_id(correlation_id)
            .with_reply_to(reply_to);

        let num: u64 = num.into();
        let msg = num.to_string();

        self.exchange.publish(Publish::with_properties(msg.as_bytes(), self.route_key, msg_props))?;

        let msg = match self.consumer.receiver().recv_timeout(time::Duration::from_secs(2)) {
            Ok(msg) => msg,
            Err(_) => {
                eprintln!(" [x] Timeout of get message from server. Try again...");
                process::exit(1);
            },
        };

        let num = match msg {
            ConsumerMessage::Delivery(delivery) => {
                if let Some(correlation_id) = delivery.properties.correlation_id() {
                    if correlation_id != &local_correlation_id {
                        eprintln!(" [x] Received uncorrect correlation_id {{correlation_id: {correlation_id}}} {{local_correlation_id: {local_correlation_id}}}");
                        process::exit(1);
                    }
                    
                    let msg = str::from_utf8(&delivery.body).unwrap().to_string();
                    self.consumer.ack(delivery)?;

                    msg.trim().parse().unwrap()
                } else {
                    eprintln!(" [x] Received msg without correlation_id");
                    process::exit(1);
                }
            },
            other => {
                eprintln!(" [x] Received unmatched message {{{other:#?}}} from server... Exit");
                process::exit(1);
            }
        };

        Ok(num)
    }
}

fn show_usage(program: &str) -> ! {
    eprintln!("Uncorrect arguments");
    eprintln!("\tUsage: {program} exchange route_key local_route num");
    process::exit(1);
}

fn main() -> Result<(), Error> {
    let args: Vec<String> = env::args().collect();

    let program_name = &args[0];

    if args.len() != 5 {
        show_usage(&program_name);
    }

    let exchange = &args[1];
    let route_key = &args[2];
    let local_route = &args[3];

    let num: u64 = match args[4].trim().parse() {
        Ok(num) => num,
        Err(_) => show_usage(&program_name)
    };

    let mut connection = Connection::insecure_open("amqp://localhost:5672/")?;
    let channel = connection.open_channel(None)?;

    let client = FibonacciClientRPC::new(&channel, &exchange, &route_key, &local_route)?;

    let recv_num = client.call(num)?;

    println!(" [x] fibonacci({num}) = {recv_num}");

    Ok(())
}