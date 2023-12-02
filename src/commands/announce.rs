use regex::Regex;
use serenity::builder::{
    CreateCommand, CreateCommandOption, CreateEmbed, CreateEmbedFooter, CreateMessage,
};
use serenity::model::application::{
    CommandInteraction, CommandOptionType, ResolvedOption, ResolvedValue,
};
use serenity::model::mention::Mention;
use serenity::prelude::Context;
use serenity::utils::MessageBuilder;

use crate::api::anilist::{self, Media};
use crate::api::tmdb::{
    get_configuration, get_episode, get_movie, get_season, get_tv_show, Configuration, Episode,
    Movie, Season, TvShow,
};
use crate::utils::decode_hex;
use crate::Handler;

pub async fn run(command: &CommandInteraction, handler: &Handler, ctx: &Context) -> String {
    let options: &[ResolvedOption] = &command.data.options();

    if command.user.id != handler.admin_user_id {
        return format!(
            "Only {} is allowed to run this command!",
            Mention::from(handler.admin_user_id)
        );
    }

    let subcommand_name = options.get(0).expect("Expected subcommand").name;

    match &options.first().unwrap().value {
        ResolvedValue::SubCommand(options) => match subcommand_name {
            "anilist" => {
                let id_option = options.first().unwrap();
                if let ResolvedValue::Integer(id) = id_option.value {
                    let api_response = anilist::get_data(&id).await;
                    match api_response {
                        Ok(media) => send_anilist_announcement(handler, ctx, media).await,
                        Err(e) => e,
                    }
                } else {
                    "Id must be a integer".to_string()
                }
            }
            _ => "Invalid subcommand".to_string(),
        },
        ResolvedValue::SubCommandGroup(options) => match subcommand_name {
            "tmdb" => {
                let config = match get_configuration().await {
                    Ok(config) => config,
                    Err(e) => return e,
                };
                let subcommand = options.first().unwrap();
                if let ResolvedValue::SubCommand(options) = &subcommand.value {
                    match subcommand.name {
                        "movie" => {
                            let id = if let ResolvedValue::Integer(id) =
                                options.first().unwrap().value
                            {
                                id
                            } else {
                                return "Please provide a valid ID".to_string();
                            };
                            match get_movie(&id).await {
                                Ok(movie) => {
                                    return send_tmdb_movie_announcement(
                                        handler, ctx, config, movie,
                                    )
                                    .await
                                }
                                Err(e) => return e,
                            }
                        }
                        "tv_show" => {
                            let id = if let ResolvedValue::Integer(id) =
                                options.first().unwrap().value
                            {
                                id
                            } else {
                                return "Please provide a valid ID".to_string();
                            };
                            match get_tv_show(&id).await {
                                Ok(tv_show) => {
                                    return send_tmdb_show_announcement(
                                        handler, ctx, config, tv_show,
                                    )
                                    .await
                                }
                                Err(e) => return e,
                            }
                        }
                        "season" => {
                            let id = if let ResolvedValue::Integer(id) =
                                options.first().unwrap().value
                            {
                                id
                            } else {
                                return "Please provide a valid ID".to_string();
                            };
                            let season_number = if let ResolvedValue::Integer(season_number) =
                                options.get(1).unwrap().value
                            {
                                season_number
                            } else {
                                return "Please provide a valid number".to_string();
                            };
                            match get_tv_show(&id).await {
                                Ok(tv_show) => match get_season(&id, &season_number).await {
                                    Ok(season) => {
                                        return send_tmdb_season_announcement(
                                            handler, ctx, config, tv_show, season,
                                        )
                                        .await
                                    }
                                    Err(e) => return e,
                                },
                                Err(e) => return e,
                            }
                        }
                        "episode" => {
                            let id = if let ResolvedValue::Integer(id) =
                                options.first().unwrap().value
                            {
                                id
                            } else {
                                return "Please provide a valid ID".to_string();
                            };
                            let season_number = if let ResolvedValue::Integer(season_number) =
                                options.get(1).unwrap().value
                            {
                                season_number
                            } else {
                                return "Please provide a valid number".to_string();
                            };
                            let episode_number = if let ResolvedValue::Integer(episode_number) =
                                options.get(2).unwrap().value
                            {
                                episode_number
                            } else {
                                return "Please provide a valid number".to_string();
                            };
                            match get_tv_show(&id).await {
                                Ok(tv_show) => match get_season(&id, &season_number).await {
                                    Ok(season) => {
                                        match get_episode(&id, &season_number, &episode_number)
                                            .await
                                        {
                                            Ok(episode) => {
                                                return send_tmdb_episode_announcement(
                                                    handler, ctx, config, tv_show, season, episode,
                                                )
                                                .await
                                            }
                                            Err(e) => return e,
                                        }
                                    }
                                    Err(e) => return e,
                                },
                                Err(e) => return e,
                            }
                        }
                        _ => "Unknown type".to_string(),
                    }
                } else {
                    "No type defined".to_string()
                }
            }
            _ => "Invalid subcommand".to_string(),
        },
        _ => "Invallid command".to_string(),
    }
}

pub fn register() -> CreateCommand {
    let anilist_subcommand_id_option = CreateCommandOption::new(
        CommandOptionType::Integer,
        "id",
        "Id of the item on AniList",
    )
    .required(true);
    let anilist_subcommand = CreateCommandOption::new(
        CommandOptionType::SubCommand,
        "anilist",
        "Announce a new movie, serie or season on Jellyfin",
    )
    .add_sub_option(anilist_subcommand_id_option);
    let tmdb_subcommand_group_id_option =
        CreateCommandOption::new(CommandOptionType::Integer, "id", "Id of the item on TMDB")
            .required(true);
    let tmdb_subcommand_group_season_number_option =
        CreateCommandOption::new(CommandOptionType::Integer, "season_number", "Season number")
            .required(true);
    let tmdb_subcommand_group_episode_number_option = CreateCommandOption::new(
        CommandOptionType::Integer,
        "episode_number",
        "Episode number",
    )
    .required(true);
    let tmdb_subcommand_group_movie_subcommand = CreateCommandOption::new(
        CommandOptionType::SubCommand,
        "movie",
        "Announce a new movie on Jellyfin",
    )
    .add_sub_option(tmdb_subcommand_group_id_option.clone());
    let tmdb_subcommand_group_tv_show_subcommand = CreateCommandOption::new(
        CommandOptionType::SubCommand,
        "tv_show",
        "Announce a new TV show on Jellyfin",
    )
    .add_sub_option(tmdb_subcommand_group_id_option.clone());
    let tmdb_subcommand_group_season_subcommand = CreateCommandOption::new(
        CommandOptionType::SubCommand,
        "season",
        "Announce a new season on Jellyfin",
    )
    .add_sub_option(tmdb_subcommand_group_id_option.clone())
    .add_sub_option(tmdb_subcommand_group_season_number_option.clone());
    let tmdb_subcommand_group_episode_subcommand = CreateCommandOption::new(
        CommandOptionType::SubCommand,
        "episode",
        "Announce a new episode on Jellyfin",
    )
    .add_sub_option(tmdb_subcommand_group_id_option)
    .add_sub_option(tmdb_subcommand_group_season_number_option)
    .add_sub_option(tmdb_subcommand_group_episode_number_option);
    let tmdb_subcommand_group = CreateCommandOption::new(
        CommandOptionType::SubCommandGroup,
        "tmdb",
        "Announce a new movie, series, season or episode on Jellyfin",
    )
    .add_sub_option(tmdb_subcommand_group_movie_subcommand)
    .add_sub_option(tmdb_subcommand_group_tv_show_subcommand)
    .add_sub_option(tmdb_subcommand_group_season_subcommand)
    .add_sub_option(tmdb_subcommand_group_episode_subcommand);
    CreateCommand::new("announce")
        .description("Post an announcement")
        .add_option(anilist_subcommand)
        .add_option(tmdb_subcommand_group)
}

async fn send_anilist_announcement(handler: &Handler, ctx: &Context, media: Media) -> String {
    let regex_html = Regex::new(r"<[^>]*>").unwrap();
    let embed_footer = CreateEmbedFooter::new("Powered by AniList")
        .icon_url("https://anilist.co/img/icons/android-chrome-512x512.png");
    let embed = CreateEmbed::new()
        .title(format!(
            "{} is now available on Jellyfin!",
            media.title.english
        ))
        .description(regex_html.replace_all(&media.description, ""))
        .image(media.cover_image.large)
        .color(decode_hex(&media.cover_image.color))
        .footer(embed_footer);
    let message = CreateMessage::new().embed(embed);
    let message_sent = handler
        .jellyfin_announcements_channel_id
        .send_message(&ctx.http, message)
        .await;

    match message_sent {
        Ok(_message) => MessageBuilder::new()
            .push("Announcement sent in ")
            .mention(&handler.jellyfin_announcements_channel_id)
            .build(),
        Err(e) => format!("Cannot post announcement: {}", e).to_string(),
    }
}

async fn send_tmdb_show_announcement(
    handler: &Handler,
    ctx: &Context,
    config: Configuration,
    tv_show: TvShow,
) -> String {
    let embed_footer = CreateEmbedFooter::new("Powered by TMDB")
    .icon_url("https://www.themoviedb.org/assets/2/favicon-43c40950dbf3cffd5e6d682c5a8986dfdc0ac90dce9f59da9ef072aaf53aebb3.png");
    let embed = CreateEmbed::new()
        .title(format!("{} is now available on Jellyfin!", tv_show.name))
        .description(tv_show.overview)
        .image(format!(
            "{}original{}",
            config.images.secure_base_url, tv_show.poster_path
        ))
        .color((13, 37, 63))
        .footer(embed_footer);
    let message = CreateMessage::new().embed(embed);
    let message_sent = handler
        .jellyfin_announcements_channel_id
        .send_message(&ctx.http, message)
        .await;

    match message_sent {
        Ok(_message) => MessageBuilder::new()
            .push("Announcement sent in ")
            .mention(&handler.jellyfin_announcements_channel_id)
            .build(),
        Err(e) => format!("Cannot post announcement: {e}").to_string(),
    }
}

async fn send_tmdb_movie_announcement(
    handler: &Handler,
    ctx: &Context,
    config: Configuration,
    movie: Movie,
) -> String {
    let embed_footer = CreateEmbedFooter::new("Powered by TMDB")
    .icon_url("https://www.themoviedb.org/assets/2/favicon-43c40950dbf3cffd5e6d682c5a8986dfdc0ac90dce9f59da9ef072aaf53aebb3.png");
    let embed = CreateEmbed::new()
        .title(format!("{} is now available on Jellyfin!", movie.title))
        .description(movie.overview)
        .image(format!(
            "{}original{}",
            config.images.secure_base_url, movie.poster_path
        ))
        .color((13, 37, 63))
        .footer(embed_footer);
    let message = CreateMessage::new().embed(embed);
    let message_sent = handler
        .jellyfin_announcements_channel_id
        .send_message(&ctx.http, message)
        .await;

    match message_sent {
        Ok(_message) => MessageBuilder::new()
            .push("Announcement sent in ")
            .mention(&handler.jellyfin_announcements_channel_id)
            .build(),
        Err(e) => format!("Cannot post announcement: {e}").to_string(),
    }
}

async fn send_tmdb_season_announcement(
    handler: &Handler,
    ctx: &Context,
    config: Configuration,
    tv_show: TvShow,
    season: Season,
) -> String {
    let embed_footer = CreateEmbedFooter::new("Powered by TMDB")
    .icon_url("https://www.themoviedb.org/assets/2/favicon-43c40950dbf3cffd5e6d682c5a8986dfdc0ac90dce9f59da9ef072aaf53aebb3.png");
    let embed = CreateEmbed::new()
        .title(format!(
            "{} {} is now available on Jellyfin!",
            tv_show.name, season.name
        ))
        .description(season.overview)
        .image(format!(
            "{}original{}",
            config.images.secure_base_url, season.poster_path
        ))
        .thumbnail(format!(
            "{}original{}",
            config.images.secure_base_url, tv_show.poster_path
        ))
        .color((13, 37, 63))
        .footer(embed_footer);
    let message = CreateMessage::new().embed(embed);
    let message_sent = handler
        .jellyfin_announcements_channel_id
        .send_message(&ctx.http, message)
        .await;

    match message_sent {
        Ok(_message) => MessageBuilder::new()
            .push("Announcement sent in ")
            .mention(&handler.jellyfin_announcements_channel_id)
            .build(),
        Err(e) => format!("Cannot post announcement: {e}").to_string(),
    }
}

async fn send_tmdb_episode_announcement(
    handler: &Handler,
    ctx: &Context,
    config: Configuration,
    tv_show: TvShow,
    season: Season,
    episode: Episode,
) -> String {
    let embed_footer = CreateEmbedFooter::new("Powered by TMDB")
    .icon_url("https://www.themoviedb.org/assets/2/favicon-43c40950dbf3cffd5e6d682c5a8986dfdc0ac90dce9f59da9ef072aaf53aebb3.png");
    let embed = CreateEmbed::new()
        .title(format!(
            "{} {} Episode {} is now available on Jellyfin!",
            tv_show.name, season.name, episode.episode_number
        ))
        .description(episode.overview)
        .image(format!(
            "{}original{}",
            config.images.secure_base_url, episode.still_path
        ))
        .thumbnail(format!(
            "{}original{}",
            config.images.secure_base_url, season.poster_path
        ))
        .color((13, 37, 63))
        .footer(embed_footer);
    let message = CreateMessage::new().embed(embed);
    let message_sent = handler
        .jellyfin_announcements_channel_id
        .send_message(&ctx.http, message)
        .await;

    match message_sent {
        Ok(_message) => MessageBuilder::new()
            .push("Announcement sent in ")
            .mention(&handler.jellyfin_announcements_channel_id)
            .build(),
        Err(e) => format!("Cannot post announcement: {e}").to_string(),
    }
}
