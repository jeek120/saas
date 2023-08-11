use futures::{join, try_join, executor::block_on};

fn main() {
    let (book, music) = block_on(get_book_and_music()).expect("获取失败");
    println!("book: {}, music: {}", book, music);
}


async fn get_book() -> Result<String, String> {
    Ok(String::from("Rust"))
}

async fn get_music() -> Result<String, String> {
    Ok(String::from("星辰大海"))
}

async fn get_book_and_music() -> Result<(String, String), String> {
    let book_fut = get_book();
    let music_fut = get_music();

    try_join!(book_fut, music_fut)
}