extern crate rand;

use rand::seq::SliceRandom;
use rand::thread_rng;

use serenity::{
    builder::CreateApplicationCommand,
    model::prelude::{
        command::CommandOptionType,
        interaction::application_command::{ApplicationCommandInteraction, CommandDataOptionValue},
        ChannelType, GuildChannel,
    },
    prelude::Context,
};

use crate::Handler;

pub async fn run(
    command: &ApplicationCommandInteraction,
    handler: &Handler,
    ctx: &Context,
) -> String {
    let options = &command.data.options;

    let n_teams_option = options
        .get(0)
        .expect("Expected subcommand")
        .resolved
        .as_ref()
        .unwrap();

    let n_teams = if let CommandDataOptionValue::Integer(n_teams) = n_teams_option {
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
                    match channel.1.members(&ctx.cache).await {
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

    for n in 0..*n_teams {
        match handler
            .guild_id
            .create_channel(&ctx.http, |channel| {
                channel
                    .category(handler.shuffle_category_id)
                    .name(format!("team {}", n + 1))
                    .kind(ChannelType::Voice)
            })
            .await
        {
            Ok(channel) => team_channels.push(channel),
            Err(e) => return e.to_string(),
        }
    }

    members_in_lobby.shuffle(&mut thread_rng());

    for (i, members) in members_in_lobby
        .chunks((members_in_lobby.len() as f32 / *n_teams as f32).ceil() as usize)
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

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("shuffle")
        .description("Shuffle users to different voice channels. Useful for playing against eachother in random teams")
        .create_option(|option| {
            option
                .name("n_teams")
                .description("Number of teams")
                .kind(CommandOptionType::Integer)
                .max_int_value(10)
                .required(true)
        })
}
