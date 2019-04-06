extern crate regex;
extern crate reqwest;

use regex::Regex;
use std::io::Read;
use std::net::TcpStream;
use twitchchat::commands::PrivMsg;
use twitchchat::{Client, Writer, UserConfig, TWITCH_IRC_ADDRESS};

fn main() {
    let read = TcpStream::connect(TWITCH_IRC_ADDRESS).expect("error connecting");
    let write = read
        .try_clone()
        .expect("must be able to clone the tcp stream");

    let config = UserConfig::builder()
        .token("OAUTH")
        .nick("USERNAME")
        .membership()
        .commands()
        .tags()
        .build()
        .expect("partial configuration initialized");

    let mut client = Client::new(read, write);

    client.register(config).unwrap();

    let user = client.wait_for_ready().unwrap();
    println!(
        "connected as {} (id: {})",
        user.display_name.unwrap(),
        user.user_id
    );

    client.on(|msg: PrivMsg, wr: Writer<_>| {
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
                println!("{} !!! Beginning Analysis", msg.message);
                let msg_text = msg.message;

                if msg_text.contains("youtube.com") {
                    println!("Youtube Link Detected");
                    println!("message text: {}", &msg_text);

                    // filter message, build link, request page, read to buffer
                    let youtube_link_regex = Regex::new(r"\?v=([a-zA-Z0-9-]+)").unwrap();
                    let link_cap = youtube_link_regex.captures(&msg_text).unwrap();
                    let video_id = link_cap[0].to_string();
                    let complete_url = format!("https://www.youtube.com/watch?v={}", video_id);
                    let mut response = reqwest::get(&complete_url).expect("error getting page");
                    let mut buffer = String::new();
                    response.read_to_string(&mut buffer);


                    // "lengthSeconds\":\"675\"
                    let youtube_length_regex = Regex::new(r#"lengthSeconds\\":\\"(\d+)\\""#).unwrap();
                    let youtube_length = youtube_length_regex.captures(&buffer).unwrap();
                    let youtube_seconds_length = youtube_length[1].to_string();
                    let converted_length = youtube_seconds_length.parse::<u32>().expect("error parsing string to u32");
                    let final_length = converted_length / 60;

                    // wr.send("CHANNEL", final_length_message).unwrap();  :[{"runs":[{"text":
                    // "simpleText":"Category"},"contents":[{"runs":[{"text":"Film \u0026 Animation"
                    let youtube_views_regex = Regex::new(r#":[{"runs":[{"text": ([a-zA-Z0-9-]+)"#).unwrap();

                }
            }
        }

    });

    let w = client.writer();
    w.join("CHANNEL").unwrap();
    w.send("CHANNEL", "I have arrived!").unwrap();


    if let Err(err) = client.run() {
        println!("error running: {}", err);
        std::process::exit(1);
    }

}