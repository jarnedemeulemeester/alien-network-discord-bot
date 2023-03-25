mod api;
mod commands;
mod utils;

use std::env;
use regex::Regex;

use serenity::async_trait;
use serenity::model::application::interaction::{Interaction, InteractionResponseType};
use serenity::model::gateway::Ready;
use serenity::model::prelude::{GuildId, ChannelId};
use serenity::prelude::*;
use serenity::utils::MessageBuilder;

use crate::utils::decode_hex;

struct Handler {
    jellyfin_announcements_channel: ChannelId,
}

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            println!(
                "Received command interaction: {:#?} from {:#?}",
                command.data.name, command.user.name
            );

            let command_result = match command.data.name.as_str() {
                "announce" => commands::announce::run(&command.data.options).await,
                _ => Err("not implemented".to_string()),
            };

            match command_result {
                Ok(media) => {
                    let regex_html = Regex::new(r"<[^>]*>").unwrap();
                    if let Err(why) = self.jellyfin_announcements_channel.send_message(&ctx.http, |message| {
                        message
                            .embed(|e| {
                                e
                                .title(media.title.english + " is now available on Jellyfin!")
                                .description(regex_html.replace_all(&media.description, ""))
                                .image(media.cover_image.large)
                                .color(decode_hex(&media.cover_image.color))
                                .footer(|f| {
                                    f
                                    .text("Powered by AniList")
                                    .icon_url("https://anilist.co/img/icons/android-chrome-512x512.png")
                                })
                            })
                    })
                    .await
                    {
                        println!("Cannot post announcement: {}", why);
                    }

                    if let Err(why) = command
                        .create_interaction_response(&ctx.http, |response| {
                            response
                                .kind(InteractionResponseType::ChannelMessageWithSource)
                                .interaction_response_data(|message| {
                                    message
                                    .content(
                                        MessageBuilder::new()
                                        .push("Announcement sent in ")
                                        .mention(&self.jellyfin_announcements_channel)
                                        .build()
                                    )
                                    .ephemeral(true)
                                })
                        })
                        .await
                    {
                        println!("Cannot respond to slash command: {}", why);
                    }
                }
                Err(e) => {
                    if let Err(why) = command
                        .create_interaction_response(&ctx.http, |response| {
                            response
                                .kind(InteractionResponseType::ChannelMessageWithSource)
                                .interaction_response_data(|message| {
                                    message
                                    .content(e)
                                    .ephemeral(true)
                                })
                        })
                        .await
                    {
                        println!("Cannot respond to slash command: {}", why);
                    }
                }
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        let guild_id = GuildId(
            env::var("GUILD_ID")
                .expect("Expected GUILD_ID in environment")
                .parse()
                .expect("GUILD_ID must be an integer"),
        );

        let commands = GuildId::set_application_commands(&guild_id, &ctx.http, |commands| {
            commands.create_application_command(|command| commands::announce::register(command))
        })
        .await;

        println!("I created the following slash command: {:#?}", commands);
    }
}

#[tokio::main]
async fn main() {
    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    let jellyfin_announcements_channel = ChannelId(
        env::var("JELLYFIN_ANNOUNCEMENTS_CHANNEL_ID")
            .expect("Expected JELLYFIN_ANNOUNCEMENTS_CHANNEL_ID in environment")
            .parse()
            .expect("JELLYFIN_ANNOUNCEMENTS_CHANNEL_ID must be an integer"),
    );

    // Build our client.
    let mut client = Client::builder(token, GatewayIntents::empty())
        .event_handler(Handler { jellyfin_announcements_channel: jellyfin_announcements_channel })
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
