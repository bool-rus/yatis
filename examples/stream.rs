use std::error::Error;

#[allow(deprecated)]
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    use t_types::*;
    use yatis::*;
    let token = std::env::var("TOKEN").expect("need to set env var 'TOKEN'");
    let api = Api::create_invest_service(token)?;

    let figi = "TCS80A107UL4".to_string(); //T-Techno shares
    let (s, mut r) = futures::channel::mpsc::channel::<StreamResponse>(10);
    api.clone().start_stream(MarketDataServerSideStreamRequest {
        subscribe_last_price_request: Some(SubscribeLastPriceRequest {
            subscription_action: SubscriptionAction::Subscribe.into(),
            instruments: vec![LastPriceInstrument {figi,..Default::default()}],
            ..Default::default()
        }),
        ping_settings: Some(PingDelaySettings {ping_delay_ms: Some(5000)}),
        subscribe_candles_request: None, subscribe_order_book_request: None, subscribe_trades_request: None, subscribe_info_request: None,
    }, s).await?;
    use futures::StreamExt;
    while let Some(response) = r.next().await {
        println!("{response:?}");
        //do not forget to set exit condition... if needed...
    }
    Ok(())
}
