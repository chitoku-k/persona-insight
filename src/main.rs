use std::sync::Arc;

use anyhow::{anyhow, Error};
use clap::Parser;
use log::LevelFilter;
use serde::Deserialize;
use slack_hook::{Slack, PayloadBuilder, AttachmentBuilder};
use tokio::time::{sleep, Duration};
use twapi_reqwest::v1::Client;

#[derive(Debug, Parser)]
struct App {
    /// Twitter API Consumer key
    #[arg(long, env)]
    consumer_key: String,

    /// Twitter API Consumer secret
    #[arg(long, env)]
    consumer_secret: String,

    /// Twitter API Access key
    #[arg(long, env)]
    access_key: String,

    /// Twitter API Access secret
    #[arg(long, env)]
    access_secret: String,

    /// Slack webhook URL
    #[arg(long, env)]
    slack_webhook_url: String,

    /// Twitter User ID to watch update for
    #[arg(long, env)]
    user_id: String,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct User {
    id: u64,
    name: String,
    screen_name: String,
    location: Option<String>,
    url: Option<String>,
    description: Option<String>,
    protected: bool,
    verified: bool,
    followers_count: i64,
    friends_count: i64,
    listed_count: i64,
    favourites_count: i64,
    statuses_count: i64,
    created_at: String,
    profile_banner_url: String,
    profile_image_url_https: String,
    default_profile: bool,
    default_profile_image: bool,
}

async fn get_user(client: &Client, user_id: &str) -> Result<User, Error> {
    let user = client.get("https://api.twitter.com/1.1/users/show.json", &vec![("user_id", user_id)])
        .await?
        .json()
        .await?;

    Ok(user)
}

async fn notify_slack(slack: Arc<Slack>, image_url: &str, text: &str) -> Result<(), Error> {
    let attachment = AttachmentBuilder::new(image_url)
        .image_url(image_url)
        .build()
        .map_err(|e| anyhow!(e.to_string()))?;

    let payload = PayloadBuilder::new()
        .text(text)
        .attachments(vec![ attachment ])
        .build()
        .map_err(|e| anyhow!(e.to_string()))?;

    tokio::task::spawn_blocking(move || { slack.send(&payload) })
        .await?
        .map_err(|e| anyhow!(e.to_string()))
}

#[tokio::main]
async fn main() {
    env_logger::builder()
        .format_target(false)
        .format_timestamp_secs()
        .format_indent(None)
        .filter(None, LevelFilter::Info)
        .parse_env("LOG_LEVEL")
        .init();

    let app = App::parse();
    let client = Client::new(&app.consumer_key, &app.consumer_secret, &app.access_key, &app.access_secret);
    let slack = Arc::new(Slack::new(&*app.slack_webhook_url).expect("slack initialization error"));

    let mut profile_image_url_https = None;
    loop {
        match get_user(&client, &app.user_id).await {
            Ok(user) => {
                log::debug!("profile retrieved");
                log::debug!("- current: {}", user.profile_image_url_https);

                if let Some(url) = profile_image_url_https {
                    if url != user.profile_image_url_https {
                        log::info!("profile updated");
                        log::info!("- old: {}", url);
                        log::info!("- new: {}", user.profile_image_url_https);

                        if let Err(e) = notify_slack(
                            slack.clone(),
                            &user.profile_image_url_https.replace("_normal", ""),
                            "The icon URL has been updated.",
                        ).await {
                            log::error!("notification error: {:?}", e);
                        }
                    }
                }
                profile_image_url_https = Some(user.profile_image_url_https);
            },
            Err(e) => {
                log::error!("profile retrieval error: {:?}", e);
            },
        }
        sleep(Duration::from_secs(60)).await;
    }
}
