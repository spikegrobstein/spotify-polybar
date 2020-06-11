use rspotify::client::Spotify;
use rspotify::oauth2::{SpotifyClientCredentials, SpotifyOAuth};
use rspotify::util::get_token;
use rspotify::model::artist::SimplifiedArtist;

// use anyhow::{Result, anyhow};

use std::env;
use std::path::PathBuf;

extern crate clap;
use clap::{Arg, App, SubCommand};

#[tokio::main]
async fn main() {
    let matches = get_cli_app().get_matches();
    
    match handle(matches).await {
        Ok(_) => {},
        Err(error) => {
            println!("Error.");
            eprintln!("Error: {}", error);
            std::process::exit(1);
        }
    }

}

async fn handle(matches: clap::ArgMatches<'_>) -> Result<(), Box<dyn std::error::Error>> {
    let spotify = get_spotify_client().await?;

    match matches.subcommand() {
        ("status", Some(_matches)) => {
            let playing = spotify.current_user_playing_track().await?;

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
        ("playpause", Some(matches)) => {
            let playing = spotify.current_user_playing_track().await?;

            let device_id = matches.value_of("device_id").map(|d| d.to_owned());

            match playing {
                None => {
                    spotify.start_playback(device_id, None, None, None, None).await?;
                },
                Some(playing) => {
                    if playing.is_playing {
                        spotify.pause_playback(device_id).await?;
                    } else {
                        spotify.start_playback(device_id, None, None, None, None).await?;
                    }
                }
            }

        },
        ("play-button", Some(matches)) => {
            let is_playing = get_is_playing(&spotify).await?;

            let button = match is_playing {
                true => "pause",
                false => "play",
            };

            println!("{}", matches.value_of(button).unwrap());
        },
        ("next", Some(_matches)) => {
            spotify.next_track(None).await?;
        },
        ("previous", Some(_matches)) => {
            spotify.previous_track(None).await?;
        },
        ("players", Some(_matches)) => {
            let devices = spotify.device().await?;
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
    };

    Ok(())
}

async fn get_is_playing(spotify: &Spotify) -> Result<bool, Box<dyn std::error::Error>> {
    let playing = spotify.current_user_playing_track().await?;

    match playing {
        None => Ok(false),
        Some(playing) => Ok(playing.is_playing),
    }
}

fn render_artist(artists: Vec<SimplifiedArtist>) -> String {
    artists.iter().map(|artist| {
        artist.name.clone()
    })
    .collect::<Vec<String>>()
    .join(", ")
}

async fn get_spotify_client() -> Result<Spotify, Box<dyn std::error::Error>> {
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
                          .arg(Arg::with_name("device_id")
                                .long("device-id")
                                .short("d")
                                .help("ID of target device")
                                .takes_value(true)
                          )
              )
              .subcommand(SubCommand::with_name("play-button")
                          .about("Output the play button")
                          .arg(Arg::with_name("play")
                              .long("play")
                              .help("The button for triggering play")
                              .takes_value(true)
                              .default_value("play")
                          )
                          .arg(Arg::with_name("pause")
                              .long("pause")
                              .help("The button for triggering pause")
                              .takes_value(true)
                              .default_value("pause")
                          )
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
