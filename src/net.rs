//! Co-op multiplayer over TCP. The host runs the game and streams snapshots
//! to everyone in the party; clients send the actions their player takes.
//!
//! Messages are bincode, each prefixed with its length as a little-endian u32.

use std::io::{self, ErrorKind, Read, Write};
use std::net::{IpAddr, Shutdown, SocketAddr, TcpListener, TcpStream, ToSocketAddrs, UdpSocket};
use std::sync::mpsc::{self, Receiver};
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::audio::Sfx;
use crate::game::{Action, Game};

pub const PORT: u16 = 7777;
pub const MAX_PLAYERS: usize = 4;
/// How often the host sends snapshots.
pub const SNAPSHOT_INTERVAL: f32 = 1.0 / 30.0;
const MAX_MESSAGE: usize = 16 * 1024 * 1024;

#[derive(Serialize, Deserialize)]
enum ToClient {
    Snapshot {
        game: Box<Game>,
        sounds: Vec<Sfx>,
    },
    /// The party is full.
    Full,
}

#[derive(Serialize, Deserialize)]
enum ToHost {
    Action(Action),
}

/// A framed, non-blocking connection.
struct Conn {
    stream: TcpStream,
    inbuf: Vec<u8>,
    outbuf: Vec<u8>,
    /// The other side hung up; report it once buffered messages are read.
    closed: bool,
}

impl Conn {
    fn new(stream: TcpStream) -> io::Result<Self> {
        stream.set_nonblocking(true)?;
        stream.set_nodelay(true)?;
        Ok(Self {
            stream,
            inbuf: Vec::new(),
            outbuf: Vec::new(),
            closed: false,
        })
    }

    fn queue<T: Serialize>(&mut self, msg: &T) -> io::Result<()> {
        let bytes = bincode::serialize(msg).map_err(io::Error::other)?;
        self.outbuf
            .extend_from_slice(&(bytes.len() as u32).to_le_bytes());
        self.outbuf.extend_from_slice(&bytes);
        self.flush()
    }

    /// Writes as much queued data as the socket takes right now.
    fn flush(&mut self) -> io::Result<()> {
        while !self.outbuf.is_empty() {
            match self.stream.write(&self.outbuf) {
                Ok(0) => return Err(ErrorKind::WriteZero.into()),
                Ok(n) => {
                    self.outbuf.drain(..n);
                }
                Err(e) if e.kind() == ErrorKind::WouldBlock => break,
                Err(e) if e.kind() == ErrorKind::Interrupted => {}
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }

    /// Reads every complete message that has arrived.
    fn receive<T: for<'de> Deserialize<'de>>(&mut self) -> io::Result<Vec<T>> {
        if self.closed {
            return Err(ErrorKind::ConnectionReset.into());
        }
        let mut chunk = [0u8; 64 * 1024];
        loop {
            match self.stream.read(&mut chunk) {
                Ok(0) => {
                    self.closed = true;
                    break;
                }
                Ok(n) => self.inbuf.extend_from_slice(&chunk[..n]),
                Err(e) if e.kind() == ErrorKind::WouldBlock => break,
                Err(e) if e.kind() == ErrorKind::Interrupted => {}
                Err(e) => return Err(e),
            }
        }
        let mut out = Vec::new();
        while self.inbuf.len() >= 4 {
            let len = u32::from_le_bytes(self.inbuf[..4].try_into().unwrap()) as usize;
            if len > MAX_MESSAGE {
                return Err(io::Error::new(ErrorKind::InvalidData, "message too large"));
            }
            if self.inbuf.len() < 4 + len {
                break;
            }
            let msg = bincode::deserialize(&self.inbuf[4..4 + len])
                .map_err(|e| io::Error::new(ErrorKind::InvalidData, e))?;
            self.inbuf.drain(..4 + len);
            out.push(msg);
        }
        if self.closed && out.is_empty() {
            return Err(ErrorKind::ConnectionReset.into());
        }
        Ok(out)
    }
}

/// What happened on the host side during one poll.
#[derive(Default)]
pub struct HostEvents {
    pub actions: Vec<Action>,
    pub joined: usize,
    pub left: usize,
}

pub struct Host {
    listener: TcpListener,
    clients: Vec<Conn>,
    pub port: u16,
}

impl Host {
    pub fn start(port: u16) -> io::Result<Self> {
        let listener = TcpListener::bind(("0.0.0.0", port))?;
        listener.set_nonblocking(true)?;
        let port = listener.local_addr()?.port();
        Ok(Self {
            listener,
            clients: Vec::new(),
            port,
        })
    }

    /// Players in the party, including the host.
    pub fn players(&self) -> usize {
        self.clients.len() + 1
    }

    /// Accepts new players and collects their actions.
    pub fn poll(&mut self) -> HostEvents {
        let mut events = HostEvents::default();
        while let Ok((stream, _)) = self.listener.accept() {
            let Ok(mut conn) = Conn::new(stream) else {
                continue;
            };
            if self.players() >= MAX_PLAYERS {
                let _ = conn.queue(&ToClient::Full);
                let _ = conn.stream.shutdown(Shutdown::Write);
                continue;
            }
            self.clients.push(conn);
            events.joined += 1;
        }
        self.clients.retain_mut(|c| match c.receive::<ToHost>() {
            Ok(msgs) => {
                for ToHost::Action(a) in msgs {
                    events.actions.push(a);
                }
                true
            }
            Err(_) => {
                events.left += 1;
                false
            }
        });
        events
    }

    /// Sends the current game to everyone. Slow clients skip snapshots
    /// instead of building up a backlog.
    pub fn broadcast(&mut self, game: &Game, sounds: &[Sfx]) {
        if self.clients.is_empty() {
            return;
        }
        let msg = BorrowedToClient::Snapshot { game, sounds };
        let Ok(bytes) = bincode::serialize(&msg) else {
            return;
        };
        for c in &mut self.clients {
            let _ = c.flush();
            if c.outbuf.is_empty() {
                c.outbuf
                    .extend_from_slice(&(bytes.len() as u32).to_le_bytes());
                c.outbuf.extend_from_slice(&bytes);
                let _ = c.flush();
            }
        }
    }
}

/// Encodes exactly like `ToClient::Snapshot` without cloning the game.
#[derive(Serialize)]
enum BorrowedToClient<'a> {
    Snapshot { game: &'a Game, sounds: &'a [Sfx] },
}

pub enum ClientEvent {
    Snapshot(Box<Game>, Vec<Sfx>),
    Full,
}

pub struct Client {
    conn: Conn,
}

impl Client {
    /// Connects in a background thread so the menu keeps drawing.
    pub fn connect_async(addr: String) -> Receiver<io::Result<Client>> {
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let _ = tx.send(Self::connect(&addr));
        });
        rx
    }

    pub fn connect(addr: &str) -> io::Result<Client> {
        let addr = parse_address(addr)?;
        let stream = TcpStream::connect_timeout(&addr, Duration::from_secs(4))?;
        Ok(Client {
            conn: Conn::new(stream)?,
        })
    }

    pub fn send(&mut self, action: Action) -> io::Result<()> {
        self.conn.queue(&ToHost::Action(action))
    }

    /// Returns what arrived since the last poll; only the newest snapshot
    /// matters, so older ones are dropped.
    pub fn poll(&mut self) -> io::Result<Vec<ClientEvent>> {
        self.conn.flush()?;
        let mut sounds = Vec::new();
        let mut latest = None;
        let mut out = Vec::new();
        for msg in self.conn.receive::<ToClient>()? {
            match msg {
                ToClient::Snapshot { game, sounds: s } => {
                    sounds.extend(s);
                    latest = Some(game);
                }
                ToClient::Full => out.push(ClientEvent::Full),
            }
        }
        if let Some(game) = latest {
            out.push(ClientEvent::Snapshot(game, sounds));
        }
        Ok(out)
    }
}

/// Accepts "host", "host:port", "1.2.3.4" or "1.2.3.4:port".
fn parse_address(input: &str) -> io::Result<SocketAddr> {
    let input = input.trim();
    if input.is_empty() {
        return Err(io::Error::new(
            ErrorKind::InvalidInput,
            "enter the host's address",
        ));
    }
    let with_port = if input.contains(':') {
        input.to_string()
    } else {
        format!("{input}:{PORT}")
    };
    with_port
        .to_socket_addrs()?
        .next()
        .ok_or_else(|| io::Error::new(ErrorKind::NotFound, "address not found"))
}

/// This machine's address on the local network, to tell friends where to join.
pub fn local_ip() -> Option<IpAddr> {
    // No packets are sent; connecting a UDP socket just picks a route.
    let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    let ip = socket.local_addr().ok()?.ip();
    (!ip.is_unspecified()).then_some(ip)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::tests::{grass, run};
    use crate::tower::TowerKind;
    use std::time::Instant;

    fn wait_for<T>(mut f: impl FnMut() -> Option<T>) -> T {
        let start = Instant::now();
        loop {
            if let Some(v) = f() {
                return v;
            }
            assert!(start.elapsed() < Duration::from_secs(5), "timed out");
            std::thread::sleep(Duration::from_millis(5));
        }
    }

    #[test]
    fn client_actions_reach_host_and_snapshots_come_back() {
        let mut host = Host::start(0).unwrap();
        let mut game = Game::new();
        let mut client = Client::connect(&format!("127.0.0.1:{}", host.port)).unwrap();
        wait_for(|| (host.poll().joined == 1).then_some(()));
        assert_eq!(host.players(), 2);

        let spot = grass(&game);
        client
            .send(Action::Place {
                kind: TowerKind::Arrow,
                pos: spot,
            })
            .unwrap();
        client.send(Action::StartWave).unwrap();
        let actions = wait_for(|| {
            let a = host.poll().actions;
            (!a.is_empty()).then_some(a)
        });
        let mut actions = actions.into_iter();
        for a in actions.by_ref() {
            game.apply(a);
        }
        // The second action may arrive in a later poll.
        if game.wave == 0 {
            for a in wait_for(|| {
                let a = host.poll().actions;
                (!a.is_empty()).then_some(a)
            }) {
                game.apply(a);
            }
        }
        assert_eq!(game.towers.len(), 1);
        assert_eq!(game.wave, 1);
        run(&mut game, 2.0);

        host.broadcast(&game, &[Sfx::WaveStart]);
        let (snapshot, sounds) = wait_for(|| {
            client.poll().unwrap().into_iter().find_map(|e| match e {
                ClientEvent::Snapshot(g, s) => Some((g, s)),
                ClientEvent::Full => None,
            })
        });
        let mut mirror = Game::new();
        mirror.adopt(*snapshot);
        assert_eq!(mirror.towers.len(), 1);
        assert_eq!(mirror.wave, 1);
        assert_eq!(mirror.enemies.len(), game.enemies.len());
        assert_eq!(sounds, vec![Sfx::WaveStart]);

        drop(client);
        wait_for(|| (host.poll().left == 1).then_some(()));
        assert_eq!(host.players(), 1);
    }

    #[test]
    fn party_is_capped() {
        let mut host = Host::start(0).unwrap();
        let addr = format!("127.0.0.1:{}", host.port);
        let clients: Vec<Client> = (0..MAX_PLAYERS - 1)
            .map(|_| Client::connect(&addr).unwrap())
            .collect();
        let mut joined = 0;
        wait_for(|| {
            joined += host.poll().joined;
            (joined == MAX_PLAYERS - 1).then_some(())
        });
        let mut extra = Client::connect(&addr).unwrap();
        wait_for(|| {
            host.poll();
            extra
                .poll()
                .ok()
                .and_then(|ev| ev.into_iter().find(|e| matches!(e, ClientEvent::Full)))
        });
        assert_eq!(host.players(), MAX_PLAYERS);
        drop(clients);
    }

    #[test]
    fn addresses() {
        assert_eq!(parse_address("127.0.0.1").unwrap().port(), PORT);
        assert_eq!(parse_address("127.0.0.1:9000").unwrap().port(), 9000);
        assert!(parse_address("  ").is_err());
    }
}
