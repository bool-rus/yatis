# YATIS - Yet Another TBank Investment Sdk

## Usage

```rust
use yatis::*;
use t_types::*;
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let token = std::env::var("SANDBOX_TOKEN").expect("need to set env var 'TOKEN'");
    let api = Api::create_invest_service(token)?; //creating api
    //let api = SandboxApi::create_invest_service(token)?; //or sandbox api
    trading(api).await
}
async fn trading(api: impl InvestApi) -> Result<(), Box<dyn std::error::Error>> {
    // You can execute unary requests:
    let share: ShareResponse = api.request(InstrumentRequest{ //use native types from investAPI
        id_type:InstrumentIdType::Ticker.into(),
        class_code:Some("TQBR".to_string()),
        id:"T".to_string()
    }).await?;
    println!("T: {share:?}\n");
    let figi = "TCS80A107UL4".to_string(); //T-Techno shares

    // Or create streams:
    let (s, mut r) = futures::channel::mpsc::channel::<StreamResponse>(10);
    api.start_stream(MarketDataServerSideStreamRequest {
        subscribe_last_price_request: Some(SubscribeLastPriceRequest {
            subscription_action: SubscriptionAction::Subscribe.into(),
            instruments: vec![LastPriceInstrument {figi,..Default::default()}],
            ..Default::default()
        }),
        ping_settings: Some(PingDelaySettings {ping_delay_ms: Some(5000)}), //used by yatis to detect hung connections
        subscribe_candles_request: None, subscribe_order_book_request: None, subscribe_trades_request: None, subscribe_info_request: None,
    }, s).await?;
    use futures::StreamExt;
    if let Some(response) = r.next().await { //or while instead of if
        println!("{response:?}");
    }

    Ok(())
}
```

## Targets

- more usability
- easy to use any types of generated proto of  [investAPI]
- ability to create your own implementation of common traits, like `InvestService`, `OwnedSender` and `StartStream`
- easy to use pool objects

## Goals

- [x] Investing api implementation
- [x] Single api trait for all operations `InvestApi`
- [x] Unary operations (all)
- [x] Pool of reusable connections
- [x] Sandbox API with polymorphism (see examples)
- [x] Server side streams (all)
  - [x] Authomatic reconnect on stucked connections
  - [x] Resubscribtion on reconnect
  - [ ] Connections limitation by tariff
- [ ] Bidirecional streams
  - [ ] Balanced pool of streams
  - [ ] Authomatic reconnect on stucked connections
  - [ ] Resubscribtion on reconnect
- [x] Arithmetic opertions with `Quotation` 

[investAPI]: https://github.com/RussianInvestments/investAPI/tree/124813610a9dbb0d8c91067a67d9c26a02c8c713/src/docs/contracts
