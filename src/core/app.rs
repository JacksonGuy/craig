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

use crate::core::cpu::CPUData;
use crate::core::mem::MemData;
use crate::core::server::ServerState;

struct SystemStats {
    pub cpu_usages: Vec<f32>,
    pub mem_usage: u64,
    pub max_mem: u64,

    cpu_data: CPUData,
    mem_data: MemData,
}

impl SystemStats {
    pub fn new() -> Self {
        Self {
            cpu_usages: Vec::new(),
            mem_usage: 0,
            max_mem: 0,
            cpu_data: CPUData::new(),
            mem_data: MemData::new(),
        }
    }

    pub fn update(&mut self) {
        self.cpu_usages = self.cpu_data.get_cpu_usage();
    
        self.mem_usage = self.mem_data.get_used();
        self.max_mem = self.mem_data.get_total();
    }
}

pub struct App {
    cpu_data: CPUData,
    mem_data: MemData,
    should_exit: bool,
    ips: [String; 3],
    server_states: Vec<ServerState>,
    list_state: ListState,
    system_stats: SystemStats,
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
            server_states: Vec::new(),
            list_state: ListState::default(),
            system_stats: SystemStats::new(),
        }
    }

    pub fn run(&mut self, mut terminal: DefaultTerminal) -> io::Result<()> {
        // Create server states
        for ip in &self.ips {
            let state: ServerState = ServerState::new(&ip);
            self.server_states.push(state);
        }

        // Get system info
        self.system_stats.update();

        // Initial Draw
        terminal.draw(|frame| self.render(frame))?;
        self.list_state.select(Some(0));

        let mut system_update = Instant::now();
        let mut state_timer = Instant::now();
        while !self.should_exit {
            // Update Server Stats
            if system_update.elapsed() >= Duration::from_millis(500) {
                self.system_stats.update();
                system_update = Instant::now();
            }
            
            // Get server information
            if state_timer.elapsed() >= Duration::from_millis(30000) {
                for state in &mut self.server_states {
                    match state.update() {
                        Ok(_) => (),
                        Err(_) => continue
                    }
                }
                state_timer = Instant::now();
            }

            terminal.draw(|frame| self.render(frame))?;
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

    fn render(&mut self, frame: &mut Frame) {
        // Add one for RAM, 2 for top and bottom border
        let count = self.cpu_data.cpu_count + 1 + 2;

        let vertical = Layout::vertical([
            Constraint::Length(count as u16),
            Constraint::Fill(1)
        ]);
        let horizontal = Layout::horizontal([
            Constraint::Percentage(20),
            Constraint::Percentage(80),
        ]);

        let [system_view, process_view] = vertical.areas(frame.area());
        let [process_view, process_details] = horizontal.areas(process_view);
    
        let mut state = self.list_state.clone();

        frame.render_widget(self.cpu_chart(), system_view);
        frame.render_stateful_widget(self.player_list(), process_view, &mut state);
        frame.render_widget(self.server_details(), process_details);
    }

    fn server_details(&mut self) -> Paragraph {
        let index = match self.list_state.selected() {
            Some(i) => i,
            None => 0,
        };
        let state = &self.server_states[index];

        let mut lines: Vec<Line> = Vec::new();

        let status_line: &str = match state.status {
            false => "Status: Offline",
            true => "Status: Online"
        };

        let player_count_line = format!(
            "Player Count: {} / {}",
            state.player_count, state.max_players,
        );

        lines.push(Line::from(status_line));
        lines.push(Line::from(player_count_line));
        lines.push(Line::from("Players:"));
        for player in &state.players {
            lines.push(Line::from(format!("\t{}", player)));
        }

        Paragraph::new(lines)
            .block(Block::bordered().title("Server Info"))
    }

    fn player_list(&mut self) -> List {
        for state in &mut self.server_states {
            match state.update() {
                Ok(_) => (),
                Err(_) => continue,
            }
        }

        let mut list_items: Vec<ListItem> = Vec::new();
        for i in 0..self.server_states.len() {
            let item: ListItem = ListItem::from(
                format!("{} - {}", self.ips[i], self.server_states[i].player_count)
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

    fn cpu_chart(&mut self) -> BarChart {
        let cpu_usage = &self.system_stats.cpu_usages;
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
        let max: u64 = self.system_stats.max_mem;
        let used: u64 = self.system_stats.mem_usage;
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
