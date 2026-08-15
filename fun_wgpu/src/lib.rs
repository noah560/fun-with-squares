use std::sync::Arc;
use std::time::{Duration, Instant};
use winit::{
    application::ApplicationHandler,
    event::*,
    event_loop::{ActiveEventLoop, EventLoop, ControlFlow},
    keyboard::{PhysicalKey}
};

pub use anyhow;

/// WGPU API
pub use wgpu;

/// Key codes
pub use winit::keyboard::KeyCode;

/// Mouse buttons
pub use winit::event::MouseButton;

/// A window
pub use winit::window::Window;

/// An application with one window
pub trait App: Sized {
    /// Get a mutable reference to the `Arc` containing the window.
    fn get_window(&mut self) -> &mut Arc<Window>;
    /// Create an instance of the app with a window handle.
    #[allow(async_fn_in_trait)]
    async fn new(window: Arc<Window>) -> anyhow::Result<Self>;
    /// Called on resizing
    fn on_resize(&mut self, width: u32, height: u32);
    /// Called on a keyboard event
    fn on_keyboard(&mut self, code: KeyCode, pressed: bool);
    /// Called when the mouse is moved.
    fn on_mouse_move(&mut self, x: f32, y: f32);
    /// Called when a mouse button is pressed or released.
    fn on_mouse(&mut self, button: MouseButton, state: bool);
    /// Return the number of frames per second
    fn get_fps(&self) -> u64;
    /// Update game logic
    fn update(&mut self);
    /// Render the window
    fn render(&mut self) -> anyhow::Result<()>;
}

/// A wrapper for an `App` with the functionality required by winit.
pub struct Wrapper<A: App> {
    app: Option<A>,
}

/// Basic methods for `Wrapper`.
impl<A: App + 'static> Wrapper<A> {
    /// Create an empty `Wrapper`.
    pub fn new() -> Self {
        Self { app: None }
    }
}

/// Make the `Wrapper` compatible with WGPU.
impl<A: App + 'static> ApplicationHandler<A> for Wrapper<A> {
    /// Runs when the app is opened (The naming is weird).
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        #[allow(unused_mut)]
        let mut window_attributes = Window::default_attributes().with_inner_size(winit::dpi::LogicalSize::new(400.0, 300.0));
        let window = Arc::new(event_loop.create_window(window_attributes).expect("Failed to create window!"));
        self.app = Some(pollster::block_on(A::new(window)).expect("Failed to create window!"));
    }
    /// Not needed for non wasm but required for the trait
    #[allow(unused_mut)]
    fn user_event(&mut self, _event_loop: &ActiveEventLoop, mut event: A) {
        self.app = Some(event);
    }
    /// An input event, etc
    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let state = match &mut self.app {
            Some(canvas) => canvas,
            None => return,
        };
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => state.on_resize(size.width, size.height),
            WindowEvent::RedrawRequested => {
                event_loop.set_control_flow(ControlFlow::WaitUntil(
                    Instant::now() + Duration::from_millis(1000_u64 / state.get_fps())
                ));
                state.render().expect("Error rendering!");
            }
            WindowEvent::KeyboardInput {
                event: KeyEvent {
                    physical_key: PhysicalKey::Code(code),
                    state: key_state,
                    ..
                },
                ..
            } => state.on_keyboard(code, key_state.is_pressed()),
            WindowEvent::CursorMoved {
                position,
                ..
            } => state.on_mouse_move(position.x as f32, position.y as f32),
            WindowEvent::MouseInput {
                state: button_state,
                button,
                ..
            } => state.on_mouse(button, button_state.is_pressed()),
            _ => {}
        }
    }
    fn new_events(
        &mut self,
        _event_loop: &ActiveEventLoop,
        cause: StartCause
    ) {
        match cause {
            StartCause::ResumeTimeReached {
                ..
            } => self.app.as_mut().unwrap().get_window().request_redraw(),
            _ => {}
        }
    }
}

/// Run the `Wrapper`.
pub fn run<A: App + 'static>() -> anyhow::Result<()> {
    env_logger::init();
    let event_loop = EventLoop::with_user_event().build()?;
    let mut app = Wrapper::<A>::new();
    event_loop.run_app(&mut app)?;
    Ok(())
}
