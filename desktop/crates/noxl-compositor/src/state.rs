//! Global compositor state.
//!
//! Holds the Smithay `State`, the list of outputs (monitors), workspaces,
//! windows, and the registry of Wayland globals. This is the central data model
//! the rest of the compositor operates on.

use std::collections::HashMap;

use smithay::{
    delegate_compositor, delegate_data_device, delegate_output, delegate_seat,
    delegate_shm, delegate_xdg_shell,
    desktop::{
        layer_map, space::Space, Window, WindowSurfaceType,
    },
    input::{Seat, SeatState},
    output::{Output, PhysicalProperties, Subpixel},
    reexports::{
        calloop::EventLoop,
        wayland_server::{
            backend::{ClientData, ClientId, DisconnectedState},
            protocol::wl_surface::WlSurface,
            Display, DisplayHandle, GlobalDispatch,
        },
    },
    utils::{IsAlive, Point, Rectangle, Size},
    wayland::{
        compositor::CompositorState,
        data_device::DataDeviceState,
        output::OutputManagerState,
        shell::xdg::XdgShellState,
        shm::ShmState,
    },
};

use crate::{input, ipc, protocols, render, window};

/// A workspace (virtual desktop).
#[derive(Debug, Default)]
pub struct Workspace {
    /// Index (0-based).
    pub index: u32,
    /// Windows on this workspace.
    pub windows: Vec<Window>,
    /// Whether this workspace is active.
    pub active: bool,
}

/// A monitor/output with its own coordinate space.
#[derive(Debug)]
pub struct OutputState {
    pub output: Output,
    /// Logical size (in points).
    pub size: Size<i32, smithay::utils::Logical>,
    /// Scale factor.
    pub scale: i32,
    /// Position in the global compositor space.
    pub position: Point<i32, smithay::utils::Logical>,
}

/// The main compositor state.
pub struct Compositor {
    /// Smithay display handle.
    pub display: DisplayHandle,
    /// Event loop.
    pub event_loop: EventLoop<'static, Self>,
    /// Common state.
    pub common: CommonState,
    /// Seat (input devices).
    pub seat: Seat<Self>,
    /// Space for rendering windows.
    pub space: Space<Window>,
    /// Layer-shell surfaces (panel, launcher, notifications).
    pub layer_map: layer_map::LayerMap,
    /// Outputs (monitors).
    pub outputs: Vec<OutputState>,
    /// Workspaces.
    pub workspaces: Vec<Workspace>,
    /// Currently active workspace index.
    pub active_workspace: u32,
    /// Config.
    pub config: noxl_config::NoxlConfig,
    /// IPC server (control socket).
    pub ipc: Option<ipc::IpcServer>,
    /// Whether running nested.
    pub nested: bool,
    /// Pending window to focus after mapping.
    pub pending_focus: Option<Window>,
}

/// Common Smithay state shared across protocol implementations.
pub struct CommonState {
    pub compositor: CompositorState,
    pub xdg_shell: XdgShellState,
    pub shm: ShmState,
    pub output_manager: OutputManagerState,
    pub seat_state: SeatState<Self>,
    pub data_device: DataDeviceState,
}

impl ClientData for NoxlClientData {
    fn initialized(&self, _client_id: ClientId) {}
    fn disconnected(&self, _client_id: ClientId, _state: DisconnectedState) {}
}

/// Per-client data.
pub struct NoxlClientData {
    /// Whether the client has been told about the compositor's capabilities.
    pub compositor_ready: bool,
}

impl Compositor {
    /// Create a new compositor instance.
    pub fn new(
        config: noxl_config::NoxlConfig,
        no_ipc: bool,
        nested: bool,
    ) -> anyhow::Result<Self> {
        let event_loop = EventLoop::try_new()?;

        let display = Display::new(event_loop.handle())?;
        let display_handle = display.handle();

        // Initialize Wayland protocol globals.
        let compositor_state = CompositorState::new::<Self>(&display_handle);
        let xdg_shell = XdgShellState::new::<Self>(&display_handle);
        let shm = ShmState::new::<Self>(&display_handle, vec![]);
        let output_manager = OutputManagerState::new::<Self>(&display_handle);
        let seat_state = SeatState::new::<Self>(&display_handle, "seat-0");
        let data_device = DataDeviceState::new::<Self>(&display_handle);

        let seat = seat_state.add_seat();

        let mut compositor = Self {
            display: display_handle,
            event_loop,
            common: CommonState {
                compositor: compositor_state,
                xdg_shell,
                shm,
                output_manager,
                seat_state,
                data_device,
            },
            seat,
            space: Space::new(),
            layer_map: layer_map::LayerMap::default(),
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
            pending_focus: None,
        };

        // Set up input handling.
        input::init(&mut compositor)?;

        // Set up IPC control socket.
        if !no_ipc {
            match ipc::IpcServer::new() {
                Ok(server) => {
                    tracing::info!(path = %noxl_ipc::control_socket_path().display(), "IPC server started");
                    compositor.ipc = Some(server);
                }
                Err(e) => {
                    tracing::error!(error = %e, "failed to start IPC server");
                }
            }
        }

        Ok(compositor)
    }

    /// Run the compositor event loop.
    pub async fn run(&mut self) -> anyhow::Result<()> {
        // Create the initial output (the backend will add more).
        self.create_output("initial")?;

        loop {
            // Dispatch Wayland events.
            self.display
                .dispatch_clients(&mut self.common)
                .map_err(|e| anyhow::anyhow!("wayland dispatch error: {}", e))?;

            // Process IPC commands.
            if let Some(ipc) = &mut self.ipc {
                for cmd in ipc.poll_commands() {
                    self.handle_command(cmd);
                }
            }

            // Render a frame.
            render::render(self)?;

            // Flush and sleep briefly (the backend drives the actual frame timing).
            self.display.flush_clients()?;
            std::thread::sleep(std::time::Duration::from_millis(8));
        }
    }

    /// Create a new output (monitor).
    pub fn create_output(&mut self, name: &str) -> anyhow::Result<()> {
        let output = Output::new(
            name.to_string(),
            PhysicalProperties {
                size: Size::from((0, 0)),
                subpixel: Subpixel::Unknown,
            },
        );

        // In nested mode, use the window size; otherwise 1920x1080.
        let size = Size::from((1920, 1080));

        self.outputs.push(OutputState {
            output: output.clone(),
            size,
            scale: 1,
            position: Point::from((0, 0)),
        });

        // Commit the output to the output manager so clients see it.
        self.common.output_manager.connect(&output);

        tracing::info!(name, "output created");
        Ok(())
    }

    /// Handle an IPC command.
    fn handle_command(&mut self, cmd: noxl_ipc::Command) {
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
                // Toggle the launcher layer-shell surface.
                // (Full implementation in protocols::layer)
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
                    Err(e) => {
                        tracing::error!(error = %e, "config reload failed");
                    }
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

    /// Get the currently active workspace.
    pub fn active_workspace_mut(&mut self) -> &mut Workspace {
        &mut self.workspaces[self.active_workspace as usize]
    }
}
