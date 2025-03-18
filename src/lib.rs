#![doc = include_str!("../README.md")]
use requestor::AnyRequestor;
use stream::AnyStream;
use t_types::Quotation;
use tonic::codec::CompressionEncoding::Gzip as GZIP;
use tonic::service::interceptor::InterceptedService;
use tonic::service::Interceptor;
use tonic::transport::{Channel, ClientTlsConfig};
use tonic::{Request, Status};
use uuid::Uuid;
use tonic::client::Grpc;

/// reused tonic's Grpc type, with implementation of [crate::InvestApi]
/// # Examples:
/// ```rust
/// # #[tokio::main]
/// # async fn main() {
///     use yatis::*;
///     use t_types::*;
/// #    let token = std::env::var("TOKEN").expect("need to set env var 'TOKEN'");
///     let api = Api::create_invest_service(token).unwrap();
///     println!("{:?}", api.request(GetInfoRequest{}).await);;
/// # }
/// ``` 
pub type Api = Grpc<IService>;

/// sandbox implementation of api. Also implements [crate::InvestApi]
/// Not that Sandbox maniputlation operations not implemented in trait [crate::InvestApi],
/// but implemented in type SandboxApi
/// # Examples:
/// ```rust
/// # #[tokio::main]
/// # async fn main() {
///     use yatis::*;
///     use t_types::*;
/// #    let token = std::env::var("SANDBOX_TOKEN").expect("need to set env var 'TOKEN'");
///     let api = SandboxApi::create_invest_service(token).unwrap();
///     println!("{:?}", api.request(GetInfoRequest{}).await);
/// # }
/// ``` 
pub use sandbox::Sandbox as SandboxApi;

pub use pool::ApiPool;

pub mod t_types;
pub mod requestor;
pub mod stream;
pub mod stream_response;
pub mod pool;

mod sandbox;
mod quotation;

pub type IService = InterceptedService<Channel, TokenInterceptor>;
/// Self creator
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

pub use requestor::Requestor;
pub use stream::StartStream;
pub use stream_response::StreamResponse;

/// trait for use some methods of investing api.
/// # Examples:
/// ```rust
/// # #[tokio::main]
/// # async fn main() {
/// #    use yatis::*;
/// #    let token = std::env::var("SANDBOX_TOKEN").expect("need to set env var 'TOKEN'");
/// #    let api = yatis::SandboxApi::create_invest_service(token).unwrap();
/// #    trading(api).await;
/// # }
/// async fn trading(api: impl yatis::InvestApi) {
///    use yatis::*;
///    use t_types::*;
///    let res = api.request(GetAccountsRequest::default()).await.unwrap();
///    println!("{:?}", res);
///    println!("{:?}", api.request(GetInfoRequest{}).await);
///    let (s, mut r) = futures::channel::mpsc::channel::<StreamResponse>(10);
///    api.start_stream(PositionsStreamRequest{ 
///        accounts: vec![res.accounts[0].id.clone()], 
///        with_initial_positions: true, 
///        ping_settings: None 
///    }, s.clone()).await.unwrap();
///    use futures::StreamExt; // Receiver implements Stream
///    if let Some(res) = r.next().await {
///       println!("{res:?}");
///    }
/// }
/// ``` 
/// 
pub trait InvestApi: AnyRequestor + AnyStream<StreamResponse> + Send + 'static {}
impl<T> InvestApi for T where T: AnyRequestor + AnyStream<StreamResponse> + Send + 'static {}

pub trait QuotationExt {
    fn floor(&self, increment: Quotation) -> Self;
    fn round(&self, increment: Quotation) -> Self;
}
