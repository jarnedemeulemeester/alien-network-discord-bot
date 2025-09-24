extern crate rand;

use rand::seq::SliceRandom;
use rand::rng;

use serenity::{
    builder::{CreateChannel, CreateCommand, CreateCommandOption},
    model::application::{CommandDataOptionValue, CommandInteraction, CommandOptionType},
    model::prelude::{ChannelType, GuildChannel},
    prelude::Context,
};

use crate::Handler;

pub async fn run(command: &CommandInteraction, handler: &Handler, ctx: &Context) -> String {
    let options = &command.data.options;

    let n_teams_option = options.get(0).expect("Expected subcommand");

    let n_teams = if let CommandDataOptionValue::Integer(n_teams) = n_teams_option.value {
        n_teams
    } else {
        return "Please provide a valid ID".to_string();
    };

    let channels = handler.guild_id.channels(&ctx.http).await.unwrap();

    let mut members_in_lobby = vec![];

    for channel in channels {
        if channel.1.parent_id.is_some() {
            if channel.1.parent_id.unwrap() == handler.shuffle_category_id {
                if channel.0 != handler.lobby_channel_id {
                    match channel.1.delete(&ctx.http).await {
                        Ok(_r) => {}
                        Err(e) => return e.to_string(),
                    }
                } else {
                    match channel.1.members(&ctx.cache) {
                        Ok(members) => members_in_lobby = members,
                        Err(e) => return e.to_string(),
                    }
                }
            }
        }
    }

    if members_in_lobby.len() < 1 {
        return "There is nobody in the lobby".to_string();
    }

    let mut team_channels: Vec<GuildChannel> = Vec::new();

    for n in 0..n_teams {
        let builder = CreateChannel::new(format!("team {}", n + 1))
            .kind(ChannelType::Voice)
            .category(handler.shuffle_category_id);
        match handler.guild_id.create_channel(&ctx.http, builder).await {
            Ok(channel) => team_channels.push(channel),
            Err(e) => return e.to_string(),
        }
    }

    members_in_lobby.shuffle(&mut rng());

    for (i, members) in members_in_lobby
        .chunks((members_in_lobby.len() as f32 / n_teams as f32).ceil() as usize)
        .enumerate()
    {
        for member in members {
            member
                .move_to_voice_channel(&ctx.http, &team_channels[i])
                .await
                .ok();
        }
    }

    format!("Shuffling into {n_teams} teams").to_string()
}

pub fn register() -> CreateCommand {
    let n_teams_option =
        CreateCommandOption::new(CommandOptionType::Integer, "n_teams", "Number of teams")
            .min_int_value(2)
            .max_int_value(10)
            .required(true);

    CreateCommand::new("shuffle")
        .name("shuffle")
        .description("Shuffle users to different voice channels. Useful for playing against eachother in random teams")
        .add_option(n_teams_option)
}
