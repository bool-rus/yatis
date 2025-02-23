use std::time::Duration;

use send::{ApiPool, Sender};
use t_types::users_service_client::UsersServiceClient;
use t_types::{GetAccountsRequest, MoneyValue, PortfolioRequest};
use tonic::codec::CompressionEncoding::Gzip as GZIP;
use tonic::service::interceptor::InterceptedService;
use tonic::service::Interceptor;
use tonic::transport::{Channel, ClientTlsConfig};
use tonic::{Request, Status};
use uuid::Uuid;
use tonic::client::Grpc;

mod t_types;
mod send;

pub use t_types::market_data_service_client::MarketDataServiceClient;
use t_types::operations_service_client::OperationsServiceClient;

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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let token = std::env::var("TOKEN")?;
    let api = tonic::client::Grpc::create_invest_service(token)?;
    let pool = ApiPool::new(api);
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

    Ok(())
}
