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
    pub async fn with_api<T, Fut: Future<Output=(impl Into<Api>, T)>, Fun: FnOnce(Api) -> Fut>(&self, fun: Fun) -> T where T: Send+Sized {
        let api = self.0.pop().await;
        let (api, res) = fun(api).await;
        self.0.push(api.into());
        res
    }
}


pub trait Sender<Req, Res> {
    fn send(&self, req: Req) -> impl Future<Output = Result<Res, tonic::Status>>;
}

macro_rules! sender_impl {
    ($($res:ty = $client:ident : $method:ident ($req:ty), )+) => {$(
        impl Sender<$req, $res> for ApiPool {
        
            fn send(&self, req: $req) -> impl Future<Output = Result<$res, tonic::Status>> { 
                self.with_api(move |api|Box::pin(async move {
                    let mut client = $client::from(api);
                    let r = client.$method(req).await.map(|r|r.into_inner());
                    (client, r)
                }))
            }
        }
    )+}
}

use crate::t_types::instruments_service_client::InstrumentsServiceClient;
use crate::t_types::users_service_client::UsersServiceClient;
use crate::t_types::operations_service_client::OperationsServiceClient;

use crate::t_types::*;
sender_impl![
    BondResponse = InstrumentsServiceClient:bond_by(InstrumentRequest),
    BondsResponse = InstrumentsServiceClient:bonds(InstrumentsRequest),
    CreateFavoriteGroupResponse = InstrumentsServiceClient:create_favorite_group(CreateFavoriteGroupRequest),
    DeleteFavoriteGroupResponse = InstrumentsServiceClient:delete_favorite_group(DeleteFavoriteGroupRequest),
    CurrenciesResponse = InstrumentsServiceClient:currencies(InstrumentsRequest),
    CurrencyResponse = InstrumentsServiceClient:currency_by(InstrumentRequest),
    EditFavoritesResponse = InstrumentsServiceClient:edit_favorites(EditFavoritesRequest),
    EtfResponse = InstrumentsServiceClient:etf_by(InstrumentRequest),
    EtfsResponse = InstrumentsServiceClient:etfs(InstrumentsRequest),
    FindInstrumentResponse = InstrumentsServiceClient:find_instrument(FindInstrumentRequest),
    FutureResponse = InstrumentsServiceClient:future_by(InstrumentRequest),
    FuturesResponse = InstrumentsServiceClient:futures(InstrumentsRequest),
    GetAccruedInterestsResponse = InstrumentsServiceClient:get_accrued_interests(GetAccruedInterestsRequest),
    AssetResponse = InstrumentsServiceClient:get_asset_by(AssetRequest),
    GetAssetFundamentalsResponse = InstrumentsServiceClient:get_asset_fundamentals(GetAssetFundamentalsRequest),
    GetAssetReportsResponse = InstrumentsServiceClient:get_asset_reports(GetAssetReportsRequest),
    AssetsResponse = InstrumentsServiceClient:get_assets(AssetsRequest),
    GetBondCouponsResponse = InstrumentsServiceClient:get_bond_coupons(GetBondCouponsRequest),
    GetBondEventsResponse = InstrumentsServiceClient:get_bond_events(GetBondEventsRequest),
    Brand = InstrumentsServiceClient:get_brand_by(GetBrandRequest),
    GetBrandsResponse = InstrumentsServiceClient:get_brands(GetBrandsRequest),
    GetConsensusForecastsResponse = InstrumentsServiceClient:get_consensus_forecasts(GetConsensusForecastsRequest),
    GetCountriesResponse = InstrumentsServiceClient:get_countries(GetCountriesRequest),
    GetDividendsResponse = InstrumentsServiceClient:get_dividends(GetDividendsRequest),
    GetFavoriteGroupsResponse = InstrumentsServiceClient:get_favorite_groups(GetFavoriteGroupsRequest),
    GetFavoritesResponse = InstrumentsServiceClient:get_favorites(GetFavoritesRequest),

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