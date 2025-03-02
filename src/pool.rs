
use std::future::Future;

use tokio::task::JoinHandle;

use crate::stream::StartStream;
use crate::requestor::OwnedSender;

pub struct ApiPool<T>(deadqueue::unlimited::Queue<T>);

impl<Api> ApiPool<Api> {

    pub fn new(api: Api) -> Self {
        let q = deadqueue::unlimited::Queue::new();
        q.push(api);
        Self(q)
    }
    pub fn add(&self, api: Api) {
        self.0.push(api);
    }
    pub async fn with_api<T, Fut: Future<Output=(impl Into<Api>, T)>, Fun: FnOnce(Api) -> Fut>(&self, fun: Fun) -> T where T: Send+Sized {
        let api = self.0.pop().await;
        let (api, res) = fun(api).await;
        self.0.push(api.into());
        res
    }
}

impl<T:Clone> ApiPool<T> {
    pub async fn get(&self) -> T {
        let res = self.0.pop().await;
        self.0.push(res.clone());
        res
    }
}


impl<Api, Req, Res> OwnedSender<Req, Res> for &ApiPool<Api> where Api: OwnedSender<Req, Res>, Req: Send, Res: Send, <Api as OwnedSender<Req, Res>>::Error: Send {
    type Error = <Api as OwnedSender<Req, Res>>::Error;
    fn send_and_back(self, req: Req) -> impl Future<Output = (Self,Result<Res, Self::Error>)> {
        log::warn!("Don use ApiPool::send_and_back! Please, use ApiPool::send");
        Box::pin(async move{(self, self.send(req).await)})
    }
    fn send(self, req: Req) -> impl Future<Output = Result<Res, Self::Error>> {
        self.with_api(move |api|Box::pin(async move {
            api.send_and_back(req).await
        }))
    }
}

impl<Api, Req,T> StartStream<Req,T> for &ApiPool<Api> where Api: StartStream<Req, T> + Clone {
    fn start_stream<S>(self, req: Req, sender: S) -> impl Future<Output=Result<JoinHandle<()>, tonic::Status>> 
    where S: futures::Sink<T> + Unpin + Send + 'static {
        Box::pin(async move {
            let api = self.0.pop().await;
            self.0.push(api.clone());
            api.start_stream(req, sender).await
        })
    }
}