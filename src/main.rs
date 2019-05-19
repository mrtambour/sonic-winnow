extern crate regex;
extern crate reqwest;

use regex::Regex;
use std::io::Read;
use std::net::TcpStream;
use twitchchat::commands::PrivMsg;
use twitchchat::{Client, SyncReadAdapter, UserConfig, Writer, TWITCH_IRC_ADDRESS};

fn main() {
    static USERNAME: &str = "USERNAME";
    static CHANNEL: &str = "CHANNEL";
    static OAUTH: &str = "OAUTH";
    static GET_VIDEO_LENGTH: bool = false;
    static GET_VIDEO_VIEWS: bool = true;

    let read = TcpStream::connect(TWITCH_IRC_ADDRESS).expect("error connecting");
    let write = read
        .try_clone()
        .expect("must be able to clone the tcp stream");

    let config = UserConfig::builder()
        .token(OAUTH)
        .nick(USERNAME)
        .membership()
        .commands()
        .tags()
        .build()
        .expect("partial configuration failed");

    let read = SyncReadAdapter::new(read);

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
        use twitchchat::BadgeKind::{Broadcaster, Moderator, Subscriber};

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
                println!("User: {} {} -> Analyzing...", name, msg.message());
                let msg_text = msg.message().to_string();

                if msg_text.contains("youtube.com") {
                    println!("Youtube Link Detected");
                    println!("message text: {}", &msg_text);

                    // filter message, build link, request page, read to buffer
                    let youtube_link_regex = Regex::new(r#"\?v=([a-zA-Z0-9-_]+)"#).unwrap();
                    let link_cap = youtube_link_regex.captures(&msg_text).unwrap();
                    let video_id = link_cap[0].to_string();
                    let complete_url = format!("https://www.youtube.com/watch{}", video_id);
                    println!("page got: {}", complete_url);
                    let mut response = reqwest::get(&complete_url).expect("error getting page");
                    let mut buffer = String::new();
                    response
                        .read_to_string(&mut buffer)
                        .expect("error writing to buffer");

                    let search_options = (GET_VIDEO_LENGTH, GET_VIDEO_VIEWS);

                    match search_options {
                        (true, true) => {
                            get_length(&buffer, &CHANNEL, &wr);
                            get_views(&buffer, &CHANNEL, &wr)
                        }
                        (true, false) => {
                            get_length(&buffer, &CHANNEL, &wr);
                        }
                        (false, true) => {
                            get_views(&buffer, &CHANNEL, &wr);
                        }
                        (false, false) => {
                            println!("No search option selected using default: Views");
                            get_views(&buffer, &CHANNEL, &wr);
                        }
                    };
                }
            }
        }
    });

    let w = client.writer();
    w.join(CHANNEL).unwrap();
    w.send(CHANNEL, "I have arrived!").unwrap();

    if let Err(err) = client.run() {
        println!("error running: {}", err);
        std::process::exit(1);
    }
}

fn get_length(buffer: &str, CHANNEL: &str, wr: &Writer<TcpStream>) {
    // "lengthSeconds\":\"675\"
    let youtube_length_regex = Regex::new(r#"","length_seconds":"([0-9]+)"#).unwrap();
    let youtube_length = youtube_length_regex.captures(&buffer).unwrap();
    //println!("buffer contents: {}", &buffer);
    let youtube_seconds_length = youtube_length[1].to_string();
    let converted_length = youtube_seconds_length
        .parse::<u32>()
        .expect("error parsing string to u32");
    let minutes_count = converted_length / 60;
    let seconds_count = converted_length as f64 % 60 as f64;
    //let final_length = converted_length as f64 / 60 as f64;
    let final_length_message = format!("Video length: {}:{:02}", minutes_count, seconds_count);
    wr.send(CHANNEL, final_length_message).unwrap();
}

fn get_views(buffer: &str, CHANNEL: &str, wr: &Writer<TcpStream>) {
    //"shortViewCount":{"simpleText":
    let youtube_views_regex = Regex::new(r#",\\"viewCount\\":\\"([0-9]+)"#).unwrap();
    let youtube_views = youtube_views_regex.captures(&buffer).unwrap();
    let final_views = youtube_views[1].to_string();
    let final_views_message = format!("Total views: {}", final_views);
    wr.send(CHANNEL, final_views_message).unwrap();
}

// "simpleText":"Category"},"contents":[{"runs":[{"text":"Film \u0026 Animation"
//                    let youtube_categories_regex =
//                        Regex::new(r#":\[\{"runs":\[\{"text":"([a-zA-Z0-9-\\]+)"#).unwrap();
//                    let youtube_categories_found =
//                        youtube_categories_regex.captures(&buffer).unwrap();
//                    let final_category = youtube_categories_found[1].to_string();
//                    let final_category_message = format!("Video Category: {}", final_category);
//                    wr.send(CHANNEL, final_category_message).unwrap();
