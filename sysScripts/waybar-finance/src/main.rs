mod app;
mod ui;
mod config;
mod network;

use anyhow::Result;
use clap::Parser;
use ratatui::style::Color;
use app::App;
use config::{get_config_path, load_config};
use network::run_waybar_mode;
use ui::run_tui;
use serde::{Deserialize, Serialize};

use crate::network::FinnhubQuote;

//Bool to determine if we send a tooltip or launch the full TUI
//controlled with -t or -tui flag
#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    tui: bool,
}
//I need candle data for a real chart
#[derive(Debug, Deserialize)]
struct CandleResponse {
    c: Vec<f64>,  //Closing prices
    t: Vec<i64>, //timestamps
    s: String,  //status
}
enum AppEvent {
    //Network results
    QuoteFetched(String, Result<FinnhubQuote>),
    HistoryFetched(String, Result<Vec<(f64, f64)>>),
    Input(crossterm::event::Event),
    Tick,
}

#[derive(Debug, Serialize)]
struct WaybarOutput {
    text: String,
    tooltip: String,
    class: String,
}
#[tokio::main]
async fn main() -> Result<()> {
    let client = reqwest::Client::new();
    let args = Args::parse();
    let config_path = get_config_path()?;
    let config = load_config(&config_path)?;
    let mut app = App::new(config, String::from("Ready"), Color::Gray, None);
    if args.tui {
        println!("Initializing TUI mode...");
        run_tui(&client, &mut app).await?
    } else {
        run_waybar_mode(&client).await?; 
    }
    Ok(())
}
