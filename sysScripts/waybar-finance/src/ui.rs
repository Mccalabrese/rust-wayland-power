use std::io::stdout;
use anyhow::Result;
use chrono::DateTime;
use ratatui::{
    prelude::{CrosstermBackend, Terminal},
    widgets::{Block, Borders, Paragraph, ListItem, List, Clear, Chart, Dataset, Axis, GraphType},
    layout::{Rect, Layout, Direction, Constraint},
    prelude::*,
    style::{Color},
};
use crossterm::{
    event::{self, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use crate::app::{App, InputMode};
use crate::config::save_config;
use crate::network::{fetch_quote, fetch_history, FinnhubQuote};

pub enum AppEvent {
    //Network results
    QuoteFetched(String, Result<FinnhubQuote>),
    HistoryFetched(String, Result<Vec<(f64, f64)>>),
    Input(crossterm::event::Event),
    Tick,
}

pub async fn run_tui(client: &reqwest::Client, app: &mut App) -> Result<()> {
    let mut stdout = stdout();
    stdout.execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    //create event channel
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<AppEvent>();
    terminal.clear()?;
    //start event tick task
    let tx_tick = tx.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_millis(250));
        loop {
            interval.tick().await;
            if tx_tick.send(AppEvent::Tick).is_err() {
                break;
            }
        }
    });
    //start input event task
    let tx_iput = tx.clone();
    tokio::task::spawn_blocking(move || {
        loop {
            if let Ok(event) = crossterm::event::read() {
                if tx_iput.send(AppEvent::Input(event)).is_err() {
                    break;
                }
            }
        }
    });
    //main loop
    loop {
        terminal.draw(|frame| {
            ui(frame, app);
        })?;
        if let Some(event) = rx.recv().await {
            match event {
                AppEvent::Tick => {
                    //just let the loop spin, no action
                }
                AppEvent::QuoteFetched(sym, res) => {
                    match res {
                        Ok(q) => {
                            app.current_quote = Some(q);
                            app.message = format!("Updated {}", sym);
                            app.message_color = Color::Red;
                        }
                        Err(e) => {
                            app.message = format!("Error: {}", e);
                            app.message_color = Color::Red;
                        }
                    }
                }
                AppEvent::HistoryFetched(sym, res) => {
                    match res {
                        Ok(h) => app.stock_history = Some(h),
                        Err(_) => app.stock_history = None,
                    }
                }
                AppEvent::Input(event) => {
                    if let crossterm::event::Event::Key(key_event) = event {
                        if key_event.kind == KeyEventKind::Press {
                            match app.input_mode {
                                InputMode::Normal => match key_event.code {
                                    KeyCode::Char('q') => app.should_quit = true,
                                    KeyCode::Char('a') => {
                                        app.input_mode = InputMode::Editing;
                                        app.message = "Enter Symbol...".to_string();
                                        app.message_color = Color::Yellow;
                                    }
                                    KeyCode::Down => app.next(),
                                    KeyCode::Up => app.previous(),
                                    KeyCode::Enter => {
                                        if let Some(selected) = app.state.selected() {
                                            let symbol = app.stocks[selected].clone();
                                            if let Some(api_key) = &app.api_key {
                                                let client_clone = client.clone();
                                                let api_key_clone = api_key.clone();
                                                let tx_clone = tx.clone();
                                                
                                                app.message = format!("Fetching {}...", symbol);
                                                app.message_color = Color::Cyan;

                                                tokio::spawn(async move {
                                                    let q_res = fetch_quote(&client_clone, &symbol, &api_key_clone).await;
                                                    let _ = tx_clone.send(AppEvent::QuoteFetched(symbol.clone(), q_res));
                                                    
                                                    let h_res = fetch_history(&client_clone, &symbol, &api_key_clone).await;
                                                    let _ = tx_clone.send(AppEvent::HistoryFetched(symbol, h_res));
                                                });
                                            }
                                        }
                                    }
                                    KeyCode::Char('d') | KeyCode::Delete => app.delete(),
                                    _ => {}
                                },
                                InputMode::Editing => match key_event.code {
                                    KeyCode::Enter => {
                                        let new_symbol = app.input.trim().to_uppercase();
                                        if !new_symbol.is_empty() {
                                            if app.stocks.contains(&new_symbol) {
                                                app.message = format!("{} exists!", new_symbol);
                                                app.message_color = Color::Yellow;
                                                app.input.clear();
                                                app.input_mode = InputMode::Normal;
                                            } else {
                                                if let Some(api_key) = &app.api_key {
                                                    let client_clone = client.clone();
                                                    let api_key_clone = api_key.clone();
                                                    let tx_clone = tx.clone();
                                                    let symbol = new_symbol.clone();

                                                    app.message = format!("Adding {}...", symbol);
                                                    app.stocks.push(symbol.clone());
                                                    app.state.select(Some(app.stocks.len() - 1));
                                                    app.input.clear();
                                                    app.input_mode = InputMode::Normal;

                                                    tokio::spawn(async move {
                                                        let q_res = fetch_quote(&client_clone, &symbol, &api_key_clone).await;
                                                        let _ = tx_clone.send(AppEvent::QuoteFetched(symbol.clone(), q_res));
                                                        
                                                        let h_res = fetch_history(&client_clone, &symbol, &api_key_clone).await;
                                                        let _ = tx_clone.send(AppEvent::HistoryFetched(symbol, h_res));
                                                    });
                                                }
                                            }
                                        }
                                    }
                                    KeyCode::Esc => {
                                        app.input.clear();
                                        app.input_mode = InputMode::Normal;
                                        app.message = "Ready".to_string();
                                        app.message_color = Color::Gray;
                                    }
                                    KeyCode::Char(c) => app.input.push(c),
                                    KeyCode::Backspace => { app.input.pop(); },
                                    _ => {}
                                }
                            }
                        }
                    }
                }
            }
        }
        if app.should_quit {
            break;
        }
    }
    terminal.backend_mut().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    //save new config
    save_config(app)?;
    Ok(())
}


pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

pub fn ui(frame: &mut ratatui::Frame, app: &mut App) {
    //verticle split for main vs footer
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(frame.area());
    //horizontal split (List vs Chart)
    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Min(0),
        ])
        .split(main_layout[0]);
    let watchlist: Vec<ListItem> = app
        .stocks
        .iter()
        .map(|s| ListItem::new(s.as_str()))
        .collect();
    let list = List::new(watchlist)
        .block(Block::default()
            .title("Watchlist")
            .borders(Borders::ALL))
        .highlight_style(Style::default().bg(Color::Blue))
        .highlight_symbol(">> ");
    frame.render_stateful_widget(list, content_chunks[0], &mut app.state);
    if let Some(history) = &app.stock_history {
        let first_price = history[0].1;
        let last_price = history.last().unwrap().1;
        let start_ts = history[0].0 as i64;
        let end_ts = history.last().unwrap().0 as i64;
        let start_date = DateTime::from_timestamp(start_ts, 0).unwrap_or_default();
        let end_date = DateTime::from_timestamp(end_ts, 0).unwrap_or_default();
        let start_label = start_date.format("%Y-%m-%d").to_string();
        let end_label = end_date.format("%Y-%m-%d").to_string();
        let chart_color = if last_price >= first_price {
            Color::Green
        } else {
            Color::Red
        };
        let datasets = vec![
            Dataset::default()
                .marker(ratatui::symbols::Marker::Braille)
                .graph_type(GraphType::Line)
                .style(Style::default().fg(chart_color))
                .data(history),
        ];
        //Find y axis bounds 
        let min_price = history.iter().map(|(_, y)| *y).fold(f64::INFINITY, |a, b| a.min(b));
        let max_price = history.iter().map(|(_, y)| *y).fold(f64::NEG_INFINITY, |a, b| a.max(b));
        //Create the chart
        let chart = Chart::new(datasets)
            .block(Block::default().title("1 Year History").borders(Borders::ALL))
            .x_axis(Axis::default()
                .title("Date")
                .style(Style::default().fg(Color::Gray))
                .bounds([history[0].0, history.last().unwrap().0]) //these are times, start to end time
                .labels(vec![
                    Span::raw(start_label),
                    Span::raw(end_label),
                ]))
            .y_axis(Axis::default()
                .title("Price")
                .style(Style::default().fg(Color::Gray))
                .bounds([min_price, max_price])
                .labels(vec![
                    Span::raw(format!("{:.0}", min_price)),
                    Span::raw(format!("{:.0}", max_price)),
                ]));
        frame.render_widget(chart, content_chunks[1]);
    } else {
        let placeholder = Paragraph::new("Press Enter to load Chart")
            .block(Block::default().title("Chart").borders(Borders::ALL));
        frame.render_widget(placeholder, content_chunks[1]);
    }
    if app.input_mode == InputMode::Editing {
        let area = centered_rect(60, 20, frame.area());
        // 1. Clear the space
        frame.render_widget(Clear, area);
        //draw input box
        let input_block = Paragraph::new(app.input.as_str())
            .block(Block::default()
                .borders(Borders::ALL)
                .title("Input Stock Ticker (Press Enter to Confirm, Esc to Cancel)"));
        frame.render_widget(input_block, area);
    }
    let footer = Paragraph::new(app.message.as_str())
        .style(Style::default().fg(app.message_color));
    frame.render_widget(footer, main_layout[1]);

}

