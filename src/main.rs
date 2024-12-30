
#![allow(clippy::never_loop)]

use tokio::time;
use reqwest::{Client, Error, header};
use std::sync::{LazyLock, Arc};
use serde_json::Value;

static TOKEN: LazyLock<String> = LazyLock::new(|| {
    std::env::var("TOKEN").expect("No TOKEN provided in the .env file")
});

static INTERVAL: LazyLock<u64> = LazyLock::new(|| {
    std::env::var("INTERVAL").unwrap_or(String::from("70")).parse::<u64>().unwrap_or(70)
});

static DOMAINS: LazyLock<Vec<String>> = LazyLock::new(|| {
    let d = std::env::var("DOMAINS").expect("No DOMAINS provided in the .env file");
    d.split(',').map(|record| record.trim().to_string()).collect::<Vec<String>>()
});

static ZONE: LazyLock<String> = LazyLock::new(|| {
    std::env::var("ZONE").expect("No ZONE provided in the .env file")
});

const ADDR: &str = "https://cloudflare.com/cdn-cgi/trace";
const API: &str = "https://api.cloudflare.com/client/v4/zones/";

const VERSION: &str = env!("CARGO_PKG_VERSION");





#[tokio::main(flavor = "current_thread")]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() == 2 && args[1] == "-v" || args.len() >= 2 && args[1] == "--version" {
        println!("{}", VERSION);
        std::process::exit(0);
    } 
    
    if args.len() == 3 && args[1] == "-f" {
        dotenvy::from_path(&args[2]).expect("File you provided is not accessible");
    } else {
        dotenvy::dotenv().expect(".env file is inaccessible");
    }


    let client = Client::new();
    let mut interval = time::interval(time::Duration::from_secs(*INTERVAL));
    let clone = client.clone();
    let records = Arc::new(get_record_ids(clone).await.unwrap());

    loop {
        interval.tick().await;
        let clone = client.clone();
        let clone2 = client.clone();
        let r = records.clone();
        tokio::spawn(async move {
            let my_ip = get_my_ip(clone).await.unwrap();
            let my_ip = my_ip.lines().filter(|line| line.starts_with("ip=")).collect::<String>();
            let my_ip = my_ip.strip_prefix("ip=").unwrap();
            update_ip(clone2, my_ip, r).await.unwrap();
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





async fn get_record_ids(client: Client) -> Result<Vec<String>, Error> {
    let mut records = Vec::new();

    let res = client.get(format!("{}/{}/dns_records/", API, *ZONE))
            .header(header::CONNECTION, "Close")
            .header(header::HOST, "api.cloudflare.com")
            .header(header::AUTHORIZATION, format!("Bearer {}", *TOKEN))
            .send()
            .await?
            .text()
            .await?;

    let res: Value = serde_json::from_str(&res).unwrap();
    assert!(res["success"] == true, "Can't read cloudflare DNS entry ids, cloudflare error response");

    let result = res["result"].as_array().unwrap();
    'outer: for domain in DOMAINS.iter() {
        for record in result {
            let name = record["name"].as_str().unwrap();
            if name == domain {
                records.push(record["id"].as_str().unwrap());
                continue 'outer;
            }
        }
        panic!("The domain {} you provided doesn't match any of the entries returned from cloudflare", domain);
    }

    Ok(records.iter().map(|s| s.to_string()).collect())
}






async fn update_ip(client: Client, ip: &str, records: Arc<Vec<String>>) -> Result<(), Error> {
    for (idx, record) in records.iter().enumerate() {
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


