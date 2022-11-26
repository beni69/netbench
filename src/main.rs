use clap::{Parser, Subcommand};
use indicatif::{ProgressBar, ProgressStyle};
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    sync::mpsc::channel,
    thread,
    time::Instant,
};

const DEFAULT_PORT: u16 = 42069;
const PINGS: usize = 100;
const RUNS: usize = 1000;
const BUFSIZE: usize = 2usize.pow(18);

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,

    #[clap(short, long)]
    verbose: bool,
}
#[derive(Debug, Subcommand)]
enum Commands {
    Server {
        #[clap(short = 'H', long)]
        #[arg(default_value_t = String::from("0.0.0.0"))]
        host: String,

        #[clap(short, long)]
        #[arg(default_value_t = DEFAULT_PORT)]
        port: u16,
    },
    Client {
        addr: String,
    },
}

fn main() -> Result<(), std::io::Error> {
    let args = Args::parse();
    match args.command {
        Commands::Server { host, port } => server(host, port),
        Commands::Client { addr } => client(addr),
    }
}

#[derive(Debug)]
enum State {
    Connected,
    Ping,
    Read,
    Write,
    Disconnected,
}
impl State {
    fn new() -> Self {
        eprintln!("State: Connected");
        Self::Connected
    }

    fn next(&mut self) {
        let new = match self {
            State::Connected => State::Ping,
            State::Ping => State::Read,
            State::Read => State::Write,
            State::Write => State::Disconnected,
            State::Disconnected => State::Connected,
        };
        eprintln!("State: {self:?} => {new:?}");
        *self = new;
    }
}

fn server(host: String, port: u16) -> std::io::Result<()> {
    println!("Starting server on {}:{}", host, port);
    let listener = TcpListener::bind((host, port))?;
    for stream in listener.incoming() {
        let stream = stream?;
        eprintln!("Connection from {}", stream.peer_addr()?);
        // server_handle(stream)?;
        thread::spawn(move || server_handle(stream).unwrap());
    }
    Ok(())
}
fn server_handle(mut stream: TcpStream) -> std::io::Result<()> {
    let mut state = State::new();
    stream.write_all(&get_version())?;

    state.next();
    for _ in 0..PINGS {
        let mut buffer = [0; 4];
        stream.read_exact(&mut buffer)?;
        stream.write_all(b"pong")?;
    }

    state.next();
    write_test(&mut stream, false)?;

    state.next();
    read_test(&mut stream, false)?;

    state.next();
    Ok(())
}

fn client(addr: String) -> std::io::Result<()> {
    let mut stream = if addr.contains(':') {
        TcpStream::connect(addr)
    } else {
        TcpStream::connect((addr, DEFAULT_PORT))
    }?;

    let mut state = State::new();
    let mut hello = [0; 3];
    stream.read_exact(&mut hello)?;
    let ver = hello
        .map(|x| {
            char::from_digit(x as u32, 10)
                .expect("server sent invalid version number")
                .to_string()
        })
        .join(".");
    eprintln!("Server version: {ver}");

    state.next();
    let mut pings = [0.0; PINGS];
    for i in 0..PINGS {
        let now = Instant::now();
        stream.write_all(b"ping")?;
        let mut buffer = [0; 4];
        stream.read_exact(&mut buffer)?;
        pings[i] = now.elapsed().as_nanos() as f32 / 1_000_000.0;
    }
    println!(
        "Average ping: {}ms",
        pings.iter().sum::<f32>() / PINGS as f32
    );

    state.next();
    read_test(&mut stream, true)?;

    state.next();
    write_test(&mut stream, true)?;

    state.next();
    Ok(())
}

fn get_version() -> [u8; 3] {
    let s = env!("CARGO_PKG_VERSION")
        .split('.')
        .map(|x| x.parse::<u8>().unwrap())
        .collect::<Vec<_>>();
    [s[0], s[1], s[2]]
}

// return arbitrary data to use for the benchmark
fn get_chunk() -> [u8; BUFSIZE] {
    let mut chunk = [0; BUFSIZE];
    for i in 0..BUFSIZE {
        chunk[i] = i as u8;
    }
    chunk
}

fn read_test(stream: &mut TcpStream, term: bool) -> std::io::Result<f32> {
    let mut res = [0.0; RUNS];
    let bar = if term { Some(bar()) } else { None };

    let (tx, rx) = channel::<f32>();
    let t = thread::spawn({
        let mut stream = stream.try_clone()?;
        move || {
            for _ in 0..RUNS {
                let now = Instant::now();
                stream.read_exact(&mut [0; BUFSIZE]).unwrap();
                tx.send(now.elapsed().as_nanos() as f32 / 1_000_000.0)
                    .unwrap();
            }
        }
    });

    for i in 0..RUNS {
        res[i] = rx.recv().unwrap();
        stream.write_all(&(i as u64).to_be_bytes())?;

        if let Some(bar) = &bar {
            bar.inc(BUFSIZE as u64);
            bar.set_message(speedfmt(res[i]));
        }
    }
    t.join().unwrap();

    let avg = res.iter().sum::<f32>() / RUNS as f32;
    if let Some(bar) = bar {
        bar.finish_with_message(speedfmt(avg));
    }

    Ok(avg)
}

fn write_test(stream: &mut TcpStream, term: bool) -> std::io::Result<f32> {
    let mut res = [0.0; RUNS];
    let bar = if term { Some(bar()) } else { None };

    let (tx, rx) = channel::<f32>();
    let t = thread::spawn({
        let mut stream = stream.try_clone()?;
        move || {
            let chunk = get_chunk();
            for _ in 0..RUNS {
                let now = Instant::now();
                stream.write_all(&chunk).unwrap();
                tx.send(now.elapsed().as_nanos() as f32 / 1_000_000.0)
                    .unwrap();
            }
        }
    });

    for i in 0..RUNS {
        res[i] = rx.recv().unwrap();
        let mut b = [0; 8];
        stream.read_exact(&mut b)?;
        assert_eq!(u64::from_be_bytes(b), i as u64, "out of sync");

        if let Some(bar) = &bar {
            bar.inc(BUFSIZE as u64);
            bar.set_message(speedfmt(res[i]));
        }
    }
    t.join().unwrap();

    let avg = res.iter().sum::<f32>() / RUNS as f32;
    if let Some(bar) = bar {
        bar.finish_with_message(speedfmt(avg));
    }

    Ok(avg)
}

#[allow(non_snake_case)]
fn speedfmt(ms: f32) -> String {
    let Bps = (BUFSIZE as f32 / ms) * 1000.0;
    let bps = Bps * 8.0;

    format!("{}/s - {}/s", bfmt(bps, 'b', 0), bfmt(Bps, 'B', 0))
}
fn bfmt(b: f32, q: char, lvl: usize) -> String {
    let lvls = ["", "K", "M", "G", "T", "P", "E", "Z", "Y"];
    if b >= 1000.0 {
        bfmt(b / 1000.0, q, lvl + 1)
    } else {
        format!("{:.1}{}{}", b, lvls[lvl], q)
    }
}

fn bar() -> ProgressBar {
    ProgressBar::new((RUNS * BUFSIZE) as u64).with_style( ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta}) {msg}").unwrap()
        .progress_chars("=>-"))
}
