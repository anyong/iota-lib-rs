// Copyright 2021-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use iota_client::{
    bee_message::MessageDto,
    message_interface::{
        create_message_handler, ClientMessageHandler, Message as ClientMessage, MessageType, Response, ResponseType,
    },
    MqttPayload, Topic, TopicEvent,
};

use neon::prelude::*;
use serde::Serialize;
use tokio::sync::mpsc::unbounded_channel;

use std::sync::Arc;

type JsCallback = Root<JsFunction<JsObject>>;

pub struct MessageHandler {
    channel: Channel,
    client_message_handler: ClientMessageHandler,
}

impl Finalize for MessageHandler {}
impl MessageHandler {
    fn new(channel: Channel, options: String) -> Arc<Self> {
        let client_message_handler = crate::RUNTIME
            .block_on(async move { create_message_handler(Some(options)).await })
            .expect("error initializing account manager");

        Arc::new(Self {
            channel,
            client_message_handler,
        })
    }

    async fn send_message(&self, serialized_message: String) -> (String, bool) {
        log::debug!("{}", serialized_message);
        match serde_json::from_str::<MessageType>(&serialized_message) {
            Ok(message) => {
                let (response_tx, mut response_rx) = unbounded_channel();
                let client_message = ClientMessage::new(message.clone(), response_tx);

                self.client_message_handler.handle(client_message).await;
                let response = response_rx.recv().await;
                if let Some(res) = response {
                    let mut is_err = matches!(res.response_type(), ResponseType::Error(_) | ResponseType::Panic(_));

                    let msg = match serde_json::to_string(&res) {
                        Ok(msg) => msg,
                        Err(e) => {
                            is_err = true;
                            serde_json::to_string(&Response::new(message, ResponseType::Error(e.into())))
                                .expect("The response is generated manually, so unwrap is safe.")
                        }
                    };

                    (msg, is_err)
                } else {
                    ("No response".to_string(), true)
                }
            }
            Err(e) => {
                log::debug!("{:?}", e);
                (format!("Couldn't parse to message with error - {:?}", e), true)
            }
        }
    }
    fn call_event_callback(&self, event: TopicEvent, callback: Arc<JsCallback>) {
        self.channel.send(move |mut cx| {
            #[derive(Serialize)]
            struct MqttResponse {
                topic: String,
                payload: String,
            }
            let payload = match &event.payload {
                MqttPayload::Json(val) => serde_json::to_string(&val).unwrap(),
                MqttPayload::Message(msg) => serde_json::to_string(&MessageDto::from(msg)).unwrap(),
            };
            let response = MqttResponse {
                topic: event.topic,
                payload,
            };
            let cb = (*callback).to_inner(&mut cx);
            let this = cx.undefined();
            let args = vec![
                cx.undefined().upcast::<JsValue>(),
                cx.string(serde_json::to_string(&response).unwrap()).upcast::<JsValue>(),
            ];

            cb.call(&mut cx, this, args)?;

            Ok(())
        });
    }
}

pub fn message_handler_new(mut cx: FunctionContext) -> JsResult<JsBox<Arc<MessageHandler>>> {
    let options = cx.argument::<JsString>(0)?;
    let options = options.value(&mut cx);
    let channel = cx.channel();
    let message_handler = MessageHandler::new(channel, options);

    Ok(cx.boxed(message_handler))
}

pub fn send_message(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let message = cx.argument::<JsString>(0)?;
    let message = message.value(&mut cx);
    let message_handler = Arc::clone(&&cx.argument::<JsBox<Arc<MessageHandler>>>(1)?);
    let callback = cx.argument::<JsFunction>(2)?.root(&mut cx);

    crate::RUNTIME.spawn(async move {
        let (response, is_error) = message_handler.send_message(message).await;
        log::debug!("{:?}", response);
        message_handler.channel.send(move |mut cx| {
            let cb = callback.into_inner(&mut cx);
            let this = cx.undefined();

            let args = vec![
                if is_error {
                    cx.string(response.clone()).upcast::<JsValue>()
                } else {
                    cx.undefined().upcast::<JsValue>()
                },
                cx.string(response).upcast::<JsValue>(),
            ];

            cb.call(&mut cx, this, args)?;

            Ok(())
        });
    });

    Ok(cx.undefined())
}

// MQTT
pub fn listen(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let js_arr_handle: Handle<JsArray> = cx.argument(0)?;
    let vec: Vec<Handle<JsValue>> = js_arr_handle.to_vec(&mut cx)?;
    let mut topics = vec![];
    for topic_string in vec {
        let topic = topic_string.downcast::<JsString, FunctionContext>(&mut cx).unwrap();
        topics.push(Topic::try_from(topic.value(&mut cx).as_str().to_string()).expect("Invalid MQTT topic"));
    }

    let callback = Arc::new(cx.argument::<JsFunction>(1)?.root(&mut cx));
    let message_handler = Arc::clone(&&cx.argument::<JsBox<Arc<MessageHandler>>>(2)?);

    crate::RUNTIME.spawn(async move {
        let cloned_message_handler = message_handler.clone();
        let mut cloned_client = message_handler.client_message_handler.client.clone();
        cloned_client
            .subscriber()
            .with_topics(topics)
            .subscribe(move |event_data| {
                cloned_message_handler.call_event_callback(event_data.clone(), callback.clone())
            })
            .await
            .unwrap();
    });

    Ok(cx.undefined())
}