use serenity::builder::CreateApplicationCommand;
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::prelude::interaction::application_command::{
    CommandDataOption, CommandDataOptionValue,
};

pub fn run(options: &[CommandDataOption]) -> String {
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
                return anilist_lookup(&id);
            } else {
                "Please provide a valid ID".to_string()
            }
        }
        "tmdb" => "Not implemented yet".to_string(),
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
        })
}

fn anilist_lookup(id: &i64) -> String {
    "Yes".to_string()
}
