mod fallback;
mod serv;

#[tokio::main]
async fn main() {
    serv::serv().await;
}
