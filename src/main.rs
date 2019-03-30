extern crate regex;
extern crate reqwest;

use select::document::Document;
use select::predicate::Name;
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

    //let CHANNEL = "CHANNEL";

    let mut client = Client::new(read, write);

    client.register(config).unwrap();

    let user = client.wait_for_ready().unwrap();
    println!(
        "connected as {} (id: {}). Your username color is: {}",
        user.display_name.unwrap(),
        user.user_id,
        user.color.unwrap_or_default()
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
                    let regexpression = Regex::new(r"\?v=([a-zA-Z0-9-]+)").unwrap();
                    if let Some(cap) = regexpression.captures_iter(&msg_text).next() {
                        println!("beginning capture: {:?}", cap);
                        let video_id = cap[0].to_string();
                        let video_id_copy = cap[1].to_string();
                        let complete_url = format!("https://www.youtube.com/watch?v={}", video_id);
                        let mut response = reqwest::get(&complete_url).expect("error getting page");
                        let mut buffer = String::new();
                        response.read_to_string(&mut buffer);
                        println!("buffer: {:?}", buffer);

                        // thread index out of bounds: the len is 0 but the index is 0' panic after this point
                        let html: &str = &buffer;
                        let document = Document::from(html);
                        let view_count = document.find(Name("a"))
                                                 .filter_map(|n| n.attr(""));
                        let mut text_bucket: Vec<&str> = vec![];
                        for text in view_count {
                            text_bucket.push(text);
                            println!("{}", text);
                        }
                        // panic is right here for text bucket
                        println!("bucket: {}", text_bucket[0]);

                        let final_message = format!("Video Views: {:?}", text_bucket.pop().expect("error popping vector"));
                        //println!("views: {:?}", text_bucket[0]);
                        wr.send("CHANNEL", final_message).unwrap();
                    }

                } else if msg_text.contains("!sr") {
                    println!("Analyzing abnormal request");
                    // scan as link or search
                }
            }
        }

    });

    let w = client.writer();
    w.join("CHANNEL").unwrap();
    w.send("USERNAME", "I have arrived!").unwrap();
    client.run();

}