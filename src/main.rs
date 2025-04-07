use anyhow::Result;
use dotenvy::dotenv;
use fantoccini::Locator;
use std::collections::VecDeque;
use std::time::Duration;
use tokio::time::sleep;
use urlencoding::encode;

#[tokio::main]
async fn main() -> Result<()> {
    _ = dotenv();
    let open_page_timeout_sec = 10;
    let sleep_duration = std::env::var("SLEEP_SEC")?.parse::<u64>()?;

    loop {
        poll_site(open_page_timeout_sec).await?;
        sleep(Duration::from_secs(sleep_duration)).await;
    }
}

async fn poll_site(timeout_sec: u64) -> Result<()> {
    let rust_jobs_url = "https://rustjobs.dev";
    let job_links = get_links(rust_jobs_url, "http://localhost:9515", timeout_sec).await?;
    let inserted_links = persist_links(&job_links)?;
    if inserted_links.is_empty() {
        return Ok(());
    }

    let message = create_message(&inserted_links);

    send_message(
        &std::env::var("BOT_TOKEN")?,
        &std::env::var("CHAT_ID")?,
        &message,
    )
    .await?;

    Ok(())
}

fn create_message(links: &Vec<String>) -> String {
    let mut dequeu = VecDeque::from(links.clone());

    dequeu.push_front("RustJobs updates:\n".to_string());
    dequeu.push_back("#rust_jobs".to_string());

    dequeu
        .iter()
        .fold(String::new(), |acc, link| format!("{acc}{link}\n\n"))
        .to_string()
}

async fn send_message(token: &str, chat_id: &str, message: &str) -> Result<()> {
    let encoded_message = encode(&message);
    let url = format!(
        "https://api.telegram.org/bot{}/sendMessage?chat_id={}&text={}&disable_notification=true&disable_web_page_preview=true",
        token, chat_id, encoded_message
    );

    reqwest::Client::new().post(&url).send().await?;

    Ok(())
}

async fn get_links(url: &str, browser_url: &str, timeout: u64) -> Result<Vec<String>> {
    let mut map = serde_json::Map::new();
    let json = serde_json::json!({
        "args": [
            "--headless",
            "--no-sandbox",
            "--disable-dev-shm-usage"
        ]
    });
    map.insert("goog:chromeOptions".to_string(), json);

    let client = fantoccini::ClientBuilder::native()
        .capabilities(map)
        .connect(browser_url)
        .await?;

    client.goto(url).await?;
    sleep(Duration::from_secs(timeout)).await;

    let links = client
        .find_all(Locator::Css(r#"a[eventcategory="Featured Job Title"]"#))
        .await?;

    let mut result = Vec::new();
    for link in links {
        if let Some(href) = link.attr("href").await? {
            result.push(format!("{url}{href}"));
        }
    }

    client.close().await?;

    Ok(result)
}

fn persist_links(links: &Vec<String>) -> Result<Vec<String>> {
    let mut inserted_links = Vec::new();
    let db = sled::open("rust_jobs")?;
    for link in links {
        match db.get(link) {
            Ok(vec_opt) => match vec_opt {
                Some(_) => {
                    println!("Already exists: {:?}", link);
                }
                None => {
                    _ = db.insert(link, link.as_bytes())?;
                    inserted_links.push(link.to_string());
                    println!("Inserted: {:?}", link);
                }
            },
            Err(err) => {
                println!("Reading database error: {:?}", err);
            }
        }
    }

    Ok(inserted_links)
}
