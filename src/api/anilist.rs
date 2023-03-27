use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct RequestBody<T: Serialize> {
    query: String,
    variables: T,
}

#[derive(Deserialize)]
struct ResponseBody {
    data: Data,
}

#[derive(Deserialize)]
struct Data {
    #[serde(rename = "Media")]
    media: Media,
}

#[derive(Deserialize)]
pub struct Media {
    pub id: u32,
    pub title: Title,
    pub description: String,
    #[serde(rename = "coverImage")]
    pub cover_image: CoverImage,
}

#[derive(Deserialize)]
pub struct Title {
    pub english: String,
}

#[derive(Deserialize)]
pub struct CoverImage {
    pub large: String,
    pub color: String,
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
                description (asHtml: false)
                coverImage {
                    large
                    color
                }
            }
        }
    "#;

    let variables = Vars { id: id.clone() };

    let client = reqwest::Client::new();

    let request_body = RequestBody {
        query: query.to_string(),
        variables,
    };

    let resp = client
        .post(endpoint)
        .json(&request_body)
        .send()
        .await
        .unwrap()
        .json::<ResponseBody>()
        .await;

    match resp {
        Ok(response_body) => Ok(response_body.data.media),
        Err(e) => Err(e.to_string()),
    }
}
