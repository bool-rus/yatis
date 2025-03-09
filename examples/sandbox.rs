use std::error::Error;
use yatis::*;
use t_types::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>>{
    let token = std::env::var("SANDBOX_TOKEN").expect("need to set env var 'SANDBOX_TOKEN'");
    let api = SandboxApi::create_invest_service(token)?;
    let name = "yatis test sanbox".to_owned();
    let r = api.request(OpenSandboxAccountRequest{ name: Some(name.clone()) }).await?;
    let r = api.request(SandboxPayInRequest{ 
        account_id: r.account_id, 
        amount: Some(MoneyValue { currency: "RUB".to_string(), units: 50000, nano: 300000000 }) 
    }).await?;
    println!("{r:?}");
    let accounts = any_trade_algo(api.clone()).await?;
    for a in accounts.accounts {
        if a.name == name {
            let r = api.request(CloseSandboxAccountRequest{ account_id: a.id }).await;
            println!("{r:?}");
            break;
        }
    }
    Ok(())
}

async fn any_trade_algo(api: impl InvestApi) -> Result<GetAccountsResponse, tonic::Status> {
    let r = api.request(GetAccountsRequest::default()).await?;
    println!("{r:?}");
    for a in r.accounts.clone() {
        println!("portfolio for {}:", a.name);
        let r = api.request(PortfolioRequest{ account_id: a.id, currency: None }).await?;
        for p in r.positions {
            println!("\t{}: {}", p.figi, p.quantity.unwrap());
        }
    }
    Ok(r)
}