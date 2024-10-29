use crate::math::math_service_client::MathServiceClient;
use crate::math::{MathResponse, MathTask};
use std::net::SocketAddr;
use tonic::Request;

pub async fn send(addr: SocketAddr, task: MathTask) -> Option<MathResponse> {
    let request = Request::new(task);
    let mut client = match MathServiceClient::connect(
        format!("https://{}:{}", addr.ip().to_string(), addr.port())
    ).await {
        Ok(c) => c,
        Err(_) => {
            return None;
        }
    };
    let res = client.send_task(request)
                    .await
                    .unwrap()
                    .into_inner();
    Some(res)
}
