use amqprs::{
    callbacks::{DefaultChannelCallback, DefaultConnectionCallback},
    channel::{
        BasicAckArguments, BasicCancelArguments, BasicConsumeArguments, Channel,
        QueueBindArguments, QueueDeclareArguments,
    },
    connection::{Connection, OpenConnectionArguments},
};
use std::time;
use uuid::Uuid;

async fn connect_rabbitmq(connection_details: &RabbitConnect) -> Connection {
    //this is for demo and teaching purposes, you would fetch this information from a config of course
    let mut res = Connection::open(&OpenConnectionArguments::new(
        &connection_details.host,
        connection_details.port,
        &connection_details.username,
        &connection_details.password,
    ))
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

async fn bind_queue_to_exchange(
    connection: &mut amqprs::connection::Connection,
    channel: &mut Channel,
    connection_details: &RabbitConnect,
    queue: &str,
) {
    if !connection.is_open() {
        println!("Connection not open");
        *connection = connect_rabbitmq(connection_details).await;
        *channel = channel_rabbitmq(&connection).await;
        println!("{}", connection);
    }

    let qparams = QueueDeclareArguments::default()
        .queue(queue.to_owned())
        .auto_delete(true)
        .durable(false)
        .finish();

    let (queue, _, _) = channel.queue_declare(qparams).await.unwrap().unwrap();

    if !channel.is_open() {
        println!("channel is not open, does exchange systemmonitor exist on rabbitMQ?");
        *channel = channel_rabbitmq(&connection).await;
    }

    channel
        .queue_bind(QueueBindArguments::new(&queue, "systemmonitor", ""))
        .await
        .unwrap();
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

    // create a unique queue and bind it to the exchange systemmonitor
    let uuid = Uuid::new_v4();
    let queue = format!("c_systemmonitor_{}", uuid.as_hyphenated().to_string());
    let args = BasicConsumeArguments::new(&queue, format!("{} sub_monitor", queue).as_str());

    //this loop makes sure that on error we do a whole new reconnect and setup of the new queue and consumer/error structs
    //it will try to reconnect every two seconds in the connect_rabbitmq
    //when a connection is made it will create a new channel on that connection and bind the queue to the exchange and spawn an new worker
    //you can also put all connect logic in its own function and call it during setup and when the tokio join returns a Err option this would be a cleane approach
    //but for the sake of teaching the concept a bit convoluted, hence the loop without the actual Err action
    loop {
        let mut connection = connect_rabbitmq(&connection_details).await;
        let mut channel = channel_rabbitmq(&connection).await;

        bind_queue_to_exchange(&mut connection, &mut channel, &connection_details, &queue).await;

        let (ctag, mut messages_rx) = channel.basic_consume_rx(args.clone()).await.unwrap();

        //this is the actual worker logic, spawed with ansync tokio process
        let join = tokio::spawn(async move {
            while let Some(msg) = messages_rx.recv().await {
                let a = msg.content.unwrap();
                let s = String::from_utf8_lossy(&a);

                //call your own function and do something usefull and return Ok or Err and on Ok ack the message, this way you don't loose messages
                //this is assuming there are no symatic errors in the message in that case when the message needs to be discarded also call ack.
                //but that is up to your functional error handling
                println!("{}", s);

                let args = BasicAckArguments::new(msg.deliver.unwrap().delivery_tag(), false);
                let _ = channel.basic_ack(args).await;
            }

            // this is what to do when we get a nerror
            if let Err(e) = channel.basic_cancel(BasicCancelArguments::new(&ctag)).await {
                println!("error {}", e.to_string());
            };
        }); // we just await here otherwise we keep looping for no good reason.

        let _ = join.await;
    }
}
