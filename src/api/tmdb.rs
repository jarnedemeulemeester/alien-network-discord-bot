use std::env;

use reqwest::header::AUTHORIZATION;
use serde::Deserialize;


#[derive(Debug, Deserialize)]
pub struct Configuration {
    pub images: ImagesConfiguration
}

#[derive(Debug, Deserialize)]
pub struct ImagesConfiguration {
    pub base_url: String,
    pub secure_base_url: String,
    pub backdrop_sizes: Vec<String>,
    pub logo_sizes: Vec<String>,
    pub poster_sizes: Vec<String>,
    pub profile_sizes: Vec<String>,
    pub still_sizes: Vec<String>,
}


pub async fn get_configuration() -> Result<Configuration, String> {
    let endpoint = "https://api.themoviedb.org/3/configuration";

    let tmdb_token = env::var("TMDB_TOKEN")
        .expect("Expected JELLYFIN_ANNOUNCEMENTS_CHANNEL_ID in environment");

    let client = reqwest::Client::new();

    let resp = 
        client
        .get(endpoint)
        .header(AUTHORIZATION, format!("Bearer {}", tmdb_token))
        .send()
        .await
        .unwrap()
        .json::<Configuration>()
        .await;

    match resp {
        Ok(config) => Ok(config),
        Err(e) => Err(e.to_string())
    }
}

#[derive(Debug, Deserialize)]
pub struct TvShow {
    pub name: String,
    pub overview: String,
    pub poster_path: String,
}

pub async fn get_tv_show(id: &i64) -> Result<TvShow, String> {
    let endpoint = format!("https://api.themoviedb.org/3/tv/{id}");

    let tmdb_token = env::var("TMDB_TOKEN")
        .expect("Expected JELLYFIN_ANNOUNCEMENTS_CHANNEL_ID in environment");

    let client = reqwest::Client::new();

    let resp = 
        client
        .get(endpoint)
        .header(AUTHORIZATION, format!("Bearer {}", tmdb_token))
        .send()
        .await
        .unwrap()
        .json::<TvShow>()
        .await;

    match resp {
        Ok(tv_show) => Ok(tv_show),
        Err(e) => Err(e.to_string())
    }
}

#[derive(Debug, Deserialize)]
pub struct Movie {
    pub title: String,
    pub overview: String,
    pub poster_path: String,
}

pub async fn get_movie(id: &i64) -> Result<Movie, String> {
    let endpoint = format!("https://api.themoviedb.org/3/movie/{id}");

    let tmdb_token = env::var("TMDB_TOKEN")
        .expect("Expected JELLYFIN_ANNOUNCEMENTS_CHANNEL_ID in environment");

    let client = reqwest::Client::new();

    let resp = 
        client
        .get(endpoint)
        .header(AUTHORIZATION, format!("Bearer {}", tmdb_token))
        .send()
        .await
        .unwrap()
        .json::<Movie>()
        .await;

    match resp {
        Ok(movie) => Ok(movie),
        Err(e) => Err(e.to_string())
    }
}