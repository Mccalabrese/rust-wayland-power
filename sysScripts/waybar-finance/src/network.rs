use anyhow::{Result, Context};
use yahoo_finance_api::YahooConnector;
use time::OffsetDateTime;
use serde::{Deserialize, Serialize};
use crate::config::{get_config_path, load_config};

#[derive(Debug, Deserialize)]
pub struct FinnhubQuote {
    #[serde(rename = "c")]
    pub price: f64,
    #[serde(rename = "dp")]
    pub percent: f64,
}

#[derive(Debug, Serialize)]
pub struct WaybarOutput {
    pub text: String,
    pub tooltip: String,
    pub class: String,
}


pub async fn fetch_quote(client: &reqwest::Client, symbol: &str, key: &str) -> Result<FinnhubQuote> {
    let url = format!(
        "https://finnhub.io/api/v1/quote?symbol={}&token={}",
        symbol, key
    );
    let resp = client.get(&url).send().await?;
    if !resp.status().is_success() {
        return Err(anyhow::anyhow!("Failed to fetch quote: HTTP {}", resp.status()));
    }
    let quote: FinnhubQuote = resp.json().await?;
    Ok(quote)
}
pub async fn fetch_history(_client: &reqwest::Client, symbol: &str, _key: &str) -> Result<Vec<(f64, f64)>> {
    let provider = YahooConnector::new()?;
    let end = OffsetDateTime::now_utc();
    let start = end - time::Duration::days(365);
    let response = provider.get_quote_history(symbol, start, end).await
        .context("Yaho API Error")?;
    let quotes = response.quotes().context("No quotes in response")?;
    let points: Vec<(f64, f64)> = quotes.iter()
        .map(|q| (q.timestamp as f64, q.close))
        .collect();
    if points.is_empty() {
        return Err(anyhow::anyhow!("History data is empty"));
    }
    Ok(points)
}
pub async fn run_waybar_mode(client: &reqwest::Client) -> Result<()> {
    let config_path = get_config_path()?;
    let config = load_config(&config_path)?;
    let api_key = match &config.api_key {
        Some(k) => k,
        None => {
            eprintln!("Error: API key not found in config.json");
            return Ok(());
        }
    };
    let mut text_parts = Vec::new();
    let mut tooltip_parts = Vec::new();
    for symbol in &config.stocks {
        match fetch_quote(client, symbol, api_key).await {
            Ok(quote) => {
                let (color, icon) = if quote.percent >= 0.0 {
                    ("#a6e3a1", "")
                } else {
                    ("#f38ba8", "")
                };
                let part = format!(
                    "<span color='{}'>{} {:.2} {}</span>",
                    color, symbol, quote.price, icon
                );
                text_parts.push(part);
                tooltip_parts.push(format!("{}: ${:.2} ({:.2}%)", symbol, quote.price, quote.percent));
            }
            Err(_) => {
                text_parts.push(format!("<span color='#6c7086'>{} ???</span>", symbol));
            }
        }
    }
    let output = WaybarOutput {
        text: text_parts.join(" "),
        tooltip: tooltip_parts.join("\n"),
        class: "finance".to_string(),
    };
    println!("{}", serde_json::to_string(&output)?);
    Ok(())
}
