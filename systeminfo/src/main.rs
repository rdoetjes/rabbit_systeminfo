use ::sysinfo::{CpuExt, System, SystemExt};
use amqprs::DELIVERY_MODE_PERSISTENT;
use amqprs::{
    callbacks::{DefaultChannelCallback, DefaultConnectionCallback},
    channel::{BasicPublishArguments, Channel},
    connection::{Connection, OpenConnectionArguments},
    BasicProperties,
};
use serde::Serialize;
use serde_json;
use std::{thread, time};

#[derive(Default, Debug, Clone, Serialize)]
struct SystemInfo {
    tot_memory: u64,
    used_memory: u64,
    tot_swap: u64,
    used_swap: u64,
    cpu_util: Vec<f32>,
}

impl SystemInfo {
    fn default() -> SystemInfo {
        let mut sys = System::new_all();
        // First we update all information of our `System` struct.
        sys.refresh_cpu();
        sys.refresh_memory();

        SystemInfo {
            cpu_util: Vec::with_capacity(sys.cpus().len()),
            ..Default::default()
        }
    }
}

async fn connect_rabbitmq(connection_details: &RabbitConnect) -> Connection {
    let mut res = Connection::open(
        &OpenConnectionArguments::new(
            &connection_details.host,
            connection_details.port,
            &connection_details.username,
            &connection_details.password,
        )
        .virtual_host("/"),
    )
    .await;

    while res.is_err() {
        println!("trying to connect after error");
        std::thread::sleep(time::Duration::from_millis(2000));
        res = Connection::open(&OpenConnectionArguments::new(
            &connection_details.host,
            connection_details.port,
            &connection_details.username,
            &connection_details.password,
        ))
        .await;
    }

    let connection = res.unwrap();
    connection
        .register_callback(DefaultConnectionCallback)
        .await
        .unwrap();
    connection
}

async fn channel_rabbitmq(connection: &amqprs::connection::Connection) -> Channel {
    let channel = connection.open_channel(None).await.unwrap();
    channel
        .register_callback(DefaultChannelCallback)
        .await
        .unwrap();
    return channel;
}

async fn send(
    connection: &mut amqprs::connection::Connection,
    channel: &mut Channel,
    connection_details: &RabbitConnect,
    exchange: &str,
    result: &str,
) {
    if !connection.is_open() {
        println!("Connection not open");
        *connection = connect_rabbitmq(connection_details).await;
        *channel = channel_rabbitmq(&connection).await;
        println!("{}", connection);
    }

    if !channel.is_open() {
        println!("channel is not open, does exchange systemmonitor exist on rabbitMQ?");
        *channel = channel_rabbitmq(&connection).await;
    } else {
        let args = BasicPublishArguments::new(exchange, "");
        channel
            .basic_publish(
                BasicProperties::default()
                    .with_delivery_mode(DELIVERY_MODE_PERSISTENT)
                    .finish(),
                result.into(),
                args,
            )
            .await
            .unwrap();
    }
}

async fn get_sys_info(
    connection: &mut amqprs::connection::Connection,
    channel: &mut Channel,
    connection_details: &RabbitConnect,
    sys: &mut System,
    details: &mut SystemInfo,
) {
    sys.refresh_cpu();
    sys.refresh_memory();

    details.tot_memory = sys.total_memory();
    details.used_memory = sys.used_memory();
    details.tot_swap = sys.total_swap();
    details.used_swap = sys.used_swap();

    details.cpu_util.clear();
    for cpu in sys.cpus() {
        details.cpu_util.push(cpu.cpu_usage());
    }

    let result = serde_json::to_string(&details.to_owned())
        .expect("{}")
        .to_string();
    send(connection, channel, connection_details, "systemmonitor", &result).await;
}

struct RabbitConnect {
    host: String,
    port: u16,
    username: String,
    password: String,
}

#[tokio::main]
async fn main() {
    let connection_details = RabbitConnect {
        host: "localhost".to_string(),
        port: 5672,
        username: "consumer".to_string(),
        password: "crabs".to_string(),
    };
    let mut sys = System::new_all();
    let mut details = SystemInfo::default();

    let mut connection = connect_rabbitmq(&connection_details).await;
    let mut channel = channel_rabbitmq(&connection).await;

    loop {
        get_sys_info(
            &mut connection,
            &mut channel,
            &connection_details,
            &mut sys,
            &mut details,
        )
        .await;
        thread::sleep(time::Duration::from_millis(1000));
    }
}
