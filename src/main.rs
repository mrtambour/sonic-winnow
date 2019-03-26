use std::net::TcpStream;
use twitchchat::commands::PrivMsg;
use twitchchat::{Client, Writer, UserConfig, TWITCH_IRC_ADDRESS};

fn main() {
    let read = TcpStream::connect(TWITCH_IRC_ADDRESS).expect("to connect");
    let write = read
        .try_clone()
        .expect("must be able to clone the tcp stream");

    let config = UserConfig::builder()
        .token("OAUTH")
        .nick("NICKNAME")
        .membership()
        .commands()
        .tags()
        .build()
        .expect("partial config");

    let mut client = Client::new(read, write);

    client.register(config).unwrap();

    let user = client.wait_for_ready().unwrap();
    println!(
        "connectedd with {} (id: {}). our color is: {}",
        user.display_name.unwrap(),
        user.user_id,
        user.color.unwrap_or_default()
    );

    client.on(|msg: PrivMsg, _: Writer<_>| {
        let name = msg.display_name().unwrap_or_else(|| msg.user());

        println!("{}: {}", name, msg.message())
    });

    let w = client.writer();
    w.join("CHANNEL").unwrap();
    w.send("CHANNEL", "joined").unwrap();

    client.run();

}
