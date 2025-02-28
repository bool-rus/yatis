
use std::future::Future;

use crate::Api;
use crate::send::Sender;

pub struct ApiPool(deadqueue::unlimited::Queue<Api>);

impl ApiPool {
    pub fn with_capacity(api: Api, capacity: usize) -> Self {
        let q = deadqueue::unlimited::Queue::new();
        for _ in 0..capacity {
            q.push(api.clone());
        }
        Self(q)
    }
    pub fn new(api: Api) -> Self {
        Self::with_capacity(api, 1)
    }
    pub async fn get(&self) -> Api {
        let res = self.0.pop().await;
        self.0.push(res.clone());
        res
    }
    pub async fn with_api<T, Fut: Future<Output=(impl Into<Api>, T)>, Fun: FnOnce(Api) -> Fut>(&self, fun: Fun) -> T where T: Send+Sized {
        let api = self.0.pop().await;
        let (api, res) = fun(api).await;
        self.0.push(api.into());
        res
    }
}

impl<Req, Res> Sender<Req, Res> for &ApiPool where Api: Sender<Req, Res>, Req: Send, Res: Send, <Api as Sender<Req, Res>>::Error: Send {
    type Error = <Api as Sender<Req, Res>>::Error;
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