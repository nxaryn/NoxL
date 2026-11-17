//! NoxL Wayland compositor state.
//!
//! Holds the Smithay `Display`, the seat, the xdg-shell state, and the
//! NoxL-specific data (workspaces, outputs, config, IPC).
//!
//! The API matches Smithay 0.7 (see examples/minimal.rs).

use std::sync::Arc;

use smithay::{
    desktop::Space,
    input::{Seat, SeatState},
    reexports::wayland_server::{
        backend::{ClientData, ClientId, DisconnectReason},
        protocol::wl_surface::WlSurface,
        Client, Display, DisplayHandle,
    },
    utils::{Point, Rectangle, Size},
    wayland::{
        buffer::BufferHandler,
        compositor::{
            CompositorClientState, CompositorHandler, CompositorState,
        },
        selection::{
            data_device::{
                ClientDndGrabHandler, DataDeviceHandler, DataDeviceState,
                ServerDndGrabHandler,
            },
            SelectionHandler,
        },
        shell::xdg::{PopupSurface, PositionerState, ToplevelSurface, XdgShellHandler, XdgShellState},
        shm::{ShmHandler, ShmState},
    },
};

use crate::{ipc, window};

// Delegate the Wayland protocol globals to Smithay's implementations.
smithay::delegate_compositor!(Noxl);
smithay::delegate_data_device!(Noxl);
smithay::delegate_seat!(Noxl);
smithay::delegate_shm!(Noxl);
smithay::delegate_xdg_shell!(Noxl);

/// Per-client state.
#[derive(Default)]
pub struct ClientState {
    pub compositor_state: CompositorClientState,
}

impl ClientData for ClientState {
    fn initialized(&self, _client_id: ClientId) {}
    fn disconnected(&self, _client_id: ClientId, _reason: DisconnectReason) {}
}

/// A workspace (virtual desktop).
#[derive(Debug, Default)]
pub struct Workspace {
    pub index: u32,
    pub windows: Vec<smithay::desktop::Window>,
    pub active: bool,
}

/// A monitor/output.
#[derive(Debug)]
pub struct OutputState {
    pub output: smithay::output::Output,
    pub size: Size<i32, smithay::utils::Logical>,
    pub scale: i32,
    pub position: Point<i32, smithay::utils::Logical>,
}

/// The main compositor state.
pub struct Noxl {
    pub compositor_state: CompositorState,
    pub xdg_shell_state: XdgShellState,
    pub shm_state: ShmState,
    pub seat_state: SeatState<Self>,
    pub data_device_state: DataDeviceState,
    pub seat: Seat<Self>,
    pub space: Space<smithay::desktop::Window>,
    pub outputs: Vec<OutputState>,
    pub workspaces: Vec<Workspace>,
    pub active_workspace: u32,
    pub config: noxl_config::NoxlConfig,
    pub ipc: Option<ipc::IpcServer>,
    pub nested: bool,
    pub display: DisplayHandle,
}

impl Noxl {
    pub fn new(
        display: &Display<Self>,
        config: noxl_config::NoxlConfig,
        no_ipc: bool,
        nested: bool,
    ) -> anyhow::Result<Self> {
        let dh = display.handle();

        let compositor_state = CompositorState::new::<Self>(&dh);
        let xdg_shell_state = XdgShellState::new::<Self>(&dh);
        let shm_state = ShmState::new::<Self>(&dh, vec![]);
        let mut seat_state = SeatState::new();
        let seat = seat_state.new_wl_seat(&dh, "noxl");
        let data_device_state = DataDeviceState::new::<Self>(&dh);

        let mut state = Self {
            compositor_state,
            xdg_shell_state,
            shm_state,
            seat_state,
            data_device_state,
            seat,
            space: Space::default(),
            outputs: Vec::new(),
            workspaces: vec![Workspace {
                index: 0,
                windows: Vec::new(),
                active: true,
            }],
            active_workspace: 0,
            config,
            ipc: None,
            nested,
            display: dh.clone(),
        };

        // Set up IPC.
        if !no_ipc {
            match ipc::IpcServer::new() {
                Ok(server) => {
                    tracing::info!(path = %noxl_ipc::control_socket_path().display(), "IPC server started");
                    state.ipc = Some(server);
                }
                Err(e) => tracing::error!(error = %e, "failed to start IPC server"),
            }
        }

        Ok(state)
    }

    /// Focus a workspace by index.
    pub fn focus_workspace(&mut self, index: u32) {
        if index as usize >= self.workspaces.len() {
            return;
        }
        for (i, ws) in self.workspaces.iter_mut().enumerate() {
            ws.active = i == index as usize;
        }
        self.active_workspace = index;
        tracing::info!(index, "workspace focused");
        if let Some(ipc) = &mut self.ipc {
            ipc.send_event(noxl_ipc::Event::WorkspaceChanged { index });
        }
    }

    /// Handle an IPC command.
    pub fn handle_command(&mut self, cmd: noxl_ipc::Command) {
        use noxl_ipc::Command;
        match cmd {
            Command::Ping => {
                if let Some(ipc) = &mut self.ipc {
                    ipc.send_event(noxl_ipc::Event::Pong {
                        protocol: noxl_ipc::PROTOCOL_VERSION,
                    });
                }
            }
            Command::LauncherToggle => {
                tracing::debug!("launcher toggle requested");
            }
            Command::Lock => {
                tracing::debug!("lock requested");
            }
            Command::Spawn { app } => {
                window::spawn_app(&app);
            }
            Command::WorkspaceFocus { index } => {
                self.focus_workspace(index);
            }
            Command::WorkspaceNext => {
                let next = (self.active_workspace + 1) % self.workspaces.len() as u32;
                self.focus_workspace(next);
            }
            Command::WorkspacePrev => {
                let prev = if self.active_workspace == 0 {
                    self.workspaces.len() as u32 - 1
                } else {
                    self.active_workspace - 1
                };
                self.focus_workspace(prev);
            }
            Command::Screenshot { mode } => {
                tracing::info!(?mode, "screenshot requested");
            }
            Command::GetConfig => {
                if let Some(ipc) = &mut self.ipc {
                    let value = serde_json::to_value(&self.config).unwrap_or_default();
                    ipc.send_event(noxl_ipc::Event::Config(value));
                }
            }
            Command::SetConfig { config } => {
                if let Ok(new_config) = serde_json::from_value(config) {
                    self.config = new_config;
                    let _ = self.config.save();
                    tracing::info!("config replaced via IPC");
                }
            }
            Command::ReloadConfig => {
                match noxl_config::NoxlConfig::load() {
                    Ok(c) => {
                        self.config = c;
                        tracing::info!("config reloaded");
                        if let Some(ipc) = &mut self.ipc {
                            ipc.send_event(noxl_ipc::Event::ConfigReloaded);
                        }
                    }
                    Err(e) => tracing::error!(error = %e, "config reload failed"),
                }
            }
            Command::Notify { summary, body, app_icon } => {
                tracing::info!(%summary, "notification requested");
                if let Some(ipc) = &mut self.ipc {
                    ipc.send_event(noxl_ipc::Event::NotificationPosted { summary, body });
                }
            }
            Command::Quit => {
                tracing::info!("quit requested");
                std::process::exit(0);
            }
        }
    }
}

// --- Smithay handler trait implementations ---

impl BufferHandler for Noxl {
    fn buffer_destroyed(&mut self, _buffer: &smithay::reexports::wayland_server::protocol::wl_buffer::WlBuffer) {}
}

impl XdgShellHandler for Noxl {
    fn xdg_shell_state(&mut self) -> &mut XdgShellState {
        &mut self.xdg_shell_state
    }

    fn new_toplevel(&mut self, surface: ToplevelSurface) {
        // Activate the window and add it to the active workspace.
        surface.with_pending_state(|state| {
            state.states.set(smithay::reexports::wayland_protocols::xdg::shell::server::xdg_toplevel::State::Activated);
        });
        surface.send_configure();

        let window = smithay::desktop::Window::new(surface);
        if let Some(ws) = self.workspaces.iter_mut().find(|ws| ws.active) {
            ws.windows.push(window);
        }
        tracing::debug!("new toplevel surface");
    }

    fn new_popup(&mut self, _surface: PopupSurface, _positioner: PositionerState) {
        tracing::debug!("new popup surface");
    }

    fn grab(&mut self, _surface: PopupSurface, _seat: smithay::reexports::wayland_server::protocol::wl_seat::WlSeat, _serial: smithay::utils::Serial) {}

    fn reposition_request(&mut self, _surface: PopupSurface, _positioner: PositionerState, _token: u32) {}
}

impl SelectionHandler for Noxl {
    type SelectionUserData = ();
}

impl DataDeviceHandler for Noxl {
    fn data_device_state(&self) -> &DataDeviceState {
        &self.data_device_state
    }
}

impl ClientDndGrabHandler for Noxl {}
impl ServerDndGrabHandler for Noxl {
    fn send(&mut self, _mime_type: String, _fd: std::os::unix::io::OwnedFd, _seat: Seat<Self>) {}
}

impl CompositorHandler for Noxl {
    fn compositor_state(&mut self) -> &mut CompositorState {
        &mut self.compositor_state
    }

    fn client_compositor_state<'a>(&self, client: &'a Client) -> &'a CompositorClientState {
        &client.get_data::<ClientState>().unwrap().compositor_state
    }

    fn commit(&mut self, surface: &WlSurface) {
        smithay::backend::renderer::utils::on_commit_buffer_handler::<Self>(surface);
    }
}

impl ShmHandler for Noxl {
    fn shm_state(&self) -> &ShmState {
        &self.shm_state
    }
}

impl smithay::input::SeatHandler for Noxl {
    type KeyboardFocus = WlSurface;
    type PointerFocus = WlSurface;
    type TouchFocus = WlSurface;

    fn seat_state(&mut self) -> &mut SeatState<Self> {
        &mut self.seat_state
    }

    fn focus_changed(&mut self, _seat: &Seat<Self>, _focused: Option<&WlSurface>) {}
    fn cursor_image(&mut self, _seat: &Seat<Self>, _image: smithay::input::pointer::CursorImageStatus) {}
}
