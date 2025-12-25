use std::io;
use std::time::{Instant, Duration};
use std::error::Error;

use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    layout::{Constraint, Layout, Direction},
    style::{
        Style, Stylize, Color, Modifier,
        palette::tailwind::{BLUE, GREEN, SLATE},
    },
    text::Line,
    widgets::{
        Block, List, Paragraph, ListItem, ListState,
        Bar, BarChart, BarGroup,
    },
    DefaultTerminal, Frame,
};
use serde::{Serialize, Deserialize};
use serde_json::Value;
use ureq;

use crate::core::cpu::CPUData;
use crate::core::mem::MemData;

pub struct App {
    cpu_data: CPUData,
    mem_data: MemData,
    should_exit: bool,
    ips: [String; 3],
    list_state: ListState,
}

impl App {
    pub fn new() -> Self {
        Self {
            cpu_data: CPUData::new(),
            mem_data: MemData::new(),
            should_exit: false,
            ips: [
                String::from("129.80.58.106:8080"),
                String::from("129.80.58.106:8081"),
                String::from("129.80.58.106:8082"),
            ],
            list_state: ListState::default(),
        }
    }

    pub fn run(&mut self, mut terminal: DefaultTerminal) -> io::Result<()> {
        terminal.draw(|frame| self.render(frame))?;
        self.list_state.select(Some(0));

        let mut last = Instant::now();
        while !self.should_exit {
            let time = last.elapsed();
            if time >= Duration::from_millis(500) {
                terminal.draw(|frame| self.render(frame))?;
                last = Instant::now();
            }

            self.handle_events()?;
        }
        Ok(())
    }

    fn handle_events(&mut self) -> io::Result<()> {
        if event::poll(Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') => self.should_exit = true,
                        KeyCode::Up => self.list_state_previous(),
                        KeyCode::Down => self.list_state_next(),
                        _ => ()
                    }
                }
            }
        }
        Ok(())
    }

    fn render(&mut self, frame: &mut Frame) {
        // Add one for RAM, 2 for top and bottom border
        let count = self.cpu_data.cpu_count + 1 + 2;

        let vertical = Layout::vertical([
            Constraint::Length(count as u16),
            Constraint::Fill(1)
        ]);
        let horizontal = Layout::horizontal([
            Constraint::Percentage(30),
            Constraint::Percentage(70),
        ]);

        let [system_view, process_view] = vertical.areas(frame.area());
        let [process_view, process_details] = horizontal.areas(process_view);
    
        let mut state = self.list_state.clone();

        frame.render_widget(self.cpu_chart(), system_view);
        //frame.render_widget(self.player_list(), process_view);
        frame.render_stateful_widget(self.player_list(), process_view, &mut state);
    }

    fn list_state_next(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= self.ips.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0
        };
        self.list_state.select(Some(i));
    }

    fn list_state_previous(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.ips.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0
        };
        self.list_state.select(Some(i));
    }

    fn player_list(&mut self) -> List {
        let player_counts = match self.get_player_counts() {
            Ok(vec) => vec,
            _ => vec![0,0,0]
        };

        let mut list_items: Vec<ListItem> = Vec::new();
        for i in 0..player_counts.len() {
            let item: ListItem = ListItem::from(
                format!("{} - {}", self.ips[i], player_counts[i])
            );
            list_items.push(item);
        }
        
        let style: Style = Style::new().bg(SLATE.c800).add_modifier(Modifier::BOLD);
        List::new(list_items)
            .block(Block::bordered().title("Player Counts"))
            .highlight_style(style)
            .highlight_symbol(">> ")
            .highlight_spacing(ratatui::widgets::HighlightSpacing::Always)
    }

    fn get_player_counts(&mut self) -> Result<Vec<u64>, Box<dyn Error>> {
        let mut player_counts: Vec<u64> = Vec::new();
        for ip in &self.ips {
            let response = ureq::get(
                format!("https://api.mcstatus.io/v2/status/java/{ip}")
            )
            .call()?
            .body_mut()
            .read_json::<Value>()?;
       
            player_counts.push(response["players"]["online"].as_u64().unwrap());
        }

        Ok(player_counts)
    }

    fn get_player_names(&mut self) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let mut player_names: Vec<Vec<String>> = Vec::new();
        for ip in &self.ips {
            let mut server_players: Vec<String> = Vec::new();

            let response = ureq::get(
                format!("https://api.mcstatus.io/v2/java/{ip}")
            )
            .call()?
            .body_mut()
            .read_json::<Value>()?;

            if let Some(players) = response["players"]["list"].as_array() {
                for player in players {
                    server_players.push(player["name_clean"].to_string());
                }
            }
            player_names.push(server_players);
        }

        Ok(player_names)
    }

    fn cpu_chart(&mut self) -> BarChart {
        let cpu_usage = self.cpu_data.get_cpu_usage();
    
        let mut bars: Vec<Bar> = Vec::new();

        // CPU usage bars
        for i in 0..cpu_usage.len() {
            let value = cpu_usage[i];
            let label = format!("CPU {i}");
            
            bars.push(
                self.horizontal_bar(label, value)
            );
        }

        // Memory
        let max: u64 = self.mem_data.get_total();
        let used: u64 = self.mem_data.get_used();
        let percent = used as f64 / max as f64;
        let used_str = self.mem_data.bytes_to_string(used);
        let max_str = self.mem_data.bytes_to_string(max);
        let green = (255.0 * (1.0 - percent)) as u8;
        let red = (255.0 * percent) as u8;
        let style = Style::new().fg(Color::Rgb(red, green, 0));
        bars.push(
            Bar::default()
                .value((percent * 100.0) as u64)
                .label(Line::from("RAM"))
                .text_value(format!("{used_str}/{max_str}"))
                .style(style)
                .value_style(style.reversed())
        );

        BarChart::default()
            .block(Block::bordered().title("System"))
            .data(BarGroup::default().bars(&bars))
            .max(100)
            .bar_width(1)
            .bar_gap(0)
            .direction(Direction::Horizontal)
    }

    fn horizontal_bar(&self, label: String, value: f32) -> Bar {
        let style = self.bar_color(value);
        Bar::default()
            .value(value as u64)
            .label(Line::from(label))
            .text_value(format!("{value:.0}"))
            .style(style)
            .value_style(style.reversed())
    }

    fn bar_color(&self, value: f32) -> Style {
        let green = (255.0 * (1.0 - (value / 100.0))) as u8;
        let red = (255.0 * (value / 100.0)) as u8;
        let color = Color::Rgb(red, green, 0);
        Style::new().fg(color)
    }
}
