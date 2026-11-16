use highrise_rs::{BotHandler, BoxFuture, Client, Config, Highrise, SessionMetadata, User};

struct EchoBot;

impl BotHandler for EchoBot {
    fn on_ready<'a>(&'a self, _api: &'a Highrise, meta: SessionMetadata) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            println!("Online as {} in {}", meta.user_id, meta.room_info.room_name);
        })
    }

    fn on_chat<'a>(&'a self, api: &'a Highrise, user: User, message: String) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            if message == "!ping" {
                let _ = api.whisper(&user.id, "Pong!").await;
            }
        })
    }
}

#[tokio::main]
async fn main() {
    let token = std::env::var("API_TOKEN").expect("API_TOKEN required");
    let room_id = std::env::var("ROOM_ID").expect("ROOM_ID required");
    let client = Client::new(Config::new(token, room_id), EchoBot);
    if let Err(err) = client.run().await {
        eprintln!("Bot stopped: {err}");
    }
}
