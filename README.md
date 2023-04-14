# rabbit_systeminfo

Is a demo as to how publish and consume data to and from RabbitMQ with error correcting behaviour.<br/>
When the RabbitMQ server goes down, the publisher and consumer will go into a 2 second retry mode (blocking behavior) until the server comes back online
and they will restart their job.
Also the consumer, will show how to ack a message only after succesful processing and not automatic ack when message is send to the consumer callback -- which seems to be the default behaviour in all examples.
However the coal of a MQ is to only ack a message when it has been processed successfully, as to not loose any unprocessed events.

The little demo project merely serves as a more detailed supplement to the amqprs crate (sparse) documentation.
