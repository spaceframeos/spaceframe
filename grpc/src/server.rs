use tonic::transport::Server;

use spaceframe_grpc::hello_world::greeter_server::GreeterServer;
use spaceframe_grpc::MyGreeter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let greeter = MyGreeter::default();

    println!("Server started on port 50051");

    Server::builder()
        .add_service(GreeterServer::new(greeter))
        .serve(addr)
        .await?;

    Ok(())
}
