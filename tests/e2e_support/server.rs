use wiremock::MockServer;

pub async fn start() -> MockServer {
    MockServer::start().await
}
