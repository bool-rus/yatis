//! Traits and implementations for unary requests to API. Target is implement single method [Requestor::request] for all Requests and Responses.
use std::future::Future;
use crate::Api;

/// Auto implemented trait to send unary requests to grpc. Used for best type derivation. Uses [OwnedSender] implementation.
pub trait Requestor<Req, Res> where Self: Sized {
    /// send unary request to grpc and process response
    fn request(&self, req: Req) -> impl Future<Output = Result<Res, tonic::Status>>;
}

impl<Api, Req, Res> Requestor<Req, Res> for Api where Api: OwnedSender<Req, Res>, Req: Send, Res: Send {
    fn request(&self, req: Req) -> impl Future<Output = Result<Res, tonic::Status>> {
        self.send(req)
    }
}

/// Main trait for unary requests.
pub trait OwnedSender<Req, Res> where Self: Sized {
    /// takes ownership, execute request and return self back after execution. Need for reusing of channels
    fn send_and_back(self, req: Req) -> impl Future<Output = (Self,Result<Res, tonic::Status>)>;
    /// just execute request, using &self. In most implementatinos it clones self
    fn send(&self, req: Req) -> impl Future<Output = Result<Res, tonic::Status>>;
}

macro_rules! sender_impl {
    ($($res:ty = $client:ident : $method:ident ($req:ty), )+) => {
        /// Trait that implement OwnedSender with all unary Requests/Response
        pub trait AnyRequestor: $(OwnedSender<$req,$res> + )+ Send {}
        $(
        impl OwnedSender<$req,$res> for Api {
            fn send_and_back(self, req: $req) -> impl Future<Output = (Self,Result<$res, tonic::Status>)> {Box::pin(async move {
                let mut client = $client::from(self);
                let r = client.$method(req).await.map(|r|r.into_inner());
                (client.into(), r)
            })}
            fn send(&self, req: $req) -> impl Future<Output = Result<$res, tonic::Status>> {Box::pin(async move {
                let mut client = $client::from(self.clone());
                let r = client.$method(req).await.map(|r|r.into_inner());
                r
            })}
        }
    )+}
}

use crate::t_types::instruments_service_client::InstrumentsServiceClient;
use crate::t_types::operations_service_client::OperationsServiceClient;
use crate::t_types::market_data_service_client::MarketDataServiceClient;
use crate::t_types::orders_service_client::OrdersServiceClient;
use crate::t_types::signal_service_client::SignalServiceClient;
use crate::t_types::stop_orders_service_client::StopOrdersServiceClient;
use crate::t_types::users_service_client::UsersServiceClient;

impl AnyRequestor for Api {}

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
    GetForecastResponse = InstrumentsServiceClient:get_forecast_by(GetForecastRequest),
    GetFuturesMarginResponse = InstrumentsServiceClient:get_futures_margin(GetFuturesMarginRequest),
    InstrumentResponse = InstrumentsServiceClient:get_instrument_by(InstrumentRequest),
    RiskRatesResponse = InstrumentsServiceClient:get_risk_rates(RiskRatesRequest),
    IndicativesResponse = InstrumentsServiceClient:indicatives(IndicativesRequest),
    OptionResponse = InstrumentsServiceClient:option_by(InstrumentRequest),
    OptionsResponse = InstrumentsServiceClient:options_by(FilterOptionsRequest),
    ShareResponse = InstrumentsServiceClient:share_by(InstrumentRequest),
    SharesResponse = InstrumentsServiceClient:shares(InstrumentsRequest),
    TradingSchedulesResponse = InstrumentsServiceClient:trading_schedules(TradingSchedulesRequest),

    GetCandlesResponse = MarketDataServiceClient:get_candles(GetCandlesRequest),
    GetClosePricesResponse = MarketDataServiceClient:get_close_prices(GetClosePricesRequest),
    GetLastPricesResponse = MarketDataServiceClient:get_last_prices(GetLastPricesRequest),
    GetLastTradesResponse = MarketDataServiceClient:get_last_trades(GetLastTradesRequest),
    GetOrderBookResponse = MarketDataServiceClient:get_order_book(GetOrderBookRequest),
    GetTechAnalysisResponse = MarketDataServiceClient:get_tech_analysis(GetTechAnalysisRequest),
    GetTradingStatusResponse = MarketDataServiceClient:get_trading_status(GetTradingStatusRequest),
    GetTradingStatusesResponse = MarketDataServiceClient:get_trading_statuses(GetTradingStatusesRequest),
    
    GetDividendsForeignIssuerResponse = OperationsServiceClient:get_dividends_foreign_issuer(GetDividendsForeignIssuerRequest),
    OperationsResponse = OperationsServiceClient:get_operations(OperationsRequest),
    GetOperationsByCursorResponse = OperationsServiceClient:get_operations_by_cursor(GetOperationsByCursorRequest),
    PortfolioResponse = OperationsServiceClient:get_portfolio(PortfolioRequest),
    PositionsResponse = OperationsServiceClient:get_positions(PositionsRequest),
    WithdrawLimitsResponse = OperationsServiceClient:get_withdraw_limits(WithdrawLimitsRequest),

    CancelOrderResponse = OrdersServiceClient:cancel_order(CancelOrderRequest),
    GetMaxLotsResponse = OrdersServiceClient:get_max_lots(GetMaxLotsRequest),
    GetOrderPriceResponse = OrdersServiceClient:get_order_price(GetOrderPriceRequest),
    OrderState = OrdersServiceClient:get_order_state(GetOrderStateRequest),
    GetOrdersResponse = OrdersServiceClient:get_orders(GetOrdersRequest),
    PostOrderResponse = OrdersServiceClient:post_order(PostOrderRequest),
    PostOrderAsyncResponse = OrdersServiceClient:post_order_async(PostOrderAsyncRequest),
    PostOrderResponse = OrdersServiceClient:replace_order(ReplaceOrderRequest),

    GetSignalsResponse = SignalServiceClient:get_signals(GetSignalsRequest),
    GetStrategiesResponse = SignalServiceClient:get_strategies(GetStrategiesRequest),

    CancelStopOrderResponse = StopOrdersServiceClient:cancel_stop_order(CancelStopOrderRequest),
    GetStopOrdersResponse = StopOrdersServiceClient:get_stop_orders(GetStopOrdersRequest),
    PostStopOrderResponse = StopOrdersServiceClient:post_stop_order(PostStopOrderRequest),

    GetAccountsResponse = UsersServiceClient:get_accounts(GetAccountsRequest),
    GetInfoResponse = UsersServiceClient:get_info(GetInfoRequest),
    GetMarginAttributesResponse = UsersServiceClient:get_margin_attributes(GetMarginAttributesRequest),
    GetUserTariffResponse = UsersServiceClient:get_user_tariff(GetUserTariffRequest),
];
