use std::fs::remove_file;

use netlib::net::uds::{UnixListener, UnixStream};
use netlib::{Interest, Reactor, Result, System};

mod game;
mod server;

fn run() -> Result<()> {
    System::builder().finish();

    // cleanup possibly stale socket
    let socket_path = "/tmp/streamgame.sock";
    let _ = remove_file(&socket_path);

    let listner = UnixListener::bind(socket_path)?
        .map(Result::unwrap)
        .map(|(s, _)| {
            s.set_nonblocking(true).unwrap();
            UnixStream::new(s, Interest::Read).unwrap()
        });

    let server = listner
        .chain(server::Server::new())
        .chain(game::Game::new());

    System::start(server)?;

    Ok(())
}

fn main() -> Result<()> {
    let h = std::thread::spawn(run);
    let _ = h.join();
    Ok(())
}
