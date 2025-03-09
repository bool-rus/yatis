use requestor::AnyRequestor;
use stream::AnyStream;
use tonic::codec::CompressionEncoding::Gzip as GZIP;
use tonic::service::interceptor::InterceptedService;
use tonic::service::Interceptor;
use tonic::transport::{Channel, ClientTlsConfig};
use tonic::{Request, Status};
use uuid::Uuid;
use tonic::client::Grpc;


pub type Api = Grpc<IService>;
pub use sandbox::Sandbox as SandboxApi;

pub mod t_types;
pub mod requestor;
pub mod stream;
pub mod stream_response;
pub mod pool;

mod sandbox;
mod quotation;

//under development
mod bidirect;

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

pub use requestor::Requestor;
pub use stream::StartStream;
pub use stream_response::StreamResponse;

pub trait InvestApi: AnyRequestor + AnyStream<StreamResponse> {}
impl<T> InvestApi for T where T: AnyRequestor + AnyStream<StreamResponse> {}
