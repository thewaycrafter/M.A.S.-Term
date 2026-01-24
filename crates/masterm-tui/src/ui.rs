use crate::app::{App, Tab};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph, Tabs},
    Frame,
};

pub fn draw(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Length(3), // Header
                Constraint::Min(0),    // Content
                Constraint::Length(3), // Footer
            ]
            .as_ref(),
        )
        .split(f.size());

    draw_header(f, app, chunks[0]);
    draw_content(f, app, chunks[1]);
    draw_footer(f, app, chunks[2]);
}

fn draw_header(f: &mut Frame, app: &App, area: Rect) {
    let titles: Vec<Line> = vec!["Dashboard (1)", "Config (2)", "Plugins (3)"]
        .into_iter()
        .map(Line::from)
        .collect();

    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title(app.title.as_str()))
        .highlight_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .select(match app.tab {
            Tab::Dashboard => 0,
            Tab::Config => 1,
            Tab::Plugins => 2,
        });

    f.render_widget(tabs, area);
}

fn draw_content(f: &mut Frame, app: &App, area: Rect) {
    match app.tab {
        Tab::Dashboard => draw_dashboard(f, app, area),
        Tab::Config => draw_config(f, app, area),
        Tab::Plugins => draw_plugins(f, app, area),
    }
}

fn draw_config(f: &mut Frame, app: &App, area: Rect) {
    let rows: Vec<ratatui::widgets::Row> = app
        .config_items
        .iter()
        .map(|(k, v)| {
            ratatui::widgets::Row::new(vec![k.clone(), v.clone()])
        })
        .collect();

    let widths = [Constraint::Percentage(30), Constraint::Percentage(70)];
    
    let table = ratatui::widgets::Table::new(rows, widths)
        .header(
            ratatui::widgets::Row::new(vec!["Key", "Value"])
                .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
                .bottom_margin(1),
        )
        .block(Block::default().borders(Borders::ALL).title("Configuration"))
        .column_spacing(1);

    f.render_widget(table, area);
}

fn draw_plugins(f: &mut Frame, app: &App, area: Rect) {
    let rows: Vec<ratatui::widgets::Row> = app
        .plugin_items
        .iter()
        .map(|(name, ver, desc)| {
            ratatui::widgets::Row::new(vec![name.clone(), ver.clone(), desc.clone()])
        })
        .collect();

    let widths = [Constraint::Percentage(20), Constraint::Percentage(15), Constraint::Percentage(65)];
    
    let table = ratatui::widgets::Table::new(rows, widths)
        .header(
            ratatui::widgets::Row::new(vec!["Name", "Version", "Description"])
                .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
                .bottom_margin(1),
        )
        .block(Block::default().borders(Borders::ALL).title("Installed Plugins"))
        .column_spacing(1);

    f.render_widget(table, area);
}

fn draw_dashboard(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage(50), // CPU
                Constraint::Percentage(50), // Memory
            ]
            .as_ref(),
        )
        .split(area);

    // CPU Usage
    let global_cpu = app.system.global_cpu_info().cpu_usage();
    let cpu_gauge = ratatui::widgets::Gauge::default()
        .block(Block::default().title("CPU Usage").borders(Borders::ALL))
        .gauge_style(Style::default().fg(Color::Green))
        .percent(global_cpu as u16);
    f.render_widget(cpu_gauge, chunks[0]);

    // Memory Usage
    let used_mem = app.system.used_memory();
    let total_mem = app.system.total_memory();
    let mem_percent = (used_mem as f64 / total_mem as f64 * 100.0) as u16;

    let mem_label = format!("{:.1} GB / {:.1} GB", used_mem as f64 / 1024.0 / 1024.0 / 1024.0, total_mem as f64 / 1024.0 / 1024.0 / 1024.0);
    let mem_gauge = ratatui::widgets::Gauge::default()
        .block(Block::default().title("Memory Usage").borders(Borders::ALL))
        .gauge_style(Style::default().fg(Color::Cyan))
        .percent(mem_percent)
        .label(mem_label);
    f.render_widget(mem_gauge, chunks[1]);
}

fn draw_footer(f: &mut Frame, _app: &App, area: Rect) {
    let p = Paragraph::new("Press 'q' to quit | '1'-'3' to switch tabs")
        .style(Style::default().fg(Color::Gray))
        .block(Block::default().borders(Borders::ALL));
    
    f.render_widget(p, area);
}
