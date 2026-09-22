//! Helium TUI dashboard (`helium dash`): node, peers, market, tunnel,
//! microVM and host stats on one screen. `q`/Esc quits, `r` refreshes
//! (auto-refresh every 2s).

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph},
    Terminal,
};
use std::io;
use std::time::{Duration, Instant};
use sysinfo::{Disks, Networks, System};

use crate::discovery::DhtDiscovery;
use crate::identity::IdentityManager;
use crate::market::Market;
use crate::mesh::MeshManager;
use crate::networking::WireGuardInterface;

const LOGO: &[&str] = &[
    "██╗  ██╗███████╗██╗     ██╗██╗   ██╗███╗   ███╗",
    "██║  ██║██╔════╝██║     ██║██║   ██║████╗ ████║",
    "███████║█████╗  ██║     ██║██║   ██║██╔████╔██║",
    "██╔══██║██╔══╝  ██║     ██║██║   ██║██║╚██╔╝██║",
    "██║  ██║███████╗███████╗██║╚██████╔╝██║ ╚═╝ ██║",
    "╚═╝  ╚═╝╚══════╝╚══════╝╚═╝ ╚═════╝ ╚═╝     ╚═╝",
];

const ACCENT: Color = Color::Cyan;
const ACCENT2: Color = Color::Magenta;

struct Snapshot {
    node_line: String,
    mesh_line: String,
    peers: Vec<String>,
    offers: Vec<String>,
    requests_line: String,
    balance_line: String,
    tunnel: Vec<String>,
    vm_line: String,
    cpu_pct: f64,
    mem_used_gb: f64,
    mem_total_gb: f64,
    disk_line: String,
    disk_ratio: f64,
    net_line: String,
}

fn short(id: &str) -> String {
    if id.len() > 18 {
        format!("{}…{}", &id[..10], &id[id.len() - 6..])
    } else {
        id.to_string()
    }
}

/// sysinfo reports bytes on recent versions, KiB on older ones — normalize.
fn to_gb(v: u64) -> f64 {
    let gb = v as f64 / 1024.0 / 1024.0 / 1024.0;
    if gb < 0.5 {
        v as f64 / 1024.0 / 1024.0 // actually KiB
    } else {
        gb
    }
}

async fn gather() -> Snapshot {
    let node_id = IdentityManager::load()
        .await
        .ok()
        .and_then(|idm| idm.node_id().ok())
        .unwrap_or_else(|| "not initialized (run: helium init)".to_string());

    let (mesh_line, mesh_peers) = match MeshManager::load().await {
        Ok(mesh) => {
            let joined = if mesh.is_joined().await { "joined" } else { "not joined" };
            let n = mesh.list_peers().await.map(|p| p.len()).unwrap_or(0);
            (format!("mesh: {joined}"), n)
        }
        Err(_) => ("mesh: n/a".to_string(), 0),
    };

    let mut peers = vec![format!("mesh peers: {mesh_peers}")];
    for p in DhtDiscovery::known_peers() {
        peers.push(format!("LAN {}", short(&p.peer_id)));
    }
    for p in DhtDiscovery::kad_peers() {
        peers.push(format!("DHT {}", short(&p.peer_id)));
    }

    let (offers, requests_line, balance_line) = match Market::open() {
        Ok(m) => {
            let offers = m
                .list_offers()
                .unwrap_or_default()
                .into_iter()
                .take(6)
                .map(|o| format!("{}: {} {} @ {:.2}/h", o.id, o.rtype, o.amount, o.price_per_hour))
                .collect();
            let nreq = m.list_requests().map(|r| r.len()).unwrap_or(0);
            let bal = if node_id.starts_with("not initialized") {
                "—".to_string()
            } else {
                format!("{:.2}", m.balance(&node_id).unwrap_or(0.0))
            };
            (offers, format!("open requests: {nreq}"), format!("balance: {bal}"))
        }
        Err(_) => (vec![], "open requests: n/a".to_string(), "balance: n/a".to_string()),
    };

    let tunnel = match WireGuardInterface::new("helium-poc").show() {
        Ok(s) => s.lines().take(8).map(|l| l.to_string()).collect(),
        Err(_) => vec!["tunnel down or needs sudo".to_string()],
    };

    let mut sys = System::new_all();
    sys.refresh_all();
    let cpu_pct = sys
        .cpus()
        .iter()
        .map(|c| c.cpu_usage() as f64)
        .sum::<f64>()
        / sys.cpus().len().max(1) as f64;
    let mem_used_gb = to_gb(sys.used_memory());
    let mem_total_gb = to_gb(sys.total_memory()).max(0.1);

    let (disk_line, disk_ratio) = Disks::new_with_refreshed_list()
        .iter()
        .find(|d| d.mount_point().to_string_lossy() == "/")
        .map(|d| {
            let total = to_gb(d.total_space()).max(0.1);
            let avail = to_gb(d.available_space());
            let used = (total - avail).max(0.0);
            (
                format!("disk /: {used:.1}/{total:.1} GiB"),
                (used / total).clamp(0.0, 1.0),
            )
        })
        .unwrap_or_else(|| ("disk: n/a".to_string(), 0.0));

    let nets = Networks::new_with_refreshed_list();
    let (rx, tx): (u64, u64) = nets
        .iter()
        .map(|(_, n)| (n.received(), n.transmitted()))
        .fold((0, 0), |(a, b), (x, y)| (a + x, b + y));
    let net_line = format!("net: ↓ {:.1} MiB ↑ {:.1} MiB", rx as f64 / 1024.0 / 1024.0, tx as f64 / 1024.0 / 1024.0);

    let vm_running = sys
        .processes()
        .values()
        .any(|p| p.name().contains("firecracker"));
    let vm_line = if vm_running {
        "microVM: RUNNING (firecracker)".to_string()
    } else {
        "microVM: stopped (fc-vm.sh up)".to_string()
    };

    Snapshot {
        node_line: format!("node: {}", short(&node_id)),
        mesh_line,
        peers,
        offers,
        requests_line,
        balance_line,
        tunnel,
        vm_line,
        cpu_pct: cpu_pct.clamp(0.0, 100.0),
        mem_used_gb,
        mem_total_gb,
        disk_line,
        disk_ratio,
        net_line,
    }
}

fn title_block(title: &str) -> Block<'_> {
    Block::default()
        .title(Span::styled(
            format!(" {title} "),
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
}

fn ui(f: &mut ratatui::Frame, s: &Snapshot) {
    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(9), Constraint::Min(0), Constraint::Length(1)])
        .split(f.area());

    let logo_lines: Vec<Line> = LOGO
        .iter()
        .map(|l| {
            Line::from(vec![
                Span::styled(*l, Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
                Span::raw("  "),
                Span::styled(
                    format!("v{}", env!("CARGO_PKG_VERSION")),
                    Style::default().fg(ACCENT2),
                ),
            ])
        })
        .collect();
    let mut header_lines = logo_lines;
    header_lines.push(Line::from(Span::styled(
        "private GPU mesh — q quit · r refresh · auto 2s",
        Style::default().fg(Color::DarkGray),
    )));
    f.render_widget(Paragraph::new(header_lines), root[0]);

    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(root[1]);
    let left = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),
            Constraint::Length(9),
            Constraint::Min(0),
        ])
        .split(cols[0]);
    let right = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(11),
            Constraint::Length(8),
            Constraint::Min(0),
        ])
        .split(cols[1]);

    let node_items = vec![ListItem::new(s.node_line.clone()), ListItem::new(s.mesh_line.clone())];
    f.render_widget(List::new(node_items).block(title_block("node")), left[0]);

    let mut market_items: Vec<ListItem> = s.offers.iter().map(|o| ListItem::new(o.clone())).collect();
    if market_items.is_empty() {
        market_items.push(ListItem::new("no open offers"));
    }
    market_items.push(ListItem::new(format!("{} · {}", s.requests_line, s.balance_line)));
    f.render_widget(List::new(market_items).block(title_block("market")), left[1]);

    let peer_items: Vec<ListItem> = s.peers.iter().map(|p| ListItem::new(p.clone())).collect();
    f.render_widget(List::new(peer_items).block(title_block("peers")), left[2]);

    let tun_items: Vec<ListItem> = s.tunnel.iter().map(|l| ListItem::new(l.clone())).collect();
    f.render_widget(List::new(tun_items).block(title_block("tunnel wg")), right[0]);

    let sys_block = title_block("host");
    let sys_inner = sys_block.inner(right[1]);
    f.render_widget(title_block("host"), right[1]);
    let gauges = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1), Constraint::Length(1), Constraint::Length(1)])
        .split(sys_inner);
    f.render_widget(
        Gauge::default()
            .ratio((s.cpu_pct / 100.0).clamp(0.0, 1.0) as f64)
            .label(format!("cpu {:.0}%", s.cpu_pct))
            .gauge_style(Style::default().fg(ACCENT)),
        gauges[0],
    );
    f.render_widget(
        Gauge::default()
            .ratio((s.mem_used_gb / s.mem_total_gb).clamp(0.0, 1.0) as f64)
            .label(format!("mem {:.1}/{:.1} GiB", s.mem_used_gb, s.mem_total_gb))
            .gauge_style(Style::default().fg(ACCENT2)),
        gauges[1],
    );
    f.render_widget(
        Gauge::default()
            .ratio(s.disk_ratio as f64)
            .label(s.disk_line.clone())
            .gauge_style(Style::default().fg(Color::Yellow)),
        gauges[2],
    );
    f.render_widget(Paragraph::new(s.net_line.clone()), gauges[3]);

    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(s.vm_line.clone(), Style::default().fg(Color::Green)),
        ]))
        .block(title_block("microvm")),
        right[2],
    );

    f.render_widget(
        Paragraph::new("helium mesh · q quit · r refresh"),
        root[2],
    );
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    Ok(Terminal::new(CrosstermBackend::new(stdout))?)
}

fn restore_terminal(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
) -> Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

pub async fn run_dashboard() -> Result<()> {
    let mut terminal = setup_terminal()?;

    let mut snap = gather().await;
    let mut last = Instant::now();

    loop {
        if event::poll(Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    KeyCode::Char('r') => {
                        snap = gather().await;
                        last = Instant::now();
                    }
                    _ => {}
                }
            }
        }
        if last.elapsed() >= Duration::from_secs(2) {
            snap = gather().await;
            last = Instant::now();
        }
        terminal.draw(|f| ui(f, &snap))?;
    }

    restore_terminal(&mut terminal)
}

/// Single frame then clean exit (demos, screenshots, CI goldens).
pub async fn run_dashboard_once() -> Result<()> {
    let mut terminal = setup_terminal()?;
    let snap = gather().await;
    terminal.draw(|f| ui(f, &snap))?;
    restore_terminal(&mut terminal)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;

    fn demo_snapshot() -> Snapshot {
        Snapshot {
            node_line: "node: HkKB0S6jY…BZCxt_rE".to_string(),
            mesh_line: "mesh: not joined".to_string(),
            peers: vec![
                "mesh peers: 0".to_string(),
                "LAN 12D3KooWPyb…9F1U".to_string(),
            ],
            offers: vec!["offer-abc: gpu 32 @ 0.50/h".to_string()],
            requests_line: "open requests: 1".to_string(),
            balance_line: "balance: 99.12".to_string(),
            tunnel: vec![
                "interface: helium-poc".to_string(),
                "peer: xThH…Ux1DU=".to_string(),
            ],
            vm_line: "microVM: RUNNING (firecracker)".to_string(),
            cpu_pct: 12.0,
            mem_used_gb: 2.1,
            mem_total_gb: 7.0,
            disk_line: "disk /: 15.0/109.0 GiB".to_string(),
            disk_ratio: 0.14,
            net_line: "net: ↓ 12.0 MiB ↑ 3.0 MiB".to_string(),
        }
    }

    #[test]
    fn dash_renders_all_panels() {
        let backend = TestBackend::new(120, 32);
        let mut terminal = Terminal::new(backend).unwrap();
        let snap = demo_snapshot();
        terminal.draw(|f| ui(f, &snap)).unwrap();
        let text: Vec<String> = terminal
            .backend()
            .buffer()
            .content()
            .chunks(120)
            .map(|row| row.iter().map(|c| c.symbol()).collect::<String>())
            .collect();
        let screen = text.join("\n");
        println!("{screen}");
        let lower = screen.to_lowercase();
        for needle in ["node", "market", "peers", "tunnel", "host", "microvm", "helium", "balance"] {
            assert!(lower.contains(needle), "missing panel: {needle}");
        }
        assert!(screen.contains('█'), "logo not rendered");
    }
}
