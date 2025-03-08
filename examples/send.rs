use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>>{
    use yatis::*;
    use t_types::*;
    let token = std::env::var("TOKEN").expect("need to set env var 'TOKEN'");
    let api = Api::create_invest_service(token)?;
    let share: ShareResponse = api.clone().request(InstrumentRequest{
        id_type:InstrumentIdType::Ticker.into(),
        class_code:Some("TQBR".to_string()),
        id:"T".to_string()
    }).await?;
    let share = share.instrument.unwrap();
    println!("T: {share:?}\n");
    println!("min lot price increment: {}", share.min_price_increment.unwrap() * share.lot);
    Ok(())
}