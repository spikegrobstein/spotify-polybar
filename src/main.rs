use rspotify::{AuthCodeSpotify, Credentials, OAuth, Config};
use rspotify::prelude::*;
use rspotify::model::{AdditionalType, SimplifiedArtist, PlayableItem};
use rspotify::scopes;

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
            let playing = spotify.current_playing(None, Some(&[AdditionalType::Track])).await?;

            match playing {
                None => {
                    println!("Nothing.");
                },
                Some(context) => {
                    match context.item {
                        Some(PlayableItem::Track(track)) => {
                            println!("{}: {}", render_artist(track.artists), track.name);
                        },
                        _ => {
                            println!("Nothing.");
                        }
                    }
                }
            }
        },
        ("playpause", Some(matches)) => {
            let playing = spotify.current_playing(None, Some(&[AdditionalType::Track])).await?;

            let device_id = matches.value_of("device_id");

            match playing {
                None => {
                    spotify.resume_playback(device_id, None).await?;
                },
                Some(context) => {
                    if context.is_playing {
                        spotify.pause_playback(device_id).await?;
                    } else {
                        spotify.resume_playback(device_id, None).await?;
                    }
                }
            }

        },
        ("play-button", Some(matches)) => {
            let is_playing = get_is_playing(&spotify).await?;

            let button = match is_playing {
                Some(true) => "pause",
                Some(false) => "play",
                None => "disabled",
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
            for device in devices {
                println!("{} {} {:?}", device.name, device.id.unwrap_or_default(), device.is_active);
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

async fn get_is_playing(spotify: &AuthCodeSpotify) -> Result<Option<bool>, Box<dyn std::error::Error>> {
    let playing = spotify.current_playing(None, Some(&[AdditionalType::Track])).await?;

    match playing {
        None => Ok(None),
        Some(context) => Ok(Some(context.is_playing)),
    }
}

fn render_artist(artists: Vec<SimplifiedArtist>) -> String {
    artists.iter().map(|artist| {
        artist.name.clone()
    })
    .collect::<Vec<String>>()
    .join(", ")
}

async fn get_spotify_client() -> Result<AuthCodeSpotify, Box<dyn std::error::Error>> {
    let home_path = env::var("HOME").unwrap_or("./".to_string());

    let mut token_cache_file = PathBuf::from(home_path);
    token_cache_file.push(".spotify_polybar_token_cache.json");

    let creds = Credentials::new(
        "XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX",
        "XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX", // this has been reset and has to be populated
    );

    let oauth = OAuth {
        redirect_uri: "https://localhost.apple.com:8888/callback".to_string(),
        scopes: scopes!(
            "user-read-playback-state",
            "user-modify-playback-state",
            "user-read-private"
        ),
        ..Default::default()
    };

    let config = Config {
        cache_path: token_cache_file,
        token_cached: true,
        ..Default::default()
    };

    let spotify = AuthCodeSpotify::with_config(creds, oauth, config);

    let url = spotify.get_authorize_url(false)?;
    spotify.prompt_for_token(&url).await?;

    Ok(spotify)
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
                              .value_name("text")
                              .long("play")
                              .help("The button for triggering play")
                              .takes_value(true)
                              .default_value("play")
                          )
                          .arg(Arg::with_name("pause")
                              .value_name("text")
                              .long("pause")
                              .help("The button for triggering pause")
                              .takes_value(true)
                              .default_value("pause")
                          )
                          .arg(Arg::with_name("disabled")
                              .value_name("text")
                              .long("disabled")
                              .help("The button for disabled")
                              .takes_value(true)
                              .default_value("disabled")
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
