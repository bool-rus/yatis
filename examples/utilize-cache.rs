use anyhow::anyhow;
use uuid::Uuid;
use yatis::*;
use yatis::t_types::*;

#[tokio::main]
async fn main() -> anyhow::Result<()>{
    let token = std::env::var("TOKEN").expect("need to set env var 'TOKEN'");
    let api = Api::create_invest_service(token)?;
    if let Err(e) = algo(api).await {
        println!("error in algo: {e}");
    };
    Ok(())
}

async fn algo(api: impl InvestApi + Clone) -> anyhow::Result<()> {
    let max_spread = Quotation::from((1,2)); //0.01
    let EtfResponse{ instrument  } = api.request(InstrumentRequest{ 
        id_type: InstrumentIdType::Ticker.into(), 
        class_code: Some("SPBRU".to_string()), 
        id: "TMON@".to_string() 
    }).await?;
    let etf = instrument.ok_or(anyhow!("ETF TMON@ not found"))?;
    let GetOrderBookResponse { bids, asks, instrument_uid, .. } = api.request(GetOrderBookRequest{ 
        depth: 1, instrument_id: Some(etf.uid), ..Default::default()
    }).await?;
    let bid = bids.into_iter().next().map(|o|o.price).flatten().ok_or(anyhow!("no bids"))?;
    let ask = asks.into_iter().next().map(|o|o.price).flatten().ok_or(anyhow!("no asks"))?;
    let sell_price = ask - max_spread;
    let buy_price = bid + max_spread;
    let GetAccountsResponse { accounts } = api.request(GetAccountsRequest::default()).await?;
    let handles: Vec<_> = accounts.into_iter().map(|a|{
        let api = api.clone();
        let instrument_uid = instrument_uid.clone();
        tokio::spawn(async move{utilize_cache(api, a.id, instrument_uid, sell_price, buy_price).await})
    }).collect();
    for h in handles {
        if let Ok(Err(e)) = h.await {
            println!("{e}");
        }
    }
    Ok(())
}

async fn utilize_cache(api: impl InvestApi, account_id: String, instrument_id: String, sell: Quotation, buy: Quotation) -> anyhow::Result<()> {
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
    let rubs = rubs.ok_or(anyhow!("no rubles"))?;
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
