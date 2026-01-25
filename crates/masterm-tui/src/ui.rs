use crate::app::{App, Tab};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::{Line, Span},
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType, Paragraph, Row, Table, Tabs},
    Frame,
};

pub fn draw(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Length(8), // Big Header with Logo
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

fn draw_content(f: &mut Frame, app: &App, area: Rect) {
    match app.tab {
        Tab::Dashboard => draw_dashboard(f, app, area),
        Tab::Config => draw_config(f, app, area),
        Tab::Plugins => draw_plugins(f, app, area),
    }
}

// --- Header Components ---

fn draw_header(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(40), Constraint::Percentage(60)].as_ref())
        .split(area);

    // ASCII LOGO in left chunk
    let logo_text = [
        "███╗   ███╗ █████╗ ███████╗████████╗███████╗██████╗ ███╗   ███╗",
        "████╗ ████║██╔══██╗██╔════╝╚══██╔══╝██╔════╝██╔══██╗████╗ ████║",
        "██╔████╔██║███████║███████╗   ██║   █████╗  ██████╔╝██╔████╔██║",
        "██║╚██╔╝██║██╔══██║╚════██║   ██║   ██╔══╝  ██╔══██╗██║╚██╔╝██║",
        "██║ ╚═╝ ██║██║  ██║███████║   ██║   ███████╗██║  ██║██║ ╚═╝ ██║",
        "╚═╝     ╚═╝╚═╝  ╚═╝╚══════╝   ╚═╝   ╚══════╝╚═╝  ╚═╝╚═╝     ╚═╝",
    ];
    let logo_lines: Vec<Line> = logo_text
        .iter()
        .enumerate()
        .map(|(i, s)| {
            // Simple gradient effect
            let color = match i % 3 {
                0 => Color::Cyan,
                1 => Color::Magenta,
                _ => Color::Blue,
            };
            Line::from(Span::styled(
                *s,
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            ))
        })
        .collect();

    let logo = Paragraph::new(logo_lines).block(Block::default());
    f.render_widget(logo, chunks[0]);

    // Tabs in right chunk
    let titles: Vec<Line> = vec!["󰆼 Dashboard (1)", " Config (2)", " Plugins (3)"]
        .into_iter()
        .map(|t| {
            Line::from(Span::styled(
                t,
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ))
        })
        .collect();

    let tabs = Tabs::new(titles)
        .block(card_block("Menu"))
        .highlight_style(
            Style::default()
                .fg(Color::Cyan)
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .select(match app.tab {
            Tab::Dashboard => 0,
            Tab::Config => 1,
            Tab::Plugins => 2,
        });

    f.render_widget(tabs, chunks[1]);
}

// --- Dashboard Components ---

fn draw_dashboard(f: &mut Frame, app: &App, area: Rect) {
    // 2x2 Grid
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(area);

    let top_row = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(rows[0]);

    let bottom_row = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(rows[1]);

    // Top Left: CPU
    draw_line_chart(
        f,
        app,
        top_row[0],
        "CPU Usage %",
        &app.cpu_history,
        Color::Cyan,
    );

    // Top Right: Memory
    draw_line_chart(
        f,
        app,
        top_row[1],
        "Memory Usage %",
        &app.mem_history,
        Color::Magenta,
    );

    // Bottom Left: Network I/O
    draw_network_chart(f, app, bottom_row[0]);

    // Bottom Right: System Info
    draw_system_info(f, app, bottom_row[1]);
}

fn draw_network_chart(f: &mut Frame, app: &App, area: Rect) {
    let rx_data = &app.rx_history;
    let tx_data = &app.tx_history;

    let datasets = vec![
        Dataset::default()
            .name("RX (KB/s)")
            .marker(symbols::Marker::Braille)
            .style(Style::default().fg(Color::Green))
            .data(rx_data),
        Dataset::default()
            .name("TX (KB/s)")
            .marker(symbols::Marker::Braille)
            .style(Style::default().fg(Color::Yellow))
            .data(tx_data),
    ];

    let x_labels = vec![
        Span::styled(
            format!("{:.0}", (app.tick_count - 100.0).max(0.0)),
            Style::default().add_modifier(Modifier::BOLD),
        ),
        Span::raw(format!("{:.0}", app.tick_count)),
    ];

    let chart = Chart::new(datasets)
        .block(card_block("Network Traffic"))
        .x_axis(
            Axis::default()
                .title("Time")
                .style(Style::default().fg(Color::Gray))
                .labels(x_labels)
                .bounds([(app.tick_count - 100.0).max(0.0), app.tick_count]),
        )
        .y_axis(
            Axis::default()
                .title("KB/s")
                .style(Style::default().fg(Color::Gray))
                .labels(vec![Span::raw("0"), Span::raw("Auto")])
                .bounds([0.0, 1000.0]), // Ideally dynamic, but static for now for safety
        );

    f.render_widget(chart, area);
}

fn draw_system_info(f: &mut Frame, app: &App, area: Rect) {
    let cpu_count = format!("{}", app.system.cpus().len());
    let memory_total = format!(
        "{:.1} GB",
        app.system.total_memory() as f64 / 1024.0 / 1024.0 / 1024.0
    );

    let rows = vec![
        Row::new(vec!["OS System", &app.os_info]),
        Row::new(vec!["Kernel", &app.kernel_ver]),
        Row::new(vec!["Hostname", &app.host_name]),
        Row::new(vec!["CPUs", &cpu_count]),
        Row::new(vec!["Total Memory", &memory_total]),
    ];

    let table = Table::new(
        rows,
        [Constraint::Percentage(30), Constraint::Percentage(70)],
    )
    .block(card_block("System Information"))
    .header(
        Row::new(vec!["Property", "Value"])
            .style(
                Style::default()
                    .fg(Color::Blue)
                    .add_modifier(Modifier::BOLD),
            )
            .bottom_margin(1),
    )
    .column_spacing(2);

    f.render_widget(table, area);
}

// --- Helpers ---

fn card_block(title: &str) -> Block<'_> {
    Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::White))
        .border_type(ratatui::widgets::BorderType::Rounded)
        .title(Span::styled(
            format!(" {} ", title),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ))
        .title_alignment(ratatui::layout::Alignment::Center)
}

fn draw_line_chart(
    f: &mut Frame,
    app: &App,
    area: Rect,
    title: &str,
    data: &[(f64, f64)],
    color: Color,
) {
    let dataset = Dataset::default()
        .marker(symbols::Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(color))
        .data(data);

    let x_labels = vec![
        Span::styled(
            format!("{:.0}", (app.tick_count - 100.0).max(0.0)),
            Style::default().add_modifier(Modifier::BOLD),
        ),
        Span::raw(format!("{:.0}", app.tick_count)),
    ];

    let chart = Chart::new(vec![dataset])
        .block(card_block(title))
        .x_axis(
            Axis::default()
                .title("Time")
                .style(Style::default().fg(Color::Gray))
                .labels(x_labels)
                .bounds([(app.tick_count - 100.0).max(0.0), app.tick_count]),
        )
        .y_axis(
            Axis::default()
                .title("Usage")
                .style(Style::default().fg(Color::Gray))
                .labels(vec![
                    Span::styled("0", Style::default().add_modifier(Modifier::BOLD)),
                    Span::styled("100", Style::default().add_modifier(Modifier::BOLD)),
                ])
                .bounds([0.0, 100.0]),
        );

    f.render_widget(chart, area);
}

// --- Other Tabs ---

fn draw_config(f: &mut Frame, app: &App, area: Rect) {
    let rows: Vec<Row> = app
        .config_items
        .iter()
        .map(|(k, v)| Row::new(vec![k.clone(), v.clone()]))
        .collect();

    let widths = [Constraint::Percentage(30), Constraint::Percentage(70)];

    let table = Table::new(rows, widths)
        .header(
            Row::new(vec!["Key", "Value"])
                .style(
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )
                .bottom_margin(1),
        )
        .block(card_block("Configuration"))
        .column_spacing(1);

    f.render_widget(table, area);
}

fn draw_plugins(f: &mut Frame, app: &App, area: Rect) {
    let rows: Vec<Row> = app
        .plugin_items
        .iter()
        .map(|(name, ver, desc)| Row::new(vec![name.clone(), ver.clone(), desc.clone()]))
        .collect();

    let widths = [
        Constraint::Percentage(20),
        Constraint::Percentage(15),
        Constraint::Percentage(65),
    ];

    let table = Table::new(rows, widths)
        .header(
            Row::new(vec!["Name", "Version", "Description"])
                .style(
                    Style::default()
                        .fg(Color::Magenta)
                        .add_modifier(Modifier::BOLD),
                )
                .bottom_margin(1),
        )
        .block(card_block("Installed Plugins"))
        .column_spacing(1);

    f.render_widget(table, area);
}

fn draw_footer(f: &mut Frame, _app: &App, area: Rect) {
    let p = Paragraph::new(Line::from(vec![
        Span::raw("Press "),
        Span::styled(
            "q",
            Style::default().add_modifier(Modifier::BOLD).fg(Color::Red),
        ),
        Span::raw(" to quit | "),
        Span::styled(
            "1-3",
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::Yellow),
        ),
        Span::raw(" to switch tabs | MASTerm v1.0.0"),
    ]))
    .style(Style::default().fg(Color::Gray))
    .block(Block::default().borders(Borders::TOP));

    f.render_widget(p, area);
}
