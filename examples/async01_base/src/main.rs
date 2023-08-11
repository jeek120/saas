use std::{thread, time};
use futures::executor::block_on;

fn main() {
    // call_thread();
    // call_async();
    block_on(async_main())
}

fn download(url: &str) {
    println!("starting {}", url);
    thread::sleep(time::Duration::from_secs(1));
    println!("finished {}", url);
}

fn call_thread() {
    let thread_one = thread::spawn(|| download("https://www.baidu.com"));
    let thread_two = thread::spawn(|| download("https://www.google.com"));

    thread_one.join().expect("thread one panicked");
    thread_two.join().expect("thread two panicked");
}

async fn async_download(url: &str) {
    download(url)
}

fn call_async() {
    let future_one = async_download("https://www.baidu.com?async=true");
    //let future_two =  download("https://google.com");
    block_on(future_one);
}

#[derive(Debug)]
struct Song {
    name: String
}
async fn learn_song() -> Song {
    Song {
        name: String::from("爱的供养"),
    }
}

async fn sing(song: Song) {
    for i in 0..10 {
        thread::sleep(time::Duration::from_millis(i * 1000));
        println!("Song {:?}", song)
    }
} 

async fn dance() {
    for i in 0..10 {
        thread::sleep(time::Duration::from_millis(i * 1000));
        println!("dancing")
    }
}

async fn learn_and_sing() {
    let song = learn_song().await;
    sing(song).await;
}

async fn async_main () {
    let f1 = learn_and_sing();
    let f2 = dance();

    futures::join!(f1, f2);
}

