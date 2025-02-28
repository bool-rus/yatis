

use std::collections::HashSet;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

use crate::t_types::market_data_stream_service_client::MarketDataStreamServiceClient;
use crate::t_types::*;
use crate::Api;
use async_channel::Sender;
use tokio::sync::broadcast::Sender as Broadcast;
use tokio::task::JoinHandle;
use log::{info, warn, error};


struct MarketCache {
    lastprice: HashSet<String>,
}
impl MarketCache {
    fn new() -> Self {
        Self{lastprice: HashSet::new()}
    }
    fn market_req(&self) -> MarketDataRequest {
        let instruments = self.lastprice.iter().map(|id|LastPriceInstrument{instrument_id: id.clone(), ..Default::default()}).collect();
        MarketDataRequest {payload:Some(crate::t_types::market_data_request::Payload::SubscribeLastPriceRequest(
            SubscribeLastPriceRequest {
                subscription_action: SubscriptionAction::Subscribe.into(),
                instruments
            }
        ))}
    }
    fn add_instruments(&mut self, inst: &[LastPriceInstrument]) {
        let mut changed = false;
        inst.iter().for_each(|LastPriceInstrument { figi, instrument_id }| {
            self.lastprice.insert(instrument_id.clone());
    });
    }
    fn remove_instruments(&mut self, inst: &[LastPriceInstrument]) {
        inst.iter().for_each(|LastPriceInstrument { figi, instrument_id }|{
            self.lastprice.remove(instrument_id);
        });
    }
    fn process(&mut self, req: &MarketDataRequest) {
        use crate::t_types::market_data_request::Payload;
        if req.payload.is_none() {return};
        match req.payload.as_ref().unwrap() {
            Payload::SubscribeCandlesRequest(subscribe_candles_request) => todo!(),
            Payload::SubscribeOrderBookRequest(subscribe_order_book_request) => todo!(),
            Payload::SubscribeTradesRequest(subscribe_trades_request) => todo!(),
            Payload::SubscribeInfoRequest(subscribe_info_request) => {},
            Payload::SubscribeLastPriceRequest(SubscribeLastPriceRequest{ subscription_action, instruments }) => {
                match SubscriptionAction::try_from(*subscription_action).unwrap_or_default() {
                    SubscriptionAction::Subscribe => self.add_instruments(&instruments),
                    SubscriptionAction::Unsubscribe => self.remove_instruments(&instruments),
                    SubscriptionAction::Unspecified => {log::warn!("unspecified SubscriptionAction")},
                }
            },
            Payload::GetMySubscriptions(get_my_subscriptions) => todo!(),
            Payload::Ping(ping_request) => todo!(),
            Payload::PingSettings(ping_delay_settings) => todo!(),
        }
    }
}

pub struct StreamHolder<Req, Res> {
    sender: Sender<Req>,
    cache: Arc<Mutex<MarketCache>>,
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
        let cache = Arc::new(Mutex::new(MarketCache::new()));
        use crate::t_types::market_data_request::Payload;
        let (sender,r) = async_channel::unbounded();
        let timeout = Duration::from_secs(5);
        let req = MarketDataRequest { payload: Some(Payload::Ping(Default::default())) };
        sender.send(req.clone()).await.unwrap();

        let mut client = MarketDataStreamServiceClient::from(api.clone());
        let mut receiver = client.market_data_stream(r.clone()).await?.into_inner();
        let inner = (sender.clone(), broadcast.clone(), cache.clone());
        let handle = tokio::spawn(async move {
            let (sender, broadcast, cache) = inner;
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
                    Ok(Ok(x)) => {
                        receiver = x.into_inner();
                        let req = cache.lock().unwrap().market_req();
                        sender.send(req).await.unwrap();
                    }
                    Ok(Err(x)) => {
                        error!("err on reconnect: {x:?}");
                        tokio::time::sleep(timeout).await;
                    },
                    Err(_) => warn!("timeout on reconnect"),
                }
            };
        });
        Ok(Self {api, cache, sender, broadcast, handle})
    }
    pub async fn send(&self, req: MarketDataRequest) {
        self.cache.lock().unwrap().process(&req);
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