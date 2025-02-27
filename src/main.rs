
use std::time::Duration;

use send::{ApiPool, Sender};
use stream::StreamHolder;
use t_types::market_data_stream_service_client::MarketDataStreamServiceClient;
use t_types::operations_stream_service_client::OperationsStreamServiceClient;
use t_types::orders_service_client::OrdersServiceClient;
use t_types::orders_stream_service_client::OrdersStreamServiceClient;
use t_types::{market_data_request, GetAccountsRequest, InfoInstrument, LastPriceInstrument, MarketDataRequest, MoneyValue, PingDelaySettings, PingRequest, PortfolioRequest, Quotation, SubscribeCandlesRequest, SubscribeInfoRequest, SubscribeLastPriceRequest, SubscribeOrderBookRequest, SubscriptionAction};
use tonic::codec::CompressionEncoding::Gzip as GZIP;
use tonic::service::interceptor::InterceptedService;
use tonic::service::Interceptor;
use tonic::transport::{Channel, ClientTlsConfig};
use tonic::{client, Request, Status};
use uuid::Uuid;
use tonic::client::Grpc;


pub type Api = Grpc<IService>;

mod t_types;
mod send;
mod stream;

pub type IService = InterceptedService<Channel, TokenInterceptor>;
pub trait InvestService: Sized {
    fn create_invest_service(token: impl ToString) -> Result<Self, tonic::transport::Error>;
}

impl InvestService for IService {
    fn create_invest_service(token: impl ToString) -> Result<Self, tonic::transport::Error> {
        let tls = ClientTlsConfig::new().with_native_roots();
        let channel = Channel::from_static("https://invest-public-api.tinkoff.ru").tls_config(tls)?.connect_lazy();
        Ok(InterceptedService::new(channel, TokenInterceptor::new(token)))
    }
}
impl InvestService for Grpc<IService> {
    fn create_invest_service(token: impl ToString) -> Result<Self, tonic::transport::Error> {
        let serv = InterceptedService::create_invest_service(token)?;
        let res = Grpc::new(serv).accept_compressed(GZIP).send_compressed(GZIP);
        Ok(res)
    }
}

#[derive(Debug, Clone)]
pub struct TokenInterceptor {
    token: String,
}

impl TokenInterceptor {
    pub fn new(token: impl ToString) -> Self {
        Self { token: token.to_string() }
    }
}

impl Interceptor for TokenInterceptor {
    fn call(&mut self, mut req: Request<()>) -> Result<Request<()>, Status> {
        req.metadata_mut().append(
            "authorization",
            format!("bearer {}", self.token).parse().unwrap(),
        );
        req.metadata_mut()
            .append("x-tracking-id", Uuid::new_v4().to_string().parse().unwrap());
        Ok(req)
    }
}

fn print_money(name: &str, m: &MoneyValue) {
    print!("{}: {}.{} {}, ", name, m.units, m.nano/1000000, m.currency)
}

fn convert_price(Quotation { units, nano }: Quotation) -> i64 {
    units * 100 + (nano/10000000) as i64
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    simplelog::SimpleLogger::init(log::LevelFilter::Info, simplelog::Config::default())?;
    let token = std::env::var("TOKEN")?;
    let api = tonic::client::Grpc::create_invest_service(token)?;
    let pool = ApiPool::new(api.clone());
    let accounts = pool.send(GetAccountsRequest::default()).await?;
    let portfolio = pool.send(PortfolioRequest { account_id: accounts.accounts[0].id.clone(), currency: None }).await?;
    portfolio.total_amount_portfolio.as_ref().map(|m|print_money("total", m));
    portfolio.daily_yield.as_ref().map(|m|print_money("daily yeld", m));
    println!();
    portfolio.positions.iter().for_each(|p|{
        print!("{}, figi: {}, uid: {}, count: {}, ", p.instrument_type, p.figi, p.instrument_uid, p.quantity.unwrap_or_default().units);
        print_money("price", &p.current_price.as_ref().cloned().unwrap_or_default());
        println!();
    });
    let (s,mut r) = tokio::sync::broadcast::channel(10);
    let s2 = s.clone();
    

    //let mut client = OperationsStreamServiceClient::from(api.clone());
    //let mut client = OrdersStreamServiceClient::from(api.clone());
    //let mut client = MarketDataStreamServiceClient::from(api.clone());
    let mut x = StreamHolder::create(api.clone(), s).await?;
    let handle = tokio::spawn(async move {
        println!("streaming created");
        let mut share = (0,0);
        let mut fut = (0,0);
        let mut interval = tokio::time::interval(Duration::from_secs(1));
        loop {
            tokio::select! {
                val = r.recv() => {
                    println!("some received");
                    match val {
                        Ok(msg) => {
                            use t_types::market_data_response::Payload::*;
                            match msg.payload {
                                Some(Ping(ping)) => {
                                    use t_types::market_data_request::Payload;
                                    let pong = Payload::Ping(PingRequest { 
                                        time: ping.time,
                                    });
                                    println!("ping received");
                                    /*s2.send(MarketDataRequest{payload: Some(pong)})
                                        .map(|_|println!("pong sended"))
                                        .unwrap_or_else(|e|println!("pong send err: {e:?}"));
                                    ;*/
                                },
                                Some(Orderbook(t_types::OrderBook{ bids, asks, instrument_uid, .. })) => {
                                    let bid = convert_price(bids.get(0).cloned().unwrap_or_default().price.unwrap_or_default());
                                    let ask = convert_price(asks.get(0).cloned().unwrap_or_default().price.unwrap_or_default()); 
                                    match instrument_uid.as_str() {
                                        "9e9c5921-43ac-47a8-b537-ccf32ebc6da3" => fut = (bid, ask),
                                        "e6123145-9665-43e0-8413-cd61b8aa9b13" => share = (bid, ask),
                                        _ => {},
                                    }
                                },
                                Some(p) => {
                                    println!("{p:?}")
                                },
                                None => println!("None payload"),
                            };
                        },
                        Err(e) => println!("err: {e:?}"),
                    }
                }
                _ = interval.tick() => {
                    let (fbid, fask) = fut;
                    let (sbid, sask) = share;
                    //println!("{fbid};{sbid};{fask};{sask};{};{}", fbid - sbid, fask - sask);
                    print!("-");
                    std::io::Write::flush(&mut std::io::stdout());
                }
            }
        }
    });
    tokio::time::sleep(Duration::from_secs(5)).await;
    x.send(MarketDataRequest{
        payload: Some(t_types::market_data_request::Payload::SubscribeInfoRequest(SubscribeInfoRequest{ 
            subscription_action: SubscriptionAction::Subscribe.into(), 
            instruments: vec![
                InfoInstrument { instrument_id: "e6123145-9665-43e0-8413-cd61b8aa9b13".into(), ..Default::default() }
            ] }))
    }).await; 
    x.send(MarketDataRequest{
        payload: Some(t_types::market_data_request::Payload::SubscribeLastPriceRequest(SubscribeLastPriceRequest{
            subscription_action: SubscriptionAction::Subscribe.into(), 
            instruments: vec![LastPriceInstrument{instrument_id: "e6123145-9665-43e0-8413-cd61b8aa9b13".into(), ..Default::default() }]
        }))
    }).await; 
    if(false) {
        x.send(MarketDataRequest{
            payload: Some(t_types::market_data_request::Payload::SubscribeOrderBookRequest(SubscribeOrderBookRequest{ 
                subscription_action: SubscriptionAction::Subscribe.into(), 
                instruments: vec![t_types::OrderBookInstrument{
                    instrument_id: "9e9c5921-43ac-47a8-b537-ccf32ebc6da3".to_owned(), 
                    depth: 1, 
                    order_book_type: 3 ,
                    ..Default::default()
                }], 
            }))
        }).await;
    }
    println!("orderbook subscribed");
    handle.await?;
    Ok(())
}
