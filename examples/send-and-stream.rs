use futures::StreamExt;
use yatis::pool::ApiPool;
use yatis::send::Sender;
use yatis::stream::StartStream;
use yatis::stream_response::StreamResponse;
use yatis::Api;
use yatis::t_types::*;
use yatis::InvestService;

fn print_money(name: &str, m: &MoneyValue) {
    print!("{}: {}.{} {}, ", name, m.units, m.nano/1000000, m.currency)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    simplelog::SimpleLogger::init(log::LevelFilter::Info, simplelog::Config::default())?;
    let token = std::env::var("TOKEN")?;
    let api = Api::create_invest_service(token)?;
    let pool = ApiPool::new(api.clone());
    pool.add(api.clone());
    pool.add(api);
    let accounts = pool.send(GetAccountsRequest::default()).await?;
    let portfolio = pool.send(PortfolioRequest { account_id: accounts.accounts[0].id.clone(), currency: None }).await?;
    let t: ShareResponse = pool.send(InstrumentRequest { id_type: InstrumentIdType::Ticker.into(), class_code: Some("TQBR".to_string()), id: "T".to_string() }).await?;
    log::info!("t share: {t:?}");
    let t = t.instrument.unwrap().uid;
    portfolio.total_amount_portfolio.as_ref().map(|m|print_money("total", m));
    portfolio.daily_yield.as_ref().map(|m|print_money("daily yeld", m));
    println!();
    portfolio.positions.iter().for_each(|p|{
        print!("{}, figi: {}, uid: {}, count: {}, ", p.instrument_type, p.figi, p.instrument_uid, p.quantity.unwrap_or_default().units);
        print_money("price", &p.current_price.as_ref().cloned().unwrap_or_default());
        println!();
    });
    let (s,mut r) = futures::channel::mpsc::channel::<StreamResponse>(10);
    
    pool.start_stream(MarketDataServerSideStreamRequest{ 
        subscribe_candles_request: None, 
        subscribe_order_book_request: None, 
        subscribe_trades_request: None, 
        subscribe_info_request: None, 
        subscribe_last_price_request: Some(SubscribeLastPriceRequest{
            subscription_action: SubscriptionAction::Subscribe.into(), 
            instruments: vec![LastPriceInstrument{instrument_id: t.clone(), ..Default::default() }]
        }), 
        ping_settings: Some(PingDelaySettings { ping_delay_ms: Some(5000) }),
    }, s).await?;
    log::info!("initializing sequence completed");
    loop {
        let response = r.next().await.unwrap();
        log::info!("{response:?}");  
    }
}
