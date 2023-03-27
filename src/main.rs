mod api;
mod commands;
mod utils;

use std::env;

use serenity::async_trait;
use serenity::model::application::interaction::Interaction;
use serenity::model::gateway::Ready;
use serenity::model::prelude::interaction::InteractionResponseType;
use serenity::model::prelude::{ChannelId, GuildId};
use serenity::prelude::*;

pub struct Handler {
    guild_id: GuildId,
    jellyfin_announcements_channel_id: ChannelId,
    shuffle_category_id: ChannelId,
    lobby_channel_id: ChannelId,
}

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            println!(
                "Received command interaction: {:#?} from {:#?}",
                command.data.name, command.user.name
            );

            let response_message = match command.data.name.as_str() {
                "announce" => commands::announce::run(&command.data.options, &self, &ctx).await,
                "shuffle" => commands::shuffle::run(&command, &self, &ctx).await,
                _ => "not implemented".to_string(),
            };

            if let Err(why) = command
                .create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| {
                            message.content(response_message).ephemeral(true)
                        })
                })
                .await
            {
                println!("Cannot respond to slash command: {}", why);
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        let commands = GuildId::set_application_commands(&self.guild_id, &ctx.http, |commands| {
            commands
                .create_application_command(|command| commands::announce::register(command))
                .create_application_command(|command| commands::shuffle::register(command))
        })
        .await;

        println!("I created the following slash command: {:#?}", commands);
    }
}

#[tokio::main]
async fn main() {
    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    let guild_id = GuildId(
        env::var("GUILD_ID")
            .expect("Expected GUILD_ID in environment")
            .parse()
            .expect("GUILD_ID must be an integer"),
    );

    let jellyfin_announcements_channel_id = ChannelId(
        env::var("JELLYFIN_ANNOUNCEMENTS_CHANNEL_ID")
            .expect("Expected JELLYFIN_ANNOUNCEMENTS_CHANNEL_ID in environment")
            .parse()
            .expect("JELLYFIN_ANNOUNCEMENTS_CHANNEL_ID must be an integer"),
    );

    let shuffle_category_id = ChannelId(
        env::var("SHUFFLE_CATEGORY_ID")
            .expect("Expected SHUFFLE_CATEGORY_ID in environment")
            .parse()
            .expect("SHUFFLE_CATEGORY_ID must be an integer"),
    );

    let lobby_channel_id = ChannelId(
        env::var("LOBBY_CHANNEL_ID")
            .expect("Expected LOBBY_CHANNEL_ID in environment")
            .parse()
            .expect("LOBBY_CHANNEL_ID must be an integer"),
    );

    // Build our client.
    let mut client = Client::builder(
        token,
        GatewayIntents::GUILDS | GatewayIntents::GUILD_VOICE_STATES,
    )
    .event_handler(Handler {
        guild_id,
        jellyfin_announcements_channel_id,
        shuffle_category_id,
        lobby_channel_id,
    })
    .await
    .expect("Error creating client");

    let shard_manager = client.shard_manager.clone();

    // Handle gracefull shutdown on sigint (ctrl+c) and sigterm
    tokio::spawn(async move {
        let mut sigint =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::interrupt()).unwrap();
        let mut sigterm =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()).unwrap();

        tokio::select! {
            _ = sigint.recv() => println!("SIGINT"),
            _ = sigterm.recv() => println!("SIGTERM"),
        }
        shard_manager.lock().await.shutdown_all().await;
    });

    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform
    // exponential backoff until it reconnects.
    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
