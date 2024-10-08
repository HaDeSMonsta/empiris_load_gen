use std::net::SocketAddr;
use tonic::Request;
use crate::math::math_service_client::MathServiceClient;
use crate::math::{MathResponse, MathTask};

pub async fn send(addr: SocketAddr, task: MathTask) -> MathResponse {
    let request = Request::new(task);
    let mut client = MathServiceClient::connect(
        format!("https://{}:{}", addr.ip().to_string(), addr.port())
    ).await.unwrap();
    client.send_task(request)
        .await
        .unwrap()
        .into_inner()
}