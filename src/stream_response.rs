use crate::t_types::*;
pub use order_state_stream_response::OrderState as OrderStateResponse;

#[derive(Debug, Clone)]
pub enum StreamResponse {
    Ping,
    OrderTrades(OrderTrades),
    TradesSubscription(SubscriptionResponse),
    LastPrice(LastPrice),
    SubscribeCandlesResponse(SubscribeCandlesResponse),
    SubscribeOrderBookResponse(SubscribeOrderBookResponse),
    SubscribeLastPriceResponse(SubscribeLastPriceResponse),
    SubscribeTradesResponse(SubscribeTradesResponse),
    SubscribeInfoResponse(SubscribeInfoResponse),
    PortfolioSubscriptionResult(PortfolioSubscriptionResult),
    Candle(Candle),
    Trade(Trade),
    Orderbook(OrderBook),
    TradingStatus(TradingStatus),
    PortfolioResponse(PortfolioResponse),
    MyTradesSubscription(SubscriptionResponse),
    OrderState(OrderStateResponse),
    Position(PositionData),
    PositionsSubscriptions(PositionsSubscriptionResult),
    InitialPositions(PositionsResponse),
}

impl From<MarketDataResponse> for StreamResponse {
    fn from(value: MarketDataResponse) -> Self {
        use market_data_response::Payload;
        match value.payload {
            Some(Payload::SubscribeCandlesResponse(x)) => Self::SubscribeCandlesResponse(x),
            Some(Payload::SubscribeOrderBookResponse(x)) => Self::SubscribeOrderBookResponse(x),
            Some(Payload::SubscribeTradesResponse(x)) => Self::SubscribeTradesResponse(x),
            Some(Payload::SubscribeInfoResponse(x)) => Self::SubscribeInfoResponse(x),
            Some(Payload::SubscribeLastPriceResponse(x)) => Self::SubscribeLastPriceResponse(x),
            Some(Payload::Candle(x)) => Self::Candle(x),
            Some(Payload::Trade(x)) => Self::Trade(x),
            Some(Payload::Orderbook(x)) => Self::Orderbook(x),
            Some(Payload::TradingStatus(x)) => Self::TradingStatus(x),
            Some(Payload::LastPrice(x)) => Self::LastPrice(x),
            None | Some(Payload::Ping(_)) => Self::Ping,
        }
    }
}
impl From<TradesStreamResponse> for StreamResponse {
    fn from(value: TradesStreamResponse) -> Self {
        use trades_stream_response::Payload;
        match value.payload {
            Some(Payload::OrderTrades(x)) => Self::OrderTrades(x),
            Some(Payload::Subscription(x)) => Self::TradesSubscription(x),
            None | Some(Payload::Ping(_)) => Self::Ping,
        }
    }
}

impl From<PortfolioStreamResponse> for StreamResponse {
    fn from(value: PortfolioStreamResponse) -> Self {
        use portfolio_stream_response::Payload;
        match value.payload {
            Some(Payload::Subscriptions(x)) => Self::PortfolioSubscriptionResult(x),
            Some(Payload::Portfolio(x)) => Self::PortfolioResponse(x),
            None | Some(Payload::Ping(_)) => Self::Ping,
        }
    }
}

impl From<PositionsStreamResponse> for StreamResponse {
    fn from(value: PositionsStreamResponse) -> Self {
        use positions_stream_response::Payload;
        match value.payload {
            Some(Payload::Subscriptions(x)) => Self::PositionsSubscriptions(x),
            Some(Payload::Position(x)) => Self::Position(x),
            Some(Payload::InitialPositions(x)) => Self::InitialPositions(x),
            None | Some(Payload::Ping(_)) => Self::Ping,
        }
    }
}

impl From<OrderStateStreamResponse> for StreamResponse {
    fn from(value: OrderStateStreamResponse) -> Self {
        use order_state_stream_response::Payload;
        match value.payload {
            Some(Payload::OrderState(x)) => Self::OrderState(x),
            Some(Payload::Subscription(x)) => Self::MyTradesSubscription(x),
            None | Some(Payload::Ping(_)) => Self::Ping,
        }
    }
}