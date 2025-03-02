use std::time::Duration;

use crate::t_types::market_data_stream_service_client::MarketDataStreamServiceClient;
use crate::t_types::operations_stream_service_client::OperationsStreamServiceClient;
use crate::t_types::orders_stream_service_client::OrdersStreamServiceClient;
use crate::t_types::*;
use crate::Api;
use tokio::task::JoinHandle;
use log::{warn, error};
use futures::SinkExt;

trait PingDelayMs {
    const NET_DELAY: u64 = 2000;
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
    fn start_stream<S: futures::Sink<T> + Unpin + Send + 'static>(self, req: Req, response_sender: S) -> impl std::future::Future<Output=Result<JoinHandle<()>, tonic::Status>>;
}

macro_rules! start_stream_impl {
    ($($res:ty = $client:ident : $method:ident($req:ty),)+) => {$(
        impl<T> StartStream<$req, T> for Api where T: From<$res> + Send + 'static {
            fn start_stream<S>(self, req: $req, mut sender: S) -> impl std::future::Future<Output=Result<JoinHandle<()>, tonic::Status>> 
            where S: futures::Sink<T> + Unpin + Send + 'static { Box::pin(async move {
                let timeout = Duration::from_millis(req.delay_ms());
                let mut client = $client::from(self.clone());
                let mut receiver = client.$method(req.clone()).await?.into_inner();
                let handle = tokio::spawn(async move {
                    loop {
                        match tokio::time::timeout(timeout, receiver.message()).await {
                            Ok(Ok(Some(response))) => {
                                if sender.send(response.into()).await.is_err() { break; }
                                continue;
                            },
                            Ok(Ok(None)) => warn!("none received, reconnecting..."),
                            Ok(Err(e)) => log::error!("err {e:?}, reconnecting..."),
                            Err(_) => warn!("timeout, reconnecting..."),
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