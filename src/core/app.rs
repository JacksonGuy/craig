use std::io;
use std::time::{Instant, Duration};
use std::error::Error;

use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    layout::{Constraint, Layout, Direction},
    style::{Style, Stylize, Color},
    text::{Line},
    widgets::{
        Block, List, Paragraph, ListItem,
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
}

impl App {
    pub fn new() -> Self {
        Self {
            cpu_data: CPUData::new(),
            mem_data: MemData::new(),
            should_exit: false,
        }
    }

    pub fn run(&mut self, mut terminal: DefaultTerminal) -> io::Result<()> {
        terminal.draw(|frame| self.render(frame))?;

        let mut last = Instant::now();
        while !self.should_exit {
            let time = last.elapsed();
            if time >= Duration::from_millis(3000) {
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
                if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
                    self.should_exit = true;
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
            Constraint::Percentage(20),
            Constraint::Percentage(80),
        ]);

        let [system_view, process_view] = vertical.areas(frame.area());
        let [process_view, process_details] = horizontal.areas(process_view);
    
        frame.render_widget(self.cpu_chart(), system_view);

        match self.server_player_list() {
            _ => ()
        }
    }

    fn server_player_list(&mut self) -> Result<(), Box<dyn Error>> {
        let ips = [
            "129.80.58.106:8080",
            "129.80.58.106:8081",
            "129.80.58.106:8082",
        ];

        let mut player_counts: Vec<u64> = Vec::new();
        for ip in ips {
            let response = ureq::get(
                format!("https://api.mcstatus.io/v2/status/java/{ip}")
            )
            .call()?
            .body_mut()
            .read_json::<Value>()?;
       
            player_counts.push(response["players"]["online"].as_u64().unwrap());
        }

        Ok(())
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
