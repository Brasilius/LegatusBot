use serenity::{
    async_trait,
    model::{channel::Message, gateway::Ready},
    prelude::*,
    framework::standard::{
        StandardFramework,
        CommandResult,
        macros::{command, group},
    },
};
use reqwest;
use serde_json;
use tokio;
use dotenv::dotenv;
use std::env;

struct Handler;

#[group]
#[commands(ping, search)]
struct General;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id.say(&ctx.http, "Pong!").await?;
    Ok(())
}

#[command]
async fn search(ctx: &Context, msg: &Message) -> CommandResult {
    // Ensure there is a query to search for
    if msg.content.len() > "!search ".len() {
        let search_query = &msg.content["!search ".len()..].trim();
        let response = search_ai_api(search_query).await;
        msg.channel_id.say(&ctx.http, &response).await?;
    } else {
        msg.channel_id.say(&ctx.http, "Please provide a search query.").await?;
    }
    Ok(())
}

async fn search_ai_api(query: &str) -> String {
    let api_endpoint = "https://api.openai.com/v1/engines/davinci/completions"; // OpenAI's endpoint
    let api_key = env::var("OPENAI_API_KEY").expect("Expected OPENAI_API_KEY in the environment");

    let client = reqwest::Client::new();
    let params = serde_json::json!({
        "prompt": query,
        "max_tokens": 150
    });

    if let Ok(response) = client
        .post(api_endpoint)
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&params)
        .send()
        .await {
            if let Ok(response_text) = response.text().await {
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&response_text) {
                    if let Some(answer) = parsed["choices"][0]["text"].as_str() {
                        return answer.to_string();
                    }
                }
            }
        }

    "Failed to fetch or parse response from AI".to_string()
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    let framework = StandardFramework::new()
        .configure(|c| c.prefix("!")) // Set the bot's prefix to "!"
        .group(&GENERAL_GROUP);

    let mut client = Client::builder(&token)
        .event_handler(Handler)
        .framework(framework) // Include the framework here
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
