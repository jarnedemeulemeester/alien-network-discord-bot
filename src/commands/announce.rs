use regex::Regex;
use serenity::builder::CreateApplicationCommand;
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::prelude::interaction::application_command::{
    CommandDataOption, CommandDataOptionValue,
};
use serenity::prelude::Context;
use serenity::utils::MessageBuilder;

use crate::api::anilist::{self, Media};
use crate::api::tmdb::{get_configuration, get_tv_show, TvShow, Configuration, get_movie, Movie, get_season, Season, get_episode, Episode};
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
            let t = &options
                .get(0)
                .unwrap()
                .options
                .get(0)
                .expect("No type defined")
                .name;

            let main_id = &options
                .get(0)
                .unwrap()
                .options
                .get(0)
                .unwrap()
                .options
                .get(0)
                .expect("No ID specified")
                .resolved
                .as_ref()
                .expect("Unknown error");

            let id = if let CommandDataOptionValue::Integer(id) = main_id {
                    id
                } else {
                    return "Please provide a valid ID".to_string()
                };
            
            let config = match get_configuration().await {
                Ok(config) => config,
                Err(e) => return e,
            };

            match t.as_str() {
                "movie" => {
                    match get_movie(id).await {
                        Ok(movie) => return send_tmdb_movie_announcement(handler, ctx, config, movie).await,
                        Err(e) => return e,
                    }
                }
                "tv_show" => {
                    match get_tv_show(id).await {
                        Ok(tv_show) => return send_tmdb_show_announcement(handler, ctx, config, tv_show).await,
                        Err(e) => return e,
                    }
                }
                "season" => {
                    let season_number_option = &options
                    .get(0)
                    .unwrap()
                    .options
                    .get(0)
                    .unwrap()
                    .options
                    .get(1)
                    .expect("No number specified")
                    .resolved
                    .as_ref()
                    .expect("Unknown error");

                    let season_number = if let CommandDataOptionValue::Integer(number) = season_number_option {
                        number
                    } else {
                        return "Please provide a valid number".to_string()
                    };

                    match get_tv_show(id).await {
                        Ok(tv_show) => {
                            match get_season(id, season_number).await {
                                Ok(season) => return send_tmdb_season_announcement(handler, ctx, config, tv_show, season).await,
                                Err(e) => return e,
                            }
                        },
                        Err(e) => return e,
                    }
                }
                "episode" => {
                    let season_number_option = &options
                    .get(0)
                    .unwrap()
                    .options
                    .get(0)
                    .unwrap()
                    .options
                    .get(1)
                    .expect("No number specified")
                    .resolved
                    .as_ref()
                    .expect("Unknown error");

                    let season_number = if let CommandDataOptionValue::Integer(number) = season_number_option {
                        number
                    } else {
                        return "Please provide a valid number".to_string()
                    };

                    let episode_number_option = &options
                    .get(0)
                    .unwrap()
                    .options
                    .get(0)
                    .unwrap()
                    .options
                    .get(2)
                    .expect("No number specified")
                    .resolved
                    .as_ref()
                    .expect("Unknown error");

                    let episode_number = if let CommandDataOptionValue::Integer(number) = episode_number_option {
                        number
                    } else {
                        return "Please provide a valid number".to_string()
                    };

                    match get_tv_show(id).await {
                        Ok(tv_show) => {
                            match get_season(id, season_number).await {
                                Ok(season) => {
                                    match get_episode(id, season_number, episode_number).await {
                                        Ok(episode) => return send_tmdb_episode_announcement(handler, ctx, config, tv_show, season, episode).await,
                                        Err(e) => return e,
                                    }
                                },
                                Err(e) => return e,
                            }
                        },
                        Err(e) => return e,
                    }
                }
                _ => {}
            }
            "Please provide a valid ID".to_string()
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
                .kind(CommandOptionType::SubCommandGroup)
                .create_sub_option(|suboption| {
                    suboption
                    .name("movie")
                    .description("Announce a new movie on Jellyfin")
                    .kind(CommandOptionType::SubCommand)
                    .create_sub_option(|suboption| {
                        suboption
                            .name("id")
                            .description("Id of the item on TMDB")
                            .kind(CommandOptionType::Integer)
                            .required(true)
                    })
                })
                .create_sub_option(|suboption| {
                    suboption
                    .name("tv_show")
                    .description("Announce a new TV show on Jellyfin")
                    .kind(CommandOptionType::SubCommand)
                    .create_sub_option(|suboption| {
                        suboption
                            .name("id")
                            .description("Id of the item on TMDB")
                            .kind(CommandOptionType::Integer)
                            .required(true)
                    })
                })
                .create_sub_option(|suboption| {
                    suboption
                    .name("season")
                    .description("Announce a new season on Jellyfin")
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
                            .required(true)
                    })
                })
                .create_sub_option(|suboption| {
                    suboption
                    .name("episode")
                    .description("Announce a new episode on Jellyfin")
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
                            .required(true)
                    })
                    .create_sub_option(|suboption| {
                        suboption
                            .name("episode_number")
                            .description("Episode number")
                            .kind(CommandOptionType::Integer)
                            .required(true)
                    })
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

async fn send_tmdb_movie_announcement(handler: &Handler, ctx: &Context, config: Configuration, movie: Movie) -> String {
    let message_sent = handler
        .jellyfin_announcements_channel
        .send_message(&ctx.http, |message| {
            message.embed(|e| {
                e.title(movie.title + " is now available on Jellyfin!")
                    .description(movie.overview)
                    .image(format!("{}original{}", config.images.secure_base_url, movie.poster_path))
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

async fn send_tmdb_season_announcement(handler: &Handler, ctx: &Context, config: Configuration, tv_show: TvShow, season: Season) -> String {
    let message_sent = handler
        .jellyfin_announcements_channel
        .send_message(&ctx.http, |message| {
            message.embed(|e| {
                e.title(format!("{} {} is now available on Jellyfin!", tv_show.name, season.name))
                    .description(season.overview)
                    .image(format!("{}original{}", config.images.secure_base_url, season.poster_path))
                    .thumbnail(format!("{}original{}", config.images.secure_base_url, tv_show.poster_path))
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

async fn send_tmdb_episode_announcement(handler: &Handler, ctx: &Context, config: Configuration, tv_show: TvShow, season: Season, episode: Episode) -> String {
    let message_sent = handler
        .jellyfin_announcements_channel
        .send_message(&ctx.http, |message| {
            message.embed(|e| {
                e.title(format!("{} {} Episode {} is now available on Jellyfin!", tv_show.name, season.name, episode.episode_number))
                    .description(episode.overview)
                    .image(format!("{}original{}", config.images.secure_base_url, episode.still_path))
                    .thumbnail(format!("{}original{}", config.images.secure_base_url, season.poster_path))
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