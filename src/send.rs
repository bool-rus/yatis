use std::error::Error;
use std::future::Future;
use async_trait::async_trait;

use futures::FutureExt;
use tonic::client::Grpc;

use crate::{IService, InvestService};
use crate::t_types::{GetAccountsRequest, GetAccountsResponse};


type Api = Grpc<IService>;



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
        Self::with_capacity(api, 10)
    }
}


#[async_trait]
pub trait Sender<T> {
    type Response: Sized;
    async fn send(&self, req: T) -> Result<Self::Response, tonic::Status>;
}

macro_rules! sender_impl {
    ($($res:ty = $client:ident : $method:ident ($req:ty), )+) => {$(
        #[async_trait]
        impl Sender<$req> for ApiPool {
            type Response = $res;
        
            async fn send(&self, req: $req) -> Result<Self::Response, tonic::Status> {
                let api = self.0.pop().await;
                let mut client = $client::from(api);
                let res = client.$method(req).await;
                self.0.push(client.into());
                res.map(|r|r.into_inner())
            }
        }
    )+}
}

use crate::t_types::instruments_service_client::InstrumentsServiceClient;
use crate::t_types::users_service_client::UsersServiceClient;
use crate::t_types::operations_service_client::OperationsServiceClient;

use crate::t_types::*;
sender_impl![

    GetAccountsResponse = UsersServiceClient:get_accounts(GetAccountsRequest),
    GetInfoResponse = UsersServiceClient:get_info(GetInfoRequest),
    GetMarginAttributesResponse = UsersServiceClient:get_margin_attributes(GetMarginAttributesRequest),
    GetUserTariffResponse = UsersServiceClient:get_user_tariff(GetUserTariffRequest),
    
    GetDividendsForeignIssuerResponse = OperationsServiceClient:get_dividends_foreign_issuer(GetDividendsForeignIssuerRequest),
    OperationsResponse = OperationsServiceClient:get_operations(OperationsRequest),
    GetOperationsByCursorResponse = OperationsServiceClient:get_operations_by_cursor(GetOperationsByCursorRequest),
    PortfolioResponse = OperationsServiceClient:get_portfolio(PortfolioRequest),
    PositionsResponse = OperationsServiceClient:get_positions(PositionsRequest),
    WithdrawLimitsResponse = OperationsServiceClient:get_withdraw_limits(WithdrawLimitsRequest),


];