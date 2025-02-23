fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("running prost codegen");
    let services = [
        "InstrumentsService", 
        "MarketDataService", 
        "OperationsService", 
        "OrdersService", 
        "SanboxService", 
        "SignalsService", 
        "StopOrderService", 
        "UsersService"
    ];
    tonic_build::configure().build_server(false)
        .protoc_arg("--experimental_allow_proto3_optional")
        .client_attribute(".", "#[derive(derive_more::From, derive_more::Into)]")
        .compile_protos(
            &[
                "investAPI/src/docs/contracts/common.proto",
                "investAPI/src/docs/contracts/instruments.proto",
                "investAPI/src/docs/contracts/marketdata.proto",
                "investAPI/src/docs/contracts/operations.proto",
                "investAPI/src/docs/contracts/orders.proto",
                "investAPI/src/docs/contracts/sandbox.proto",
                "investAPI/src/docs/contracts/signals.proto",
                "investAPI/src/docs/contracts/stoporders.proto",
                "investAPI/src/docs/contracts/users.proto",
            ],
            &["investAPI/src/docs/contracts"],
        )?;
    Ok(())
}