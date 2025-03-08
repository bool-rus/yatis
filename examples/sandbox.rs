use std::error::Error;

use yatis::t_types::sandbox_service_client::SandboxServiceClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>>{
    use yatis::*;
    use t_types::*;
    let token = std::env::var("SANDBOX_TOKEN").expect("need to set env var 'SANDBOX_TOKEN'");
    let api = Api::create_sandbox_invest_service(token)?;
    let r = api.clone().request(OpenSandboxAccountRequest{ name: Some("sanbox".to_owned()) }).await?;
    let r = api.clone().request(SandboxPayInRequest{ 
        account_id: r.account_id, 
        amount: Some(MoneyValue { currency: "RUB".to_string(), units: 50000, nano: 0 }) 
    }).await?;
    println!("{r:?}");
    
    let r = api.clone().request(GetAccountsRequest::default()).await?;
    println!("{r:?}");
    Ok(())
}