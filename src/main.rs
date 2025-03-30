use std::{
    error::Error,
    io,
    rc::Rc
};
use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind
    },
    execute,
    terminal::{
        disable_raw_mode, enable_raw_mode, EnterAlternateScreen,  LeaveAlternateScreen
    },
};
use ratatui::{
    backend::{
        Backend, CrosstermBackend
    }, layout::{
        self, Direction, Layout, Rect
    }, style::{
        Color, Style, Stylize
    }, widgets::{
        Block, Borders, Gauge, List, ListItem, Paragraph
    }, Terminal
};
use std::time::{
    Duration, Instant
};
use tui_framework_experiment::{
    Button, ButtonState, ButtonTheme,
};

struct App {
    running: bool,
    trackpaused: bool,
    version: String,
}

struct Track {
    title: String,
    artist: String,
    path: String,
    duration: String,
}

impl App {
    fn new() -> App {
        App { 
            running: true,
            trackpaused: false,
            version: String::from("v0.0.1")
        }
    }

    fn ontick(&mut self) {

    }

    fn onkey(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('q') => {
                self.running = false;
            }
            KeyCode::Char(' ') => {
                self.trackpaused = !self.trackpaused;
            }
            KeyCode::Char('j') => {
                // next track
            }
            _ => {}
        }
    }
}

fn createplaylist() -> Vec<ListItem<'static>> {
    let playlist = vec![
        Track {
            title: String::from("track 1"),
            artist: String::from("artist 1"),
            path: String::from("path 1"),
            duration: String::from("2:45")
        },
    ];

    playlist
        .iter()
        .map(|track| {
            let text = format!(" {} - {} ({})", track.title, track.artist, track.duration);
            ListItem::new(text)
        })
        .collect()
}

fn createlayout(area: Rect) -> (Rc<[Rect]>, Rc<[Rect]>, Rc<[Rect]>) {
    let outerlayout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![
            layout::Constraint::Percentage(70),
            layout::Constraint::Percentage(30)
        ])
        .split(area);

    let innerlayout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![
            layout::Constraint::Percentage(50),
            layout::Constraint::Percentage(50)
        ])
        .split(outerlayout[1]);
    
    let playerlayout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![
            layout::Constraint::Percentage(33), // track info
            layout::Constraint::Percentage(33), // player controls
            layout::Constraint::Percentage(34)  // progress
        ])
        .split(innerlayout[0]);

    (innerlayout, outerlayout, playerlayout)
}

fn playlistview(listitems: Vec<ListItem>) -> List {
    List::new(listitems)
                .block(Block::default().title(" playlist ").borders(Borders::ALL))
                .highlight_style(Style::default().fg(Color::Yellow))
                .highlight_symbol(">> ")
}

fn playerview(app: &App, track: &Track, progress: u16) -> (Paragraph<'static>, Paragraph<'static>, Gauge<'static>) {
    let playpausetext = if app.trackpaused { "play" } else { "pause" };

    let controls = format!("[<<] [{}] [>>]", playpausetext);

    let trackinfo = Paragraph::new(format!("{} - {}", track.title, track.artist))
        .alignment(ratatui::layout::Alignment::Center)
        .block(Block::default().borders(Borders::NONE));

    let controlspara = Paragraph::new(controls).alignment(ratatui::layout::Alignment::Center);

    let progressbar = Gauge::default()
        .block(Block::default().borders(Borders::ALL))
        .percent(progress)
        .gauge_style(Style::default().fg(Color::Green));
    
    (trackinfo, controlspara, progressbar)
}

fn run<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    tickrate: Duration,
) -> Result<(), Box<dyn Error>> {
    let mut lasttick = Instant::now();
    // hardcoded for now
    let track = Track{title: String::from("track 1"), artist: String::from("artist 1"), path: String::from("path 1"), duration: String::from("2:45")};

    loop {
        terminal.draw(|f| {
            // main window
            let area = f.area();
            let (innerlayout, outerlayout, playerlayout) = createlayout(area);
            let listitems: Vec<ListItem> = createplaylist();
            let list = playlistview(listitems);

            f.render_widget(list, outerlayout[0]);

            let (trackinfo, controls, progress) = playerview(&app, &track, 50);

            f.render_widget(trackinfo, playerlayout[0]);
            f.render_widget(controls, playerlayout[1]);
            f.render_widget(progress, playerlayout[2]);

            f.render_widget(
                Paragraph::new("inner 1")
                    .block(Block::new().borders(Borders::ALL)),
                innerlayout[1]);
        })?;

        let timeout = tickrate
            .checked_sub(lasttick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    app.onkey(key.code);
                }
            }
        }

        if lasttick.elapsed() >= tickrate {
            app.ontick();
            lasttick = Instant::now();
        }

        if !app.running {
            return Ok(());
        }
    }
}


fn main() -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    // create app and run it
    let tickrate = Duration::from_millis(250);
    let app = App::new();
    let res = run(&mut terminal, app, tickrate);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}
