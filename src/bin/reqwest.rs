use serde::Deserialize;
use reqwest::Error;

#[derive(Deserialize, Debug)]
struct User {
    login: String,
    id: u32,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let request_url = format!("http://localhost:3030/bye");
    println!("{}", request_url);
    let response = reqwest::get(&request_url).await?;


    let users: Vec<User> = response.json().await?;
    println!("{:?}", users);
    Ok(())
}