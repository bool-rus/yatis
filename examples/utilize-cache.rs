use uuid::Uuid;
use yatis::*;
use yatis::t_types::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{
    let token = std::env::var("TOKEN").expect("need to set env var 'TOKEN'");
    let api = Api::create_invest_service(token)?;
    algo(api).await?;
    //start_algo(api).await?;
    Ok(())
}

async fn algo(api: impl InvestApi + Clone) -> Result<(), Box<dyn std::error::Error>> {
    let max_spread = Quotation::from((1,2)); //0.01
    let EtfResponse{ instrument  } = api.request(InstrumentRequest{ 
        id_type: InstrumentIdType::Ticker.into(), 
        class_code: Some("SPBRU".to_string()), 
        id: "TMON@".to_string() 
    }).await?;
    let etf = instrument.ok_or("ETF TMON@ not found")?;
    let GetOrderBookResponse { bids, asks, instrument_uid, .. } = api.request(GetOrderBookRequest{ 
        depth: 1, instrument_id: Some(etf.uid), ..Default::default()
    }).await?;
    let bid = bids.into_iter().next().map(|o|o.price).flatten().ok_or("no bids")?;
    let ask = asks.into_iter().next().map(|o|o.price).flatten().ok_or("no asks")?;
    let sell_price = ask - max_spread;
    let buy_price = bid + max_spread;
    let GetAccountsResponse { accounts } = api.request(GetAccountsRequest::default()).await?;
    for acc in accounts {
        utilize_cache(api.clone(), acc.id, instrument_uid.clone(), sell_price, buy_price).await?;
    }
    Ok(())
}

async fn utilize_cache(api: impl InvestApi, account_id: String, instrument_id: String, sell: Quotation, buy: Quotation) -> Result<(), Box<dyn std::error::Error>> {
    let PortfolioResponse { positions, account_id, .. } = api.request(PortfolioRequest{ account_id, currency: None }).await?;
    let mut rubs = None;
    let mut etfs = None;
    for p in positions {
        if p.figi == "RUB000UTSTOM" {
            rubs = Some(p.quantity.unwrap_or_default() - p.blocked_lots.unwrap_or_default());
        }
        if p.instrument_uid == instrument_id {
            etfs = Some(p.quantity.unwrap_or_default() - p.blocked_lots.unwrap_or_default());
        }
    }
    let rubs = rubs.ok_or("no rubs")?;
    let etf_quantity = etfs.unwrap_or_default().units;
    let round = Quotation::from((1,0));

    let (mut quantity, direction, price) = if rubs < Quotation::default() {(
        -(rubs / sell).round(round).units, OrderDirection::Sell.into(), Some(sell)
    )} else {(
        (rubs / buy).round(round).units, OrderDirection::Buy.into(), Some(buy)
    )};
    if direction == OrderDirection::Sell.into() && quantity > etf_quantity {
        println!("sell more than have...");
        quantity = etf_quantity;
    }
    if quantity > 0 {
        let order_id = Uuid::new_v4().to_string();
        api.request(PostOrderAsyncRequest{ instrument_id, quantity, price, direction, account_id, order_type: OrderType::Limit.into(), order_id, ..Default::default() }).await?;
        println!("ordered!");
    } else {
        println!("order not needed!");
    }
    Ok(())
}