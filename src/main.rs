// Release builds on Windows run without a console window.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod audio;
mod enemy;
mod game;
mod map;
mod net;
mod render;
mod tower;
mod ui;
mod wave;

use std::io;
use std::sync::mpsc::{Receiver, TryRecvError};

use macroquad::prelude::*;

use audio::{Audio, Sfx};
use game::{Game, Request};
use net::{Client, ClientEvent, Host};
use ui::{Lobby, LobbyAction};

/// Shown in the window title and settings menu, e.g. "v4.0".
pub fn version_label() -> String {
    let major = env!("CARGO_PKG_VERSION_MAJOR");
    let minor = env!("CARGO_PKG_VERSION_MINOR");
    format!("v{major}.{minor}")
}

fn window_conf() -> Conf {
    Conf {
        window_title: format!("Rusty Tower Defense {}", version_label()),
        window_width: map::SCREEN_W as i32,
        window_height: map::SCREEN_H as i32,
        window_resizable: false,
        ..Default::default()
    }
}

enum Role {
    Offline,
    Host(Host),
    Client(Client),
}

enum Screen {
    Menu,
    /// Connected to a host, waiting for the first snapshot.
    Joining,
    Playing,
}

struct App {
    game: Game,
    lobby: Lobby,
    role: Role,
    screen: Screen,
    audio: Audio,
    connecting: Option<Receiver<io::Result<Client>>>,
    /// Sounds the host has played since its last snapshot.
    pending_sounds: Vec<Sfx>,
    snapshot_timer: f32,
    /// This machine's LAN address, shown to the host so friends can join.
    host_ip: String,
}

impl App {
    fn play_sounds(&mut self, sounds: &[Sfx]) {
        let now = get_time();
        for &s in sounds {
            self.audio
                .play(s, self.game.tower_volume, self.game.game_volume, now);
        }
    }

    fn back_to_menu(&mut self, status: Option<String>) {
        self.role = Role::Offline;
        self.screen = Screen::Menu;
        self.game.online = false;
        self.game.is_client = false;
        self.game.party = None;
        self.game.menu = None;
        self.lobby.busy = false;
        self.lobby.status = status;
    }

    fn update_menu(&mut self) {
        let a = ui::handle_lobby(&mut self.lobby, &mut self.game);
        self.update_menu_with(a);
    }

    fn update_menu_with(&mut self, action: Option<LobbyAction>) {
        match action {
            Some(LobbyAction::Play) => {
                self.game.restart(self.lobby.map, self.lobby.sandbox);
                self.role = Role::Offline;
                self.screen = Screen::Playing;
            }
            Some(LobbyAction::Host) => match Host::start(net::PORT) {
                Ok(host) => {
                    self.host_ip = net::local_ip()
                        .map_or("this computer's IP".to_string(), |ip| ip.to_string());
                    self.game.restart(self.lobby.map, self.lobby.sandbox);
                    self.game.online = true;
                    self.role = Role::Host(host);
                    self.screen = Screen::Playing;
                    self.game.notify("Party open: friends can join now");
                }
                Err(e) => {
                    self.lobby.status = Some(format!("Couldn't host on port {}: {e}", net::PORT))
                }
            },
            Some(LobbyAction::Join(address)) => {
                self.lobby.busy = true;
                self.lobby.status = Some(format!("Connecting to {address}..."));
                self.connecting = Some(Client::connect_async(address));
            }
            Some(LobbyAction::Quit) => std::process::exit(0),
            None => {}
        }

        if let Some(rx) = &self.connecting {
            match rx.try_recv() {
                Ok(Ok(client)) => {
                    self.connecting = None;
                    self.role = Role::Client(client);
                    self.screen = Screen::Joining;
                    self.lobby.status = Some("Connected, joining party...".to_string());
                }
                Ok(Err(e)) => {
                    self.connecting = None;
                    self.lobby.busy = false;
                    self.lobby.status = Some(format!("Couldn't join: {e}"));
                }
                Err(TryRecvError::Empty) => {}
                Err(TryRecvError::Disconnected) => {
                    self.connecting = None;
                    self.lobby.busy = false;
                }
            }
        }
    }

    fn poll_client(&mut self) {
        let Role::Client(client) = &mut self.role else {
            return;
        };
        match client.poll() {
            Ok(events) => {
                for event in events {
                    match event {
                        ClientEvent::Snapshot(snapshot, sounds) => {
                            self.game.adopt(*snapshot);
                            self.game.online = true;
                            self.game.is_client = true;
                            self.game.party =
                                Some(format!("In a party  -  host {}", self.lobby.address.trim()));
                            if matches!(self.screen, Screen::Joining) {
                                self.screen = Screen::Playing;
                                self.lobby.busy = false;
                                self.lobby.status = None;
                                self.game.notify("Joined the party");
                                if std::env::var("MP").as_deref() == Ok("client") {
                                    let want = vec2(700.0, 300.0);
                                    let mut best = want;
                                    let mut bd = f32::MAX;
                                    for i in -25..25 {
                                        for j in -25..25 {
                                            let p = want + vec2(i as f32 * 4.0, j as f32 * 4.0);
                                            if p.distance(want) < bd && self.game.can_place(p) {
                                                bd = p.distance(want);
                                                best = p;
                                            }
                                        }
                                    }
                                    self.game.act(game::Action::Place {
                                        kind: tower::TowerKind::Sniper,
                                        pos: best,
                                    });
                                }
                            }
                            self.play_sounds(&sounds);
                        }
                        ClientEvent::Full => {
                            self.back_to_menu(Some("That party is full".to_string()));
                            return;
                        }
                    }
                }
            }
            Err(_) => {
                let msg = if matches!(self.screen, Screen::Joining) {
                    "The host closed the connection"
                } else {
                    "Lost connection to the host"
                };
                self.back_to_menu(Some(msg.to_string()));
            }
        }
    }

    fn update_game(&mut self) {
        ui::handle_input(&mut self.game);

        // Route this player's actions.
        let actions = std::mem::take(&mut self.game.outbox);
        let mut lost = false;
        for action in actions {
            match &mut self.role {
                Role::Client(client) => lost |= client.send(action).is_err(),
                _ => self.game.apply(action),
            }
        }
        if lost {
            self.back_to_menu(Some("Lost connection to the host".to_string()));
            return;
        }

        if let Role::Host(host) = &mut self.role {
            let events = host.poll();
            for action in events.actions {
                self.game.apply(action);
            }
            let players = host.players();
            if events.joined > 0 {
                self.game.notify("A player joined the party");
            }
            if events.left > 0 {
                self.game.notify("A player left the party");
            }
            self.game.party = Some(format!(
                "Hosting on {}:{}  -  {players}/{} players",
                self.host_ip,
                host.port,
                net::MAX_PLAYERS
            ));
        }
        self.poll_client();

        // Clamp the frame time so a stall doesn't teleport enemies, and run
        // several fixed steps for fast-forward rather than one big one.
        let dt = get_frame_time().min(1.0 / 20.0);
        let is_client = matches!(self.role, Role::Client(_));
        for _ in 0..self.game.speed {
            if is_client {
                self.game.client_tick(dt);
            } else {
                self.game.update(dt);
            }
        }

        let sounds = std::mem::take(&mut self.game.sounds);
        if !is_client {
            // Clients play the host's sounds when snapshots arrive instead.
            self.play_sounds(&sounds);
        }
        if let Role::Host(host) = &mut self.role {
            self.pending_sounds.extend(sounds);
            self.snapshot_timer += get_frame_time();
            if self.snapshot_timer >= net::SNAPSHOT_INTERVAL {
                self.snapshot_timer = 0.0;
                host.broadcast(&self.game, &self.pending_sounds);
                self.pending_sounds.clear();
            }
        }

        if self.game.request.take() == Some(Request::MainMenu) {
            self.back_to_menu(None);
        }
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut app = App {
        game: Game::new(),
        lobby: Lobby::new(),
        role: Role::Offline,
        screen: Screen::Menu,
        audio: Audio::load().await,
        connecting: None,
        pending_sounds: Vec::new(),
        snapshot_timer: 0.0,
        host_ip: String::new(),
    };

    {
        let mp = std::env::var("MP").unwrap_or_default();
        if mp == "host" {
            app.lobby.map = 1;
            app.update_menu_with(Some(LobbyAction::Host));
            app.game.gold = 5000;
            for (k, x, y) in [
                (tower::TowerKind::Arrow, 230.0, 300.0),
                (tower::TowerKind::Cannon, 330.0, 250.0),
                (tower::TowerKind::Frost, 520.0, 380.0),
            ] {
                let want = vec2(x, y);
                let mut best = want;
                let mut bd = f32::MAX;
                for i in -25..25 {
                    for j in -25..25 {
                        let p = want + vec2(i as f32 * 4.0, j as f32 * 4.0);
                        if p.distance(want) < bd && app.game.can_place(p) {
                            bd = p.distance(want);
                            best = p;
                        }
                    }
                }
                app.game.place(k, best);
            }
            app.game.apply(game::Action::ToggleAuto);
        } else if mp == "client" {
            app.update_menu_with(Some(LobbyAction::Join("127.0.0.1".into())));
        }
    }
    loop {
        match app.screen {
            Screen::Menu => {
                app.update_menu();
                render::draw_lobby(&app.lobby, &app.game);
            }
            Screen::Joining => {
                app.poll_client();
                render::draw_lobby(&app.lobby, &app.game);
            }
            Screen::Playing => {
                app.update_game();
                if matches!(app.screen, Screen::Playing) {
                    render::draw(&app.game);
                } else {
                    render::draw_lobby(&app.lobby, &app.game);
                }
            }
        }
        next_frame().await
    }
}
