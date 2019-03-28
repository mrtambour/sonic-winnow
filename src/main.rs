extern crate regex;

use std::net::TcpStream;
use twitchchat::commands::PrivMsg;
use twitchchat::{Client, Writer, UserConfig, TWITCH_IRC_ADDRESS};

fn main() {
    let read = TcpStream::connect(TWITCH_IRC_ADDRESS).expect("to connect");
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
        .expect("partial config initialized");

    let mut client = Client::new(read, write);

    client.register(config).unwrap();

    let user = client.wait_for_ready().unwrap();
    println!(
        "connected with {} (id: {}). Your username color is: {}",
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

                if msg.message.contains("KEYS") {
                    println!("Filtering");
                    // to be completed
                } else if msg.message.contains("youtube.com" | "youtu.be.com") {
                    println!("Youtube Link Detected");
                    // to be completed

                } else if msg.message.contains("www." | "http" | "://" | "goo.gl") {
                    println!(" ");
                } else {
                    println!("");
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
