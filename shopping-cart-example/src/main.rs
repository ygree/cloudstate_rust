
use shopcart_example::run;

#[tokio::main]
async fn main() -> Result<(), tonic::transport::Error> {
    run("0.0.0.0:8088".to_owned()).await
}
