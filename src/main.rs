use rspotify::client::Spotify;
use rspotify::oauth2::{SpotifyClientCredentials, SpotifyOAuth};
use rspotify::util::get_token;
use rspotify::model::artist::SimplifiedArtist;

use anyhow::{Result, anyhow};

use std::env;
use std::path::PathBuf;

extern crate clap;
use clap::{Arg, App, SubCommand};

#[tokio::main]
async fn main() {
    let matches = get_cli_app().get_matches();

    let spotify = match get_spotify_client().await {
        Ok(spotify) => spotify,
        Err(error) => {
            println!("Error.");
            eprintln!("Error: {}", error);
            std::process::exit(1);
        }
    };

    match matches.subcommand() {
        ("status", Some(_matches)) => {
            let playing = match spotify.current_user_playing_track().await {
                Ok(playing) => playing,
                Err(error) => {
                    println!("Error.");
                    eprintln!("Error: {}", error);
                    std::process::exit(1);
                }
            };

            match playing {
                None => {
                    println!("Nothing.");
                },
                Some(playing) => {
                    let item = playing.item.unwrap();
                    println!("{}: {}", render_artist(item.artists), item.name);
                }
            }
        },
        ("playpause", Some(_matches)) => {
            spotify.pause_playback(None).await.unwrap();
        },
        ("next", Some(_matches)) => {
            spotify.next_track(None).await.unwrap();
        },
        ("previous", Some(_matches)) => {
            spotify.previous_track(None).await.unwrap();
        },
        ("players", Some(_matches)) => {
            let devices = spotify.device().await.unwrap();
            for device in devices.devices {
                println!("{} {} {:?}", device.name, device.id, device.is_active);
            }
        },
        ("", None) => {
            eprintln!("Missing subcommand.");
            get_cli_app()
                .print_long_help()
                .unwrap();
            std::process::exit(1);
        },
        _ => unreachable!(),
    }
}

fn render_artist(artists: Vec<SimplifiedArtist>) -> String {
    artists.iter().map(|artist| {
        artist.name.clone()
    })
    .collect::<Vec<String>>()
    .join(", ")
}

async fn get_spotify_client() -> Result<Spotify> {
    let home_path = env::var("HOME").unwrap_or("./".to_string());

    let mut token_cache_file = PathBuf::from(home_path);
    token_cache_file.push(".spotify_polybar_token_cache.json");

    let mut oauth = SpotifyOAuth::default()
        .client_id("4abb24ee71384d518e0bb9e3d54b8241")
        .client_secret("XXXXXXXXXXXXXXX") // this has been reset and has to be populated
        .redirect_uri("http://localhost:8888/callback")
        .scope("user-read-playback-state user-modify-playback-state user-read-private")
        .cache_path(token_cache_file)
        .build();

    match get_token(&mut oauth).await {
        Some(token_info) => {
            let client_credential = SpotifyClientCredentials::default()
                .token_info(token_info)
                .build();

            Ok(Spotify::default()
                .client_credentials_manager(client_credential)
                .build())
        },
        None => {
            eprintln!("error.");
            std::process::exit(1);
        }
    }
}

fn get_cli_app() -> App<'static, 'static> {
    App::new("spotify-polybar")
              .version("1.0")
              .author("Spike Grobstein <me@spike.cx>")
              .about("Does awesome things")
              .subcommand(SubCommand::with_name("status")
                          .about("Output current track info")
              )
              .subcommand(SubCommand::with_name("playpause")
                          .about("Toggle play/pause")
              )
              .subcommand(SubCommand::with_name("next")
                          .about("Next track")
              )
              .subcommand(SubCommand::with_name("previous")
                          .about("Previous track")
              )
              .subcommand(SubCommand::with_name("players")
                          .about("List available players")
              )
}
