

use std::collections::HashSet;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

use crate::t_types::market_data_stream_service_client::MarketDataStreamServiceClient;
use crate::t_types::operations_stream_service_client::OperationsStreamServiceClient;
use crate::t_types::orders_service_client::OrdersServiceClient;
use crate::t_types::orders_stream_service_client::OrdersStreamServiceClient;
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

impl<Res> StreamHolder<MarketDataRequest, Res> where Res: From<MarketDataResponse> + Send + 'static {
    pub async fn create(api: Api, broadcast: Broadcast<Res>) -> Result<Self, tonic::Status> {
        use crate::t_types::market_data_request::Payload;
        let timeout = Duration::from_secs(5);
        let req = MarketDataRequest { payload: Some(Payload::Ping(Default::default())) };
        let (sender,r) = async_channel::unbounded();
        sender.send(req.clone()).await.unwrap();
        
        let mut client = MarketDataStreamServiceClient::from(api.clone());
        let mut receiver = client.market_data_stream(r.clone()).await?.into_inner();
        let cache = Arc::new(Mutex::new(MarketCache::new()));
        let inner = (sender.clone(), broadcast.clone(), cache.clone());
        let handle = tokio::spawn(async move {
            let (sender, broadcast, cache) = inner;
            loop {
                match tokio::time::timeout(timeout, receiver.message()).await {
                    Ok(Ok(Some(response))) => {
                        if broadcast.send(response.into()).is_err() { break; }
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
    pub fn subscribe(&self) -> Broadcast<Res> {
        self.broadcast.clone()
    }
}

trait PingDelayMs {
    const NET_DELAY: u64 = 1000;
    const DEFULT_PING_DELAY: u64 = 5000;
    fn delay_ms(&self) -> u64 {
        let delay = if let Some(delay) = self.invoke() {
            delay as u64
        } else {
            Self::DEFULT_PING_DELAY
        };
        delay + Self::NET_DELAY
    }
    fn invoke(&self) -> std::option::Option<i32>;
}
impl PingDelayMs for TradesStreamRequest {
    fn invoke(&self) -> std::option::Option<i32> {
        self.ping_delay_ms
    }
}
impl PingDelayMs for OrderStateStreamRequest {
    fn invoke(&self) -> std::option::Option<i32> {
        self.ping_delay_millis
    }
}

macro_rules! ping_from_ping_settings {
    ($($res:ty,)+) => {$(
        impl PingDelayMs for $res {
            fn invoke(&self) -> std::option::Option<i32> {
                self.ping_settings.map(|s|s.ping_delay_ms).flatten()
            }
        }
    )+}
}
ping_from_ping_settings!(PortfolioStreamRequest, PositionsStreamRequest, MarketDataServerSideStreamRequest, );

pub trait StartStream<Req, T> {
    fn start_stream(self, req: Req, sender: Broadcast<T>) -> impl std::future::Future<Output=Result<JoinHandle<()>, tonic::Status>>;
}

macro_rules! start_stream_impl {
    ($($res:ty = $client:ident : $method:ident($req:ty),)+) => {$(
        impl<T> StartStream<$req, T> for Api where T: From<$res> + Send + 'static {
            fn start_stream(self, req: $req, broadcast: Broadcast<T>) -> impl std::future::Future<Output=Result<JoinHandle<()>, tonic::Status>> { Box::pin(async move {
                let timeout = Duration::from_millis(req.delay_ms());
                let mut client = $client::from(self.clone());
                let mut receiver = client.$method(req.clone()).await?.into_inner();
                let inner = broadcast.clone();
                let handle = tokio::spawn(async move {
                    let broadcast = inner;
                    loop {
                        match tokio::time::timeout(timeout, receiver.message()).await {
                            Ok(Ok(Some(response))) => {
                                if broadcast.send(response.into()).is_err() { break; }
                                continue;
                            },
                            Ok(Ok(None)) => info!("none received, reconnecting..."),
                            Ok(Err(e)) => log::error!("err {e:?}, reconnecting..."),
                            Err(_) => info!("timeout, reconnecting..."),
                        }
                        let mut client = $client::from(self.clone());    
                        let fut = client.$method(req.clone());
                        match tokio::time::timeout(timeout, fut).await {
                            Ok(Ok(x)) => {
                                receiver = x.into_inner();
                            }
                            Ok(Err(x)) => {
                                error!("err on reconnect: {x:?}");
                                tokio::time::sleep(timeout).await;
                            },
                            Err(_) => warn!("timeout on reconnect"),
                        }
                    };
                });
                Ok(handle)
            })}
        }
    )+}
}

start_stream_impl![
    TradesStreamResponse = OrdersStreamServiceClient:trades_stream(TradesStreamRequest),
    OrderStateStreamResponse = OrdersStreamServiceClient:order_state_stream(OrderStateStreamRequest),
    PortfolioStreamResponse = OperationsStreamServiceClient:portfolio_stream(PortfolioStreamRequest),
    PositionsStreamResponse = OperationsStreamServiceClient:positions_stream(PositionsStreamRequest),
    MarketDataResponse = MarketDataStreamServiceClient:market_data_server_side_stream(MarketDataServerSideStreamRequest),
];