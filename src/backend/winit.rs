use crate::{
    backend::render::render_space,
    state::Fireplace,
};
use anyhow::Result;
use smithay::{
    backend::{
        input::{InputBackend, InputEvent},
        renderer::{ImportDma, ImportEgl},
        winit,
    },
    reexports::{
        calloop::{timer::Timer, EventLoop},
        wayland_server::protocol::wl_output::Subpixel,
    },
    wayland::{
        dmabuf::init_dmabuf_global,
        output::{Mode, PhysicalProperties},
    },
};
use std::{
    cell::RefCell,
    rc::Rc,
    sync::atomic::{AtomicUsize, Ordering},
    time::Duration,
};

static WINIT_COUNTER: AtomicUsize = AtomicUsize::new(0);

pub fn init_winit(event_loop: &mut EventLoop<Fireplace>, state: &mut Fireplace) -> Result<()> {
    let (renderer, input) = match winit::init(None) {
        Ok(ret) => ret,
        Err(err) => {
            slog_scope::crit!("Failed to initialize winit backend: {}", err);
            return Err(err.into());
        }
    };
    let renderer = Rc::new(RefCell::new(renderer));

    if renderer
        .borrow_mut()
        .renderer()
        .bind_wl_display(&state.display.borrow())
        .is_ok()
    {
        slog_scope::info!("EGL hardware-acceleration enabled");
        let dmabuf_formats = renderer
            .borrow_mut()
            .renderer()
            .dmabuf_formats()
            .cloned()
            .collect::<Vec<_>>();
        let renderer = renderer.clone();
        init_dmabuf_global(
            &mut *state.display.borrow_mut(),
            dmabuf_formats,
            move |buffer, _| renderer.borrow_mut().renderer().import_dmabuf(buffer).is_ok(),
            slog_scope::logger(),
        );
    };

    let id = WINIT_COUNTER.fetch_add(1, Ordering::SeqCst);
    let name = format!("WINIT-{}", id);

    let size = renderer.borrow().window_size();
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
    let token = event_loop
        .handle()
        .insert_source(
            timer,
            move |(mut input, renderer): (
                winit::WinitInputBackend,
                Rc<RefCell<winit::WinitGraphicsBackend>>,
            ),
            handle,
            state| {
                match input.dispatch_new_events(|event| state.process_winit_event(&name, event)) {
                    Ok(()) => {
                        let mut workspaces = state.workspaces.borrow_mut();
                        let scale = workspaces.output_by_name(&name).unwrap().scale();
                        let space = workspaces.space_by_output_name(&name).unwrap();
                        let popups = state.popups.borrow();
                        if let Err(err) = renderer
                            .borrow_mut()
                            .render(|renderer, frame| render_space(&**space, scale, &**popups, id as u64, renderer, frame))
                            .and_then(|x| x.map_err(Into::into))
                        {
                            slog_scope::error!("Failed to render frame: {}", err);
                        };
                        space.send_frames(state.start_time.elapsed().as_millis() as u32);
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
    state.tokens.push(token);

    Ok(())
}

impl Fireplace {
    pub fn process_winit_event<B>(&mut self, name: &str, event: InputEvent<B>)
    where
        B: InputBackend<SpecialEvent = smithay::backend::winit::WinitEvent>,
    {
        use smithay::backend::winit::WinitEvent;

        match event {
            InputEvent::Special(WinitEvent::Resized { size, scale_factor }) => {
                let mut workspaces = self.workspaces.borrow_mut();
                if let Some(output) = workspaces.output_by_name(&name) {
                    output.set_mode(smithay::wayland::output::Mode {
                        size,
                        refresh: 60_000,
                    });
                }

                let _scale = scale_factor;
                if let Some(space) = workspaces.space_by_output_name(&name) {
                    space.rearrange(&size.to_logical(1));
                };
            }
            x => self.process_input_event(x),
        }
    }
}
