extern crate regex;

use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;
use std::net::TcpStream;
use twitchchat::commands::PrivMsg;
use twitchchat::{Client, Writer, UserConfig, TWITCH_IRC_ADDRESS};

fn main() {
    let read = TcpStream::connect(TWITCH_IRC_ADDRESS).expect("error connecting");
    let write = read
        .try_clone()
        .expect("must be able to clone the tcp stream");

    let config = UserConfig::builder()
        .token("OAUTH_TOKEN")
        .nick("BOT_USERNAME")
        .membership()
        .commands()
        .tags()
        .build()
        .expect("partial configuration initialized");

    let mut client = Client::new(read, write);

    client.register(config).unwrap();

    let user = client.wait_for_ready().unwrap();
    println!(
        "connected as {} (id: {}). Your username color is: {}",
        user.display_name.unwrap(),
        user.user_id,
        user.color.unwrap_or_default()
    );

    client.on(|msg: PrivMsg, _: Writer<_>| {
        let name = msg.display_name().unwrap_or_else(|| msg.user());
        use twitchchat::BadgeKind::{Broadcaster, Subscriber, Moderator};

        println!("{}: {}", name, msg.message());

        let badges = msg
            .badges()
            .iter()
            .map(|badge| badge.kind.clone())
            .collect::<Vec<_>>();

        match (
            badges.contains(&Broadcaster),
            badges.contains(&Subscriber),
            msg.moderator(),
            ) {
            (true, _, _) => println!("{} --> SAFE", name),
            (_, true, _) => println!("{} --> SAFE", name),
            (_, _, true) => println!("{} --> SAFE", name),
            (_, _, _) => {
                println!("{} !!! Beginning Analysis");

                if msg.message.contains("KEYS_FILE") {
                    println!("Filtering");
                    let file = File::open("PATH_TO_FILE").expect("error opening file");
                    let mut read_buffer = BufRead::new(file);
                    let mut contents = String::new();
                    buf_reader.read_to_string(&mut contents);
                    // to be completed
                    // scan buffer
                    // make it optional

                } else if msg.message.contains("youtube.com" | "youtu.be.com") {
                    println!("Youtube Link Detected");
                    // to be completed
                    // request html page
                    // parse html page
                    // respond
                    // gather info

                } else if msg.message.contains("www." | "http" | "://" | "goo.gl") {
                    println!("Link Detected");
                    // to be completed
                    // request html page
                    // parse html page
                    // respond
                    // make it optional

                } else {
                    println!("OK");
                    // to be completed
                }
            }
        }

    });

    let w = client.writer();
    w.join("CHANNEL").unwrap();
    w.send("CHANNEL", "I have arrived!").unwrap();

    client.run();

}