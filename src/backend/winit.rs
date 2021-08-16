use crate::state::Fireplace;
use anyhow::Result;
use calloop::{timer::Timer, EventLoop};
use smithay::{
    backend::{
        input::{InputBackend, InputEvent},
        winit,
    },
    reexports::wayland_server::protocol::wl_output::Subpixel,
    wayland::output::{Mode, PhysicalProperties},
};
use std::{
    sync::atomic::{AtomicUsize, Ordering},
    time::Duration,
};

static WINIT_COUNTER: AtomicUsize = AtomicUsize::new(0);

pub fn init_winit(event_loop: &mut EventLoop<Fireplace>, state: &Fireplace) -> Result<()> {
    let (renderer, input) = match winit::init(None) {
        Ok(ret) => ret,
        Err(err) => {
            slog_scope::crit!("Failed to initialize winit backend: {}", err);
            return Err(err.into());
        }
    };

    let id = WINIT_COUNTER.fetch_add(1, Ordering::SeqCst);
    let name = format!("WINIT-{}", id);

    let size = renderer.window_size();
    let props = PhysicalProperties {
        size: (0, 0).into(),
        subpixel: Subpixel::Unknown,
        make: String::from("WINIT"),
        model: format!("{}", id),
    };
    let mode = Mode {
        size: size.physical_size,
        refresh: 60_000,
    };
    state
        .workspaces
        .borrow_mut()
        .add_output(name.clone(), props, mode);

    let timer = Timer::new()?;
    let timer_handle = timer.handle();
    event_loop
        .handle()
        .insert_source(
            timer,
            move |(mut input, mut renderer): (
                winit::WinitInputBackend,
                winit::WinitGraphicsBackend,
            ),
                  handle,
                  state| {
                match input.dispatch_new_events(|event| state.process_winit_event(&name, event)) {
                    Ok(()) => {
                        if let Err(err) = renderer
                            .render(|renderer, frame| state.render_output(&name, renderer, frame))
                            .and_then(|x| x)
                        {
                            slog_scope::error!("Failed to render frame: {}", err);
                        };
                        handle.add_timeout(Duration::from_millis(16), (input, renderer));
                    }
                    Err(winit::WinitInputError::WindowClosed) => {
                        state.workspaces.borrow_mut().remove_output_by_name(&name);
                        slog_scope::debug!("Removed {}", name);
                    }
                }
            },
        )
        .map_err(|_| anyhow::anyhow!("Failed to init eventloop timer for winit"))?;
    timer_handle.add_timeout(Duration::ZERO, (input, renderer));
    Ok(())
}

impl Fireplace {
    pub fn process_winit_event<B>(&mut self, name: &str, event: InputEvent<B>)
    where
        B: InputBackend<SpecialEvent = smithay::backend::winit::WinitEvent>,
    {
        use smithay::backend::winit::WinitEvent;

        match event {
            InputEvent::Special(WinitEvent::Resized { size, .. }) => {
                let mut workspaces = self.workspaces.borrow_mut();
                if let Some(output) = workspaces.output_by_name(&name) {
                    output.set_mode(smithay::wayland::output::Mode {
                        size,
                        refresh: 60_000,
                    });
                }

                if let Some(space) = workspaces.space_by_output_name(&name) {
                    space.rearrange(&size.to_logical(1));
                };
            }
            x => self.process_input_event(x),
        }
    }
}
