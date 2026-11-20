//! whiskerd: wireless mouse battery levels from 2.4 GHz dongles, published as a small JSON file.

mod devices;
mod discovery;
mod drivers;
mod hidraw;
mod netlink;
mod power_supply;
mod smooth;
mod state;

use std::fs::File;
use std::io::{self, Read, Write};
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd, RawFd};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use devices::Protocol;
use discovery::Found;
use drivers::{compx, keychron};
use state::DeviceState;

const POLL_INTERVAL: Duration = Duration::from_secs(300);
/// How often to retry a polled mouse that gave no reading (asleep). Only the receiver answers,
/// so this does not wake the mouse.
const RETRY_INTERVAL: Duration = Duration::from_secs(30);
const CLASS_HIDRAW: &str = "/sys/class/hidraw";
const CLASS_POWER: &str = "/sys/class/power_supply";
const DEV: &str = "/dev";

const USAGE: &str = "\
whiskerd - wireless mouse battery levels for your status bar

USAGE:
    whiskerd [--output PATH|-]   run the daemon (default output: $XDG_RUNTIME_DIR/whiskerd.json;
                                 '-' prints one JSON line to stdout on every change)
    whiskerd --probe             query every supported receiver once and print what it says
    whiskerd --udev-rules        print the udev rule for all supported receivers
    whiskerd --list              list supported receivers
    whiskerd --version | --help
";

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

fn is_access_error(e: &io::Error) -> bool {
    matches!(e.raw_os_error(), Some(libc::EACCES) | Some(libc::EPERM))
}

/// One supported receiver that is plugged in.
struct Hid {
    found: Found,
    state: DeviceState,
    /// Keychron: open interface 1 that receives pushed battery events.
    events: Option<File>,
    /// Keychron: the interface-3 slot query has answered at least once.
    slots_known: bool,
    /// Set after logging an error, so a broken device logs once, not every poll.
    warned: bool,
    /// Compx: recent readings, so the highest of the last 45 minutes is shown.
    peak: smooth::PeakHold,
}

impl Hid {
    fn fail(&mut self, what: &str, e: &io::Error) {
        self.state.connected = false;
        if self.warned {
            return;
        }
        self.warned = true;
        if is_access_error(e) {
            eprintln!(
                "whiskerd: no access to {} yet (if this persists, install the udev rule: whiskerd --udev-rules)",
                self.found.devnode.display()
            );
        } else {
            eprintln!("whiskerd: {} {}: {e}", what, self.found.devnode.display());
        }
    }

    fn query_compx(&mut self, seq: u8) {
        match hidraw::compx_query(&self.found.devnode, seq) {
            Ok(Some((r, tag))) => {
                self.state.name = compx::model_name(&tag);
                let shown = self.peak.push(r, Instant::now());
                self.state.apply(shown, now());
                self.warned = false;
            }
            Ok(None) => self.state.connected = false, // mouse asleep or out of range; keep last value
            Err(e) => self.fail("query", &e),
        }
    }

    fn keychron_slots(&mut self) {
        let Some(iface3) =
            discovery::sibling_iface(&self.found, CLASS_HIDRAW.as_ref(), DEV.as_ref(), 3)
        else {
            return;
        };
        match hidraw::keychron_slots(&iface3) {
            Ok(Some((pid, connected))) => {
                self.state.name = keychron::model_name(pid).into();
                self.state.connected = connected;
                self.slots_known = true;
                self.warned = false;
            }
            Ok(None) => {}
            Err(e) => self.fail("slot query", &e),
        }
    }

    fn open_events(&mut self) {
        match hidraw::open(&self.found.devnode, false) {
            Ok(f) => {
                self.events = Some(f);
                self.warned = false;
            }
            Err(e) => self.fail("open", &e),
        }
    }

    /// Reads every pending pushed report. Returns false if the device went away.
    fn read_events(&mut self) -> bool {
        let Some(f) = self.events.as_mut() else {
            return true;
        };
        let mut buf = [0u8; 64];
        loop {
            match f.read(&mut buf) {
                Ok(0) => return true,
                Ok(n) => {
                    if let Some(r) = keychron::parse_event(&buf[..n]) {
                        self.state.apply(r, now());
                    }
                }
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => return true,
                Err(e) if e.kind() == io::ErrorKind::Interrupted => {}
                Err(_) => {
                    self.events = None;
                    return false;
                }
            }
        }
    }

    /// A Keychron receiver whose setup did not complete (e.g. the uaccess ACL was not applied yet
    /// at login, or interface 3 appeared after interface 1 during hotplug).
    fn needs_setup(&self) -> bool {
        self.found.rx.protocol == Protocol::KeychronLink
            && (self.events.is_none() || !self.slots_known)
    }

    /// Periodic refresh.
    fn refresh(&mut self, seq: u8) {
        match self.found.rx.protocol {
            Protocol::Compx => self.query_compx(seq),
            Protocol::KeychronLink => {
                if self.events.is_none() {
                    self.open_events();
                }
                self.keychron_slots();
            }
        }
    }
}

enum Output {
    File(PathBuf),
    Stdout,
}

struct Daemon {
    hids: Vec<Hid>,
    power: Vec<DeviceState>,
    out: Output,
    last: String,
    seq: u8,
}

impl Daemon {
    fn next_seq(&mut self) -> u8 {
        self.seq = self.seq.wrapping_add(1);
        self.seq
    }

    fn rescan_hid(&mut self) {
        let found = discovery::scan(CLASS_HIDRAW.as_ref(), DEV.as_ref());
        self.hids.retain(|h| {
            let keep = found
                .iter()
                .any(|f| f.devnode == h.found.devnode && f.rx == h.found.rx);
            if !keep {
                eprintln!(
                    "whiskerd: removed {} ({})",
                    h.state.name,
                    h.found.devnode.display()
                );
            }
            keep
        });
        for f in found {
            if self.hids.iter().any(|h| h.found.devnode == f.devnode) {
                continue;
            }
            let state = DeviceState::new(f.id(), f.rx.label.into(), "mouse");
            let mut h = Hid {
                found: f,
                state,
                events: None,
                slots_known: false,
                warned: false,
                peak: smooth::PeakHold::default(),
            };
            let seq = self.next_seq();
            h.refresh(seq);
            eprintln!(
                "whiskerd: found {} ({})",
                h.state.name,
                h.found.devnode.display()
            );
            self.hids.push(h);
        }
    }

    fn rescan_power(&mut self) {
        let mut fresh = power_supply::scan(CLASS_POWER.as_ref(), now());
        // Keep the old timestamp while nothing changed, so the file is only rewritten on change.
        for d in &mut fresh {
            if let Some(old) = self.power.iter().find(|o| o.id == d.id) {
                if (old.battery, old.charging) == (d.battery, d.charging) {
                    d.updated = old.updated;
                }
            }
        }
        self.power = fresh;
    }

    fn tick(&mut self) {
        self.rescan_hid(); // also covers systems without udev netlink events
        for i in 0..self.hids.len() {
            let seq = self.next_seq();
            self.hids[i].refresh(seq);
        }
        self.rescan_power();
    }

    /// Devices to retry soon: polled mice without a fresh reading (asleep), and Keychron
    /// receivers whose setup has not completed.
    fn waiting(&self) -> impl Iterator<Item = usize> + '_ {
        self.hids
            .iter()
            .enumerate()
            .filter(|(_, h)| {
                (h.found.rx.protocol == Protocol::Compx && !h.state.connected) || h.needs_setup()
            })
            .map(|(i, _)| i)
    }

    fn retry_waiting(&mut self) {
        let idx: Vec<usize> = self.waiting().collect();
        for i in idx {
            let seq = self.next_seq();
            self.hids[i].refresh(seq);
        }
    }

    /// Completes Keychron setup after a hotplug event (interface 3 may arrive late).
    fn retry_setup(&mut self) {
        for h in self.hids.iter_mut().filter(|h| h.needs_setup()) {
            h.refresh(0);
        }
    }

    fn publish(&mut self) {
        let all: Vec<DeviceState> = self
            .hids
            .iter()
            .map(|h| h.state.clone())
            .chain(self.power.iter().cloned())
            .collect();
        let json = state::render_json(&all);
        if json == self.last {
            return;
        }
        let res = match &self.out {
            Output::File(p) => state::write_atomic(p, &json),
            Output::Stdout => io::stdout()
                .lock()
                .write_all(json.as_bytes())
                .and_then(|_| io::stdout().flush()),
        };
        match res {
            Ok(()) => self.last = json,
            Err(e) => eprintln!("whiskerd: cannot write output: {e}"),
        }
    }
}

/// Blocks SIGTERM/SIGINT/SIGHUP and returns a signalfd for them, so shutdown is a poll event.
fn signal_fd() -> io::Result<OwnedFd> {
    unsafe {
        let mut set: libc::sigset_t = std::mem::zeroed();
        libc::sigemptyset(&mut set);
        for s in [libc::SIGTERM, libc::SIGINT, libc::SIGHUP] {
            libc::sigaddset(&mut set, s);
        }
        if libc::sigprocmask(libc::SIG_BLOCK, &set, std::ptr::null_mut()) < 0 {
            return Err(io::Error::last_os_error());
        }
        let fd = libc::signalfd(-1, &set, libc::SFD_NONBLOCK | libc::SFD_CLOEXEC);
        if fd < 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(OwnedFd::from_raw_fd(fd))
    }
}

fn run(out: Output) -> io::Result<()> {
    let sigfd = signal_fd()?;
    let netlink = netlink::open()
        .map_err(|e| eprintln!("whiskerd: no hotplug events ({e}); rescanning every 5 minutes"))
        .ok();
    let mut d = Daemon {
        hids: Vec::new(),
        power: Vec::new(),
        out,
        last: String::new(),
        seq: 0,
    };
    d.rescan_hid();
    d.rescan_power();
    d.publish();
    let mut next_tick = Instant::now() + POLL_INTERVAL;
    let mut next_retry = Instant::now() + RETRY_INTERVAL;

    loop {
        // Slot 0: signals, slot 1: netlink (or -1), then one slot per Keychron event fd.
        let mut fds: Vec<libc::pollfd> = vec![
            libc::pollfd {
                fd: sigfd.as_raw_fd(),
                events: libc::POLLIN,
                revents: 0,
            },
            libc::pollfd {
                fd: netlink.as_ref().map_or(-1, |n| n.as_raw_fd()),
                events: libc::POLLIN,
                revents: 0,
            },
        ];
        let mut owners: Vec<usize> = Vec::new();
        for (i, h) in d.hids.iter().enumerate() {
            if let Some(f) = &h.events {
                fds.push(libc::pollfd {
                    fd: f.as_raw_fd() as RawFd,
                    events: libc::POLLIN,
                    revents: 0,
                });
                owners.push(i);
            }
        }
        let deadline = if d.waiting().next().is_some() {
            next_tick.min(next_retry)
        } else {
            next_tick
        };
        let timeout = deadline
            .saturating_duration_since(Instant::now())
            .as_millis()
            .min(i32::MAX as u128) as i32;
        let rc = unsafe { libc::poll(fds.as_mut_ptr(), fds.len() as libc::nfds_t, timeout) };
        if rc < 0 {
            let e = io::Error::last_os_error();
            if e.kind() != io::ErrorKind::Interrupted {
                return Err(e);
            }
            continue;
        }

        if fds[0].revents != 0 {
            break;
        }
        let mut rescan = false;
        if fds[1].revents != 0 {
            if let Some(n) = &netlink {
                let c = netlink::drain(n);
                rescan |= c.hidraw;
                if c.power_supply {
                    d.rescan_power();
                }
            }
        }
        for (slot, &i) in owners.iter().enumerate() {
            let rev = fds[2 + slot].revents;
            if rev & libc::POLLIN != 0 && !d.hids[i].read_events() {
                rescan = true;
            }
            if rev & (libc::POLLHUP | libc::POLLERR | libc::POLLNVAL) != 0 {
                d.hids[i].events = None;
                rescan = true;
            }
        }
        if rescan {
            d.rescan_hid();
            d.retry_setup();
        }
        let now_i = Instant::now();
        if now_i >= next_tick {
            d.tick();
            next_tick = now_i + POLL_INTERVAL;
            next_retry = now_i + RETRY_INTERVAL;
        } else if now_i >= next_retry {
            d.retry_waiting();
            next_retry = now_i + RETRY_INTERVAL;
        }
        d.publish();
    }

    if let Output::File(p) = &d.out {
        let _ = std::fs::remove_file(p); // the panel hides itself when the daemon is stopped
    }
    Ok(())
}

fn probe() {
    let found = discovery::scan(CLASS_HIDRAW.as_ref(), DEV.as_ref());
    if found.is_empty() {
        println!("no supported receiver found (see `whiskerd --list`)");
    }
    for f in found {
        let tested = if f.rx.verified {
            "verified"
        } else {
            "community-reported"
        };
        println!(
            "{:04x}:{:04x} {} at {} [{tested}]",
            f.rx.vid,
            f.rx.pid,
            f.rx.label,
            f.devnode.display()
        );
        match f.rx.protocol {
            Protocol::Compx => match hidraw::compx_query(&f.devnode, 1) {
                Ok(Some((r, tag))) => println!(
                    "  {}: {}%{}",
                    compx::model_name(&tag),
                    r.battery,
                    if r.charging == Some(true) {
                        " (charging)"
                    } else {
                        ""
                    }
                ),
                Ok(None) => {
                    println!("  receiver answered, but the mouse gave no reading (asleep?)")
                }
                Err(e) => println!("  error: {e}"),
            },
            Protocol::KeychronLink => {
                match discovery::sibling_iface(&f, CLASS_HIDRAW.as_ref(), DEV.as_ref(), 3)
                    .map(|p| hidraw::keychron_slots(&p))
                {
                    Some(Ok(Some((pid, c)))) => println!(
                        "  paired: {} ({}). Battery arrives when the mouse wakes; run the daemon.",
                        keychron::model_name(pid),
                        if c { "connected" } else { "asleep" }
                    ),
                    Some(Ok(None)) | None => println!("  no slot information"),
                    Some(Err(e)) => println!("  error: {e}"),
                }
            }
        }
    }
    for p in power_supply::scan(CLASS_POWER.as_ref(), now()) {
        println!("kernel battery: {} [{}] {:?}%", p.name, p.kind, p.battery);
    }
}

fn list() {
    for r in devices::RECEIVERS {
        let tested = if r.verified {
            "verified"
        } else {
            "community-reported"
        };
        println!(
            "{:04x}:{:04x}  iface {}  {:?}  {}  [{tested}]",
            r.vid, r.pid, r.iface, r.protocol, r.label
        );
    }
    println!("plus any peripheral battery the kernel exposes in /sys/class/power_supply");
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let arg = |s: &str| args.iter().any(|a| a == s);
    if arg("--help") || arg("-h") {
        print!("{USAGE}");
        return;
    }
    if arg("--version") || arg("-V") {
        println!("whiskerd {}", env!("CARGO_PKG_VERSION"));
        return;
    }
    if arg("--udev-rules") {
        print!("{}", devices::udev_rules());
        return;
    }
    if arg("--list") {
        list();
        return;
    }
    if arg("--probe") {
        probe();
        return;
    }
    let out = match args
        .iter()
        .position(|a| a == "--output")
        .map(|i| args.get(i + 1))
    {
        Some(Some(p)) if p == "-" => Output::Stdout,
        Some(Some(p)) => Output::File(PathBuf::from(p)),
        Some(None) => {
            eprintln!("whiskerd: --output needs a path or '-'");
            std::process::exit(2);
        }
        None => {
            let xdg = std::env::var("XDG_RUNTIME_DIR").ok();
            Output::File(state::runtime_path(xdg.as_deref(), unsafe {
                libc::getuid()
            }))
        }
    };
    if let Output::File(p) = &out {
        if let Some(dir) = p
            .parent()
            .filter(|d| !d.as_os_str().is_empty() && !Path::new(d).is_dir())
        {
            eprintln!(
                "whiskerd: output directory {} does not exist",
                dir.display()
            );
            std::process::exit(1);
        }
    }
    if let Err(e) = run(out) {
        eprintln!("whiskerd: {e}");
        std::process::exit(1);
    }
}
