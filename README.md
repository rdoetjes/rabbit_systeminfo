# rabbit_systeminfo

Is a demo as to how publish and consume data to and from RabbitMQ with error correcting behaviour.<br/>
When the RabbitMQ server goes down, the publisher and consumer will go into a 2 second retry mode (blocking behavior) until the server comes back online
and they will restart their job.
Also the consumer, will show how to ack a message only after succesful processing and not automatic ack when message is send to the consumer callback -- which seems to be the default behaviour in all examples.
However the coal of a MQ is to only ack a message when it has been processed successfully, as to not loose any unprocessed events.

The little demo project merely serves as a more detailed supplement to the amqprs crate (sparse) documentation.<br/>
And we are using collecting of systeminfo (memory and cpu usage) as the information. It changes frequently so it is very visible and you can even poll it very fast by setting the publisher time out form 1000ms to 1ms. Which was done to test high speed delivery.

# What is demonstarted?
This code will demonstate the following often required options
- Connecting to amqp
- Creating a queue
- Binding a queue to an exchange
- Publishing messages
- Consuming message
- Acknowledging and Not-Acknowledging messages
- Setting up a dead letter exchange to a queue (used to route messages that are Not-Acknowledge to a dead letter exchange when present)
- Recovering both publisher and consumer from RabbitMQ restart

# Authentication authorisation quirck
You will need to add the user *consumer* to your RabbitMQ with in this code's case the password *crabs* and granted access to the default virtual host /.<br/>
As you can see in the code we use the user consumer with password crabs and not the default *guest* user.<br/>
The reason for this is that the *guest*  user in RabbitMQ is by default not allowed to connect over the network!!! Only over a loopback interface. So if you were to use this code for your learning experience and change localhost to anything else, it would give you an error.

I urge you to read the whole RabbitMQ authentication and security documentation!

# RabbitMQ configuration
I have provided the bare configuration to run this demo in the file called: rabbit_server_config.json </br>
You can import this into RabbitMQ. The password for the quest user account is herpies, the password for the consumer user account is crabs.
What this config does is sets up the *absolutely required* systemmonitor exchange to which the consumer's queues will be bound. It will also setup an exchange called all.deadletter and a queue all.deadletter that will receiver the messages that were Nacked (not acknowledged).