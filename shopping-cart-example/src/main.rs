
use shopcart_example::run_server;

#[tokio::main]
async fn main() -> Result<(), tonic::transport::Error> {
    run_server("0.0.0.0:8088".to_owned()).await
}
