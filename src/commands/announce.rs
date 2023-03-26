use regex::Regex;
use serenity::builder::CreateApplicationCommand;
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::prelude::interaction::application_command::{
    CommandDataOption, CommandDataOptionValue,
};
use serenity::prelude::Context;
use serenity::utils::MessageBuilder;

use crate::api::anilist::{self, Media};
use crate::api::tmdb::{get_configuration, get_tv_show, TvShow, Configuration};
use crate::utils::decode_hex;
use crate::Handler;

pub async fn run(options: &[CommandDataOption], handler: &Handler, ctx: &Context) -> String {
    let subcommand = &options.get(0).expect("Expected subcommand").name;

    match subcommand.as_str() {
        "anilist" => {
            let option = &options
                .get(0)
                .unwrap()
                .options
                .get(0)
                .expect("No ID specified")
                .resolved
                .as_ref()
                .expect("Unknown error");

            if let CommandDataOptionValue::Integer(id) = option {
                let api_response = anilist::get_data(id).await;
                match api_response {
                    Ok(media) => send_anilist_announcement(handler, ctx, media).await,
                    Err(e) => e,
                }
            } else {
                "Please provide a valid ID".to_string()
            }
        }
        "tmdb" => {
            let option_id = &options
                .get(0)
                .unwrap()
                .options
                .get(0)
                .expect("No ID specified")
                .resolved
                .as_ref()
                .expect("Unknown error");

            let option_season = &options.get(0).unwrap().options.get(1);

            if option_season.is_some() {
                let season = option_season.unwrap().resolved.as_ref().unwrap();

                let season = if let CommandDataOptionValue::Integer(season) = season {
                    println!("{}", season);
                    "Announcement sent!".to_string()
                } else {
                    return "Please provide a valid integer".to_string();
                };
            }

            if let CommandDataOptionValue::Integer(id) = option_id {
                let config = match get_configuration().await {
                    Ok(config) => config,
                    Err(e) => return e,
                };

                match get_tv_show(id).await {
                    Ok(tv_show) => send_tmdb_show_announcement(handler, ctx, config, tv_show).await,
                    Err(e) => return e,
                }
            } else {
                "Please provide a valid ID".to_string()
            }
        }
        _ => "Invalid subcommand".to_string(),
    }
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("announce")
        .description("Post an announcement")
        .create_option(|option| {
            option
                .name("anilist")
                .description("Announce a new movie, serie or season on Jellyfin")
                .kind(CommandOptionType::SubCommand)
                .create_sub_option(|suboption| {
                    suboption
                        .name("id")
                        .description("Id of the item on AniList")
                        .kind(CommandOptionType::Integer)
                        .required(true)
                })
        })
        .create_option(|option| {
            option
                .name("tmdb")
                .description("Announce a new movie, series, season or episode on Jellyfin")
                .kind(CommandOptionType::SubCommand)
                .create_sub_option(|suboption| {
                    suboption
                        .name("id")
                        .description("Id of the item on TMDB")
                        .kind(CommandOptionType::Integer)
                        .required(true)
                })
                .create_sub_option(|suboption| {
                    suboption
                        .name("season_number")
                        .description("Season number")
                        .kind(CommandOptionType::Integer)
                })
                .create_sub_option(|suboption| {
                    suboption
                        .name("episode_number")
                        .description("Episode number")
                        .kind(CommandOptionType::Integer)
                })
        })
}

async fn send_anilist_announcement(handler: &Handler, ctx: &Context, media: Media) -> String {
    let regex_html = Regex::new(r"<[^>]*>").unwrap();
    let message_sent = handler
        .jellyfin_announcements_channel
        .send_message(&ctx.http, |message| {
            message.embed(|e| {
                e.title(media.title.english + " is now available on Jellyfin!")
                    .description(regex_html.replace_all(&media.description, ""))
                    .image(media.cover_image.large)
                    .color(decode_hex(&media.cover_image.color))
                    .footer(|f| {
                        f.text("Powered by AniList")
                            .icon_url("https://anilist.co/img/icons/android-chrome-512x512.png")
                    })
            })
        })
        .await;

    match message_sent {
        Ok(_message) => MessageBuilder::new()
            .push("Announcement sent in ")
            .mention(&handler.jellyfin_announcements_channel)
            .build(),
        Err(e) => format!("Cannot post announcement: {}", e).to_string(),
    }
}

async fn send_tmdb_show_announcement(handler: &Handler, ctx: &Context, config: Configuration, tv_show: TvShow) -> String {
    let message_sent = handler
        .jellyfin_announcements_channel
        .send_message(&ctx.http, |message| {
            message.embed(|e| {
                e.title(tv_show.name + " is now available on Jellyfin!")
                    .description(tv_show.overview)
                    .image(format!("{}original{}", config.images.secure_base_url, tv_show.poster_path))
                    .color((13, 37, 63))
                    .footer(|f| {
                        f.text("Powered by TMDB")
                            .icon_url("https://www.themoviedb.org/assets/2/favicon-43c40950dbf3cffd5e6d682c5a8986dfdc0ac90dce9f59da9ef072aaf53aebb3.png")
                    })
            })
        })
        .await;

    match message_sent {
        Ok(_message) => MessageBuilder::new()
            .push("Announcement sent in ")
            .mention(&handler.jellyfin_announcements_channel)
            .build(),
        Err(e) => format!("Cannot post announcement: {e}").to_string(),
    }
}
