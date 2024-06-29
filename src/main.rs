mod thread_pool;
use thread_pool::ThreadPool;

use std::{
    fs,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
};

fn main() {
    const PORT: &str = "1234";
    let listener = TcpListener::bind(format!("127.0.0.1:{PORT}")).unwrap();
    const POOL_COUNT: u8 = 4;
    let pool = ThreadPool::new(POOL_COUNT);
    let continue_running = Arc::new(Mutex::new(true));

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let keep_going = Arc::clone(&continue_running);

        // make it multithreaded:
        pool.execute(|| {
            handle_connection(stream, keep_going);
        });

        let dont_stop = continue_running.lock().unwrap();
        if !*dont_stop {
            break;
        }
    }

    println!("Server shutting down...");
}

fn handle_connection(mut stream: TcpStream, continue_running: Arc<Mutex<bool>>) {
    let html = fs::read_to_string("files/basic.html").unwrap();

    let buf_reader = BufReader::new(&mut stream);
    let request_line = buf_reader.lines().next().unwrap().unwrap();

    let (status_code, content) = match &request_line[..] {
        "GET / HTTP/1.1" => (200, "It works!"),
        "GET /sleep HTTP/1.1" => (200, "I'm getting sleepy"),
        "GET /html HTTP/1.1" => (200, html.as_str()),
        "GET /favicon.ico HTTP/1.1" => (200, "Supposed to be an icon here"),
        "GET /exit HTTP/1.1" => (200, "Shutting down..."),
        "POST /test HTTP/1.1" => (200, "Post test succeeded"),
        _ => (400, "woops... Route not supported"),
    };

    let status_line = get_status_line(status_code);
    let length = content.len();
    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{content}");

    stream.write_all(response.as_bytes()).unwrap();

    if content.contains("Shut") {
        let mut keep_going = continue_running.lock().unwrap();
        *keep_going = false;
    }
    //println!("Request: {http_request:#?}");
}

fn get_status_line(code: u16) -> String {
    let meaning = match code {
        200 => "OK",
        404 => "NOT FOUND",
        _ => "",
    };

    return format!("HTTP/1.1 {code} {meaning}");
}
