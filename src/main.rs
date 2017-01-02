use std::process::*;
use std::env;
use std::thread;
use std::io::prelude::*;
use std::io;
use std::io::BufReader;

#[derive(Debug, Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct Letter(u8);

impl Letter {
    fn new(c: u8) -> Option<Letter> {
        if 0x40 < c && c < 0x5B {
            Some(Letter(c))
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
struct Grid {
    width: u64,
    height: u64,
    values: Vec<Option<Letter>>
}

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
enum Message {
    Ready,
    Peel(Grid),
    Drop(Letter)
}

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
enum GameState {
   Waiting(Vec<bool>)
}

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
enum MessageError {
    ProtocolError,
    UnexpectedEOF,
    LetterError
}

struct MessageIterator<'a>(&'a mut Iterator<Item=io::Result<u8>>);

fn readDigit(chr: u8) -> u8 {
    if chr < 0x30 || 0x39 < chr {
        panic!("not a digit: {}", chr);
    }

    chr - 0x30
}

fn readInt<T>(seq: T) -> u64 where T: IntoIterator<Item=u8> {
    seq.into_iter().fold(0, |acc, x| (acc*10) + readDigit(x) as u64)
}


impl<'a> Iterator for MessageIterator<'a> {
    type Item = Message;


    fn next(&mut self) -> Option<Message> {
        let bytes = &mut self.0;

        bytes.next().and_then(|x: io::Result<u8>| x.ok()).map(|chr| {
            match chr {
                b'r' => Message::Ready,
                b'p' => {
                    let width = readInt(bytes.take(3).map(Result::unwrap));
                    let height = readInt(bytes.take(3).map(Result::unwrap));
                    Message::Peel(Grid {
                        width: width,
                        height: height,
                        values: bytes
                            .take((width*height) as usize)
                            .map(Result::unwrap)
                            .map(Letter::new)
                            .collect()
                    })
                },
                b'd' => Message::Drop(
                    Letter::new(bytes.next().expect("EOF").expect("IO Error"))
                        .expect("non-letter received when expecting letter")
                ),
                _ => panic!("protocol error: expecting command name, received {}", chr)
            }
        })
    }
}

fn make_worker(cmd: String) -> thread::JoinHandle<()> {
    thread::spawn(|| {
        let child = Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .unwrap();

        let out: ChildStdout = child.stdout.unwrap();
        // let mut bytes = BufReader::new(out).bytes();
        let mut bytes = out.bytes();

        for msg in MessageIterator(&mut bytes) {
            println!("{:?}", msg)
        }

        panic!("eof from player");

    })
}

fn main() {
    let cmds = env::args().skip(1);

    let mut GameState = GameState::Waiting(vec![false; 5]);

    let join_handles = cmds.map(make_worker);

    for handle in join_handles {
        handle.join().unwrap();
    }
}
