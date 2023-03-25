use gql_client::{Client};
use serde::{Deserialize, Serialize};


#[derive(Deserialize)]
struct Data {
    #[serde(rename = "Media")]
    media: Media,
}

#[derive(Deserialize)]
pub struct Media {
    pub id: u32,
    pub title: Title,
}

#[derive(Deserialize)]
pub struct Title {
    pub english: String,
}

#[derive(Serialize)]
pub struct Vars {
    id: i64,
}

pub async fn get_data(id: &i64) -> Result<Media, String> {
    let endpoint = "https://graphql.anilist.co/";
    let query = r#"
        query ($id: Int) {
            Media (id: $id, type: ANIME) {
                id
                title {
                    english
                }
            }
        }
    "#;

    let client = Client::new(endpoint);
    let vars = Vars { id: id.clone() };
    let request = client.query_with_vars_unwrap::<Data, Vars>(query, vars).await;

    let response = match request {
        Ok(r) => r,
        Err(_e) => return Err("GraphQL error".to_string()),
    };

    //let media = response.unwrap().media;

    Ok(response.media)
}