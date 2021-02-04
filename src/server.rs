use std::io::ErrorKind::WouldBlock;
use std::io::Read;

use netlib::net::uds::UnixStream;
use netlib::{Reaction, Reactor};

// const NUMBERS: [char; 10] = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];

pub struct Server {
    buf: [u8; 1024],
    connection: Option<UnixStream>,
}

impl Server {
    pub fn new() -> Self {
        Self {
            buf: [0; 1024],
            connection: None,
        }
    }
}

impl Reactor for Server {
    type Input = UnixStream;
    type Output = (u8, String);

    fn react(&mut self, reaction: Reaction<Self::Input>) -> Reaction<Self::Output> {
        match reaction {
            Reaction::Event(ev) if self.connection.is_none() => Reaction::Event(ev),
            Reaction::Event(ev) => {
                let con = &mut self.connection.as_mut().unwrap();

                let mut buf = Vec::new();

                if con.id == ev.owner {
                    loop {
                        match con.read(&mut self.buf) {
                            Ok(0) => {
                                self.connection = None;
                                return Reaction::Continue;
                            }
                            Ok(n) => buf.extend_from_slice(&self.buf[..n]),
                            Err(ref e) if e.kind() == WouldBlock => break,
                            Err(e) => {
                                eprintln!("{:?}", e);
                                return Reaction::Continue;
                            }
                        }
                    }

                    let s = match std::str::from_utf8(&buf) {
                        Ok(s) => s,
                        Err(_) => return Reaction::Continue,
                    };

                    let num_index = match s.find('|') {
                        Some(index) => index,
                        None => return Reaction::Continue,
                    };

                    let (num, name) = s.split_at(num_index);

                    let num = match num.parse::<u8>() {
                        Ok(n) if n < 8 => n,
                        _ => return Reaction::Continue,
                    };

                    // match s.chars().filter(|c| NUMBERS.contains(c)).next() {
                    //     Some(num) => Reaction::Value(num as u8 - 48),
                    //     None => Reaction::Continue
                    // }

                    Reaction::Value((num, name.to_string()))
                } else {
                    Reaction::Event(ev)
                }
            }
            Reaction::Value(val) => {
                self.connection = Some(val);
                Reaction::Continue
            }
            Reaction::Continue => Reaction::Continue,
        }
    }
}
