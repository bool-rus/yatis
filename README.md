# YATIS - Yet Another TBank Investment Sdk

## Usage

### Creating api

```rust
    use yatis::*;
    use t_types::*;
    let token = std::env::var("TOKEN").expect("need to set env var 'TOKEN'");
    let api = Api::create_invest_service(token)?;
```

### Unary requests

```rust
    let share: ShareResponse = api.clone().request(InstrumentRequest{
        id_type:InstrumentIdType::Ticker.into(),
        class_code:Some("TQBR".to_string()),
        id:"T".to_string()
    }).await?;
    println!("T: {share:?}\n");
```

### Stream subscriptions

```rust
    let figi = "TCS80A107UL4".to_string(); //T-Techno shares
    let (s, mut r) = futures::channel::mpsc::channel::<StreamResponse>(10);
    api.clone().start_stream(MarketDataServerSideStreamRequest {
        subscribe_last_price_request: Some(SubscribeLastPriceRequest {
            subscription_action: SubscriptionAction::Subscribe.into(),
            instruments: vec![LastPriceInstrument {figi,..Default::default()}],
            ..Default::default()
        }),
        ping_settings: Some(PingDelaySettings {ping_delay_ms: Some(5000)}), //used by sdk to detect hung connections
        subscribe_candles_request: None, subscribe_order_book_request: None, subscribe_trades_request: None, subscribe_info_request: None,
    }, s).await?;
    use futures::StreamExt;
    while let Some(response) = r.next().await {
        println!("{response:?}");
        //do not forget to set exit condition... if needed...
    }
```

## Targets

- more usability
- easy to use any types of generated proto of  [investAPI]
- ability to create your own implementation of common traits, like `InvestService`
- easy to use pool objects

## Goals

- [x] Investing api implementation
- [x] Unary operations (all)
- [x] Pool of reusable connections
- [ ] Sandbox API
- [x] Server side streams (all)
  - [x] Authomatic reconnect on stucked connections
  - [x] Resubscribtion on reconnect
  - [ ] Connections limitation by tariff
- [ ] Bidirecional streams
  - [ ] Balanced pool of streams
  - [ ] Authomatic reconnect on stucked connections
  - [ ] Resubscribtion on reconnect

[investAPI]: https://github.com/RussianInvestments/investAPI/tree/124813610a9dbb0d8c91067a67d9c26a02c8c713/src/docs/contracts
