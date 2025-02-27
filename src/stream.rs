

use std::time::Duration;

use crate::t_types::market_data_stream_service_client::MarketDataStreamServiceClient;
use crate::t_types::*;
use crate::Api;
use async_channel::Sender;
use tokio::sync::broadcast::Sender as Broadcast;
use tokio::task::JoinHandle;
use log::{info, warn, error};


struct MarketCache {

}
impl MarketCache {
    fn process(&mut self, MarketDataRequest { payload }: MarketDataRequest) {
        use crate::t_types::market_data_request::Payload;
        if payload.is_none() {return};
        
    }
}

pub struct StreamHolder<Req, Res> {
    sender: Sender<Req>,
    broadcast: Broadcast<Res>,
    api: Api,
    handle: JoinHandle<()>,
}

impl<Req, Res> Drop for StreamHolder<Req, Res> {
    fn drop(&mut self) {
        self.handle.abort();
    }
}

impl StreamHolder<MarketDataRequest, MarketDataResponse> {
    pub async fn create(api: Api, broadcast: Broadcast<MarketDataResponse>) -> Result<Self, tonic::Status> {
        use crate::t_types::market_data_request::Payload;
        let (sender,r) = async_channel::unbounded();
        let timeout = Duration::from_secs(5);
        let req = MarketDataRequest { payload: Some(Payload::Ping(Default::default())) };
        sender.send(req.clone()).await.unwrap();

        let mut client = MarketDataStreamServiceClient::from(api.clone());
        let mut receiver = client.market_data_stream(r.clone()).await?.into_inner();
        let inner = (sender.clone(), broadcast.clone());
        let handle = tokio::spawn(async move {
            let (sender, broadcast) = inner;
            loop {
                match tokio::time::timeout(timeout, receiver.message()).await {
                    Ok(Ok(Some(response))) => {
                        if broadcast.send(response).is_err() { break; }
                        continue;
                    },
                    Ok(Ok(None)) => {
                        info!("none received, reconnecting");
                    },
                    Ok(Err(e)) => log::error!("err {e:?}, reconnecting..."),
                    Err(_) => {
                        info!("timeout, send ping...");
                        if r.is_empty() {
                            sender.send(req.clone()).await.unwrap();
                        }
                        continue;
                    }
                }
    
                info!("recv count: {}", r.receiver_count());
                while r.is_empty() {
                    sender.send(req.clone()).await.unwrap();
                }
                
                match tokio::time::timeout(timeout, client.market_data_stream(r.clone())).await {
                    Ok(Ok(x)) => receiver = x.into_inner(),
                    Ok(Err(x)) => {
                        error!("err on reconnect: {x:?}");
                        tokio::time::sleep(timeout).await;
                    },
                    Err(_) => warn!("timeout on reconnect"),
                }
            };
        });
        Ok(Self {api, sender, broadcast, handle})
    }
    pub async fn send(&self, req: MarketDataRequest) {
        self.sender.send(req).await.unwrap();
    }
    pub fn stop(&self) {
        self.handle.abort();
    }
    pub fn subscribe(&self) -> Broadcast<MarketDataResponse> {
        self.broadcast.clone()
    }
}


impl Clone for StreamHolder<MarketDataRequest, MarketDataResponse> {
    fn clone(&self) -> Self {
        todo!()
    }
}