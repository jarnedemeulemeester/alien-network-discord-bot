use serenity::builder::CreateApplicationCommand;
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::prelude::interaction::application_command::{
    CommandDataOption,
    CommandDataOptionValue
};

pub fn run(options: &[CommandDataOption]) -> String {
    let option = options
        .get(0)
        .expect("Expected subcommand")
        .options
        .get(0)
        .expect("Please provide a valid ID")
        .resolved
        .as_ref()
        .expect("Error decoding");

    if let CommandDataOptionValue::Integer(id) = option {
        format!("Anilist ID: {}", id)
    } else {
        "Please provide a valid ID".to_string()
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

}
