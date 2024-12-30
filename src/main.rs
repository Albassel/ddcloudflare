
use tokio::time;
use reqwest::{Client, Error, header};
use std::sync::LazyLock;

static TOKEN: LazyLock<String> = LazyLock::new(|| {
    std::env::var("TOKEN").expect("No TOKEN provided in the .env file")
});

static INTERVAL: LazyLock<u64> = LazyLock::new(|| {
    std::env::var("INTERVAL").unwrap_or(String::from("50")).parse::<u64>().unwrap_or(50)
});

static DOMAINS: LazyLock<Vec<String>> = LazyLock::new(|| {
    let d = std::env::var("DOMAINS").expect("No TOKEN provided in the .env file");
    d.split(',').map(|record| record.trim().to_string()).collect::<Vec<String>>()
});

static ZONE: LazyLock<String> = LazyLock::new(|| {
    std::env::var("ZONE").expect("No TOKEN provided in the .env file")
});

static RECORDS: LazyLock<Vec<String>> = LazyLock::new(|| {
    let r = std::env::var("RECORDS").expect("No TOKEN provided in the .env file");
    r.split(',').map(|record| record.trim().to_string()).collect::<Vec<String>>()
});

const ADDR: &str = "https://cloudflare.com/cdn-cgi/trace";
const API: &str = "https://api.cloudflare.com/client/v4/zones/";







#[tokio::main]
async fn main() {
    if RECORDS.len() > DOMAINS.len() {panic!("Configuration error: you provided more records than domains")}
    if RECORDS.len() < DOMAINS.len() {panic!("Configuration error: you provided more domains than records")}

    dotenvy::dotenv().expect(".env file is inaccessible");
    let client = Client::new();

    let mut interval = time::interval(time::Duration::from_secs(*INTERVAL));

    loop {
        interval.tick().await;
        let clone = client.clone();
        let clone2 = client.clone();
        tokio::spawn(async move {
            let my_ip = get_my_ip(clone).await.unwrap();
            let my_ip = my_ip.lines().filter(|line| line.starts_with("ip=")).collect::<String>();
            let my_ip = my_ip.strip_prefix("ip=").unwrap();
            println!("{}", my_ip);
            update_ip(clone2, my_ip).await.unwrap();
        });
    }
}



async fn get_my_ip(client: Client) -> Result<String, Error> {
    let res = client.get(ADDR)
        .header(header::CONNECTION, "Close")
        .header(header::HOST, "cloudflare.com")
        .send()
        .await?
        .text()
        .await?;

    Ok(res)
}

async fn update_ip(client: Client, ip: &str) -> Result<(), Error> {

    for (idx, record) in RECORDS.iter().enumerate() {
        let body = format!("{{\"type\": \"A\", \"name\": \"{}\", \"content\": \"{}\"}}", DOMAINS[idx], ip);

        client.put(format!("{}/{}/dns_records/{}", API, *ZONE, record))
            .header(header::CONNECTION, "Close")
            .header(header::CONTENT_TYPE, "application/json")
            .header(header::HOST, "api.cloudflare.com")
            .header(header::AUTHORIZATION, format!("Bearer {}", *TOKEN))
            .body(body)
            .send()
            .await?;
    }

    Ok(())
}


