use backend::client::ClientInterface;
use mockall::automock;

#[automock]
#[async_trait::async_trait]
pub trait TestClientInterface: ClientInterface {} 