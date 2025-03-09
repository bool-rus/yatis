use tonic::client::Grpc;
use tonic::transport::{Channel, ClientTlsConfig};
use tonic::codec::CompressionEncoding::Gzip as GZIP;

use crate::{Api, InvestService, StartStream, StreamResponse, TokenInterceptor};

#[derive(Clone)]
pub struct Sandbox(Api);

impl InvestService for Sandbox {
    fn create_invest_service(token: impl ToString) -> Result<Self, tonic::transport::Error> {
        let tls = ClientTlsConfig::new().with_native_roots();
        let channel = Channel::from_static("https://sandbox-invest-public-api.tinkoff.ru").tls_config(tls)?.connect_lazy();
        let serv = tonic::service::interceptor::InterceptedService::new(channel, TokenInterceptor::new(token));
        let g = Grpc::new(serv).accept_compressed(GZIP).send_compressed(GZIP);
        Ok(Self(g))
    }
}

macro_rules! sandbox_sender_impl {
    ($($res:ty = $client:ident : $method:ident ($req:ty), )+) => {
        $(
        impl OwnedSender<$req,$res> for Sandbox {
            fn send_and_back(self, req: $req) -> impl std::future::Future<Output = (Self,Result<$res, tonic::Status>)> {Box::pin(async move {
                let mut client = $client::from(self.0);
                let r = client.$method(req).await.map(|r|r.into_inner());
                (Self(client.into()), r)
            })}
            fn send(&self, req: $req) -> impl std::future::Future<Output = Result<$res, tonic::Status>> {Box::pin(async move {
                let mut client = $client::from(self.0.clone());
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

use crate::t_types::sandbox_service_client::SandboxServiceClient;

use crate::t_types::*;
use crate::requestor::{AnyRequestor, OwnedSender};

impl AnyRequestor for Sandbox {}

sandbox_sender_impl![
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
    OperationsResponse = SandboxServiceClient:get_sandbox_operations(OperationsRequest),
    GetOperationsByCursorResponse = SandboxServiceClient:get_sandbox_operations_by_cursor(GetOperationsByCursorRequest),
    PortfolioResponse = SandboxServiceClient:get_sandbox_portfolio(PortfolioRequest),
    PositionsResponse = SandboxServiceClient:get_sandbox_positions(PositionsRequest),
    WithdrawLimitsResponse = SandboxServiceClient:get_sandbox_withdraw_limits(WithdrawLimitsRequest),

    CancelOrderResponse = SandboxServiceClient:cancel_sandbox_order(CancelOrderRequest),
    GetMaxLotsResponse = SandboxServiceClient:get_sandbox_max_lots(GetMaxLotsRequest),
    GetOrderPriceResponse = OrdersServiceClient:get_order_price(GetOrderPriceRequest),
    OrderState = SandboxServiceClient:get_sandbox_order_state(GetOrderStateRequest),
    GetOrdersResponse = SandboxServiceClient:get_sandbox_orders(GetOrdersRequest),
    PostOrderResponse = SandboxServiceClient:post_sandbox_order(PostOrderRequest),
    PostOrderAsyncResponse = OrdersServiceClient:post_order_async(PostOrderAsyncRequest),
    PostOrderResponse = SandboxServiceClient:replace_sandbox_order(ReplaceOrderRequest),

    GetSignalsResponse = SignalServiceClient:get_signals(GetSignalsRequest),
    GetStrategiesResponse = SignalServiceClient:get_strategies(GetStrategiesRequest),

    CancelStopOrderResponse = StopOrdersServiceClient:cancel_stop_order(CancelStopOrderRequest),
    GetStopOrdersResponse = StopOrdersServiceClient:get_stop_orders(GetStopOrdersRequest),
    PostStopOrderResponse = StopOrdersServiceClient:post_stop_order(PostStopOrderRequest),

    GetAccountsResponse = SandboxServiceClient:get_sandbox_accounts(GetAccountsRequest),
    GetInfoResponse = UsersServiceClient:get_info(GetInfoRequest),
    GetMarginAttributesResponse = UsersServiceClient:get_margin_attributes(GetMarginAttributesRequest),
    GetUserTariffResponse = UsersServiceClient:get_user_tariff(GetUserTariffRequest),

    OpenSandboxAccountResponse = SandboxServiceClient:open_sandbox_account(OpenSandboxAccountRequest),
    CloseSandboxAccountResponse = SandboxServiceClient:close_sandbox_account(CloseSandboxAccountRequest),
    SandboxPayInResponse = SandboxServiceClient:sandbox_pay_in(SandboxPayInRequest),
];


impl<Req, T> StartStream<Req,T> for Sandbox where Api: StartStream<Req, T> + Clone {
    fn start_stream<S>(&self, req: Req, sender: S) -> impl std::future::Future<Output=Result<tokio::task::JoinHandle<()>, tonic::Status>> 
    where S: futures::Sink<T> + Unpin + Send + 'static {
        Box::pin(async move {
            let Self(api) = self;
            api.start_stream(req, sender).await
        })
    }
}

impl crate::stream::AnyStream<StreamResponse> for Sandbox {}