use std::error::Error;

use yatis::{t_types::*, Api, InvestService};
use yatis::requestor::Requestor;


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>>{
    let token = std::env::var("TOKEN").expect("need to set env var 'TOKEN'");
    let api = Api::create_invest_service(token)?;
    let share: ShareResponse = api.clone().request(InstrumentRequest{
        id_type:InstrumentIdType::Ticker.into(),
        class_code:Some("TQBR".to_string()),
        id:"T".to_string()}).await?;
    println!("{share:?}");
    Ok(())
}