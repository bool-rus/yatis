use std::{collections::HashSet, sync::{Arc, Mutex}, time::Duration};

use async_channel::Sender;
use tokio::task::JoinHandle;
use futures::SinkExt;

use crate::{t_types::{market_data_stream_service_client::MarketDataStreamServiceClient, *}, Api};
use log::{info, warn, error};


#[derive(Debug)]
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
            Payload::SubscribeCandlesRequest(subscribe_candles_request) =>{},
            Payload::SubscribeOrderBookRequest(subscribe_order_book_request) => {},
            Payload::SubscribeTradesRequest(subscribe_trades_request) => {},
            Payload::SubscribeInfoRequest(subscribe_info_request) => {},
            Payload::SubscribeLastPriceRequest(SubscribeLastPriceRequest{ subscription_action, instruments }) => {
                match SubscriptionAction::try_from(*subscription_action).unwrap_or_default() {
                    SubscriptionAction::Subscribe => self.add_instruments(&instruments),
                    SubscriptionAction::Unsubscribe => self.remove_instruments(&instruments),
                    SubscriptionAction::Unspecified => {log::warn!("unspecified SubscriptionAction")},
                }
            },
            Payload::GetMySubscriptions(get_my_subscriptions) => {},
            Payload::Ping(ping_request) => {},
            Payload::PingSettings(ping_delay_settings) => {},
        }
    }
}

#[derive(Debug)]
pub struct StreamHolder<Req> {
    sender: Sender<Req>,
    cache: Arc<Mutex<MarketCache>>,
    api: Api,
    handle: JoinHandle<()>,
}

impl<Req> Drop for StreamHolder<Req> {
    fn drop(&mut self) {
        self.handle.abort();
    }
}
impl StreamHolder<MarketDataRequest>  {
    pub async fn create<Res, S>(api: Api, mut broadcast: S) -> Result<Self, tonic::Status> where S: futures::Sink<Res> + Unpin + Send + 'static, Res: From<MarketDataResponse> + Send + 'static {
    //pub async fn create<Res>(api: Api, Bcast(broadcast): Bcast<Res>) -> Result<Self, tonic::Status> where Res: From<MarketDataResponse> + Send + 'static {
        use crate::t_types::market_data_request::Payload;
        let timeout = Duration::from_secs(5);
        let req = MarketDataRequest { payload: Some(Payload::Ping(Default::default())) };
        let (sender,r) = async_channel::unbounded();
        sender.send(req.clone()).await.unwrap();
        
        let mut client = MarketDataStreamServiceClient::from(api.clone());
        let mut receiver = client.market_data_stream(r.clone()).await?.into_inner();
        let cache = Arc::new(Mutex::new(MarketCache::new()));
        let inner = (sender.clone(), cache.clone());
        let handle = tokio::spawn(async move {
            let (sender, cache) = inner;
            loop {
                match tokio::time::timeout(timeout, receiver.message()).await {
                    Ok(Ok(Some(response))) => {
                        if broadcast.send(response.into()).await.is_err() { break; }
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
            log::info!("market bi-directional stream stopped!");
        });
        Ok(Self {api, cache, sender, handle})
    }
    pub async fn send(&self, req: MarketDataRequest) {
        self.cache.lock().unwrap().process(&req);
        self.sender.send(req).await.unwrap();
    }
    pub fn stop(&self) {
        self.handle.abort();
    }
}
