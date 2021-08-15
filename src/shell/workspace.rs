use crate::handler::ActiveOutput;
use std::{
    cell::RefCell,
    rc::Rc,
};

use linked_hash_map::LinkedHashMap;
use smithay::{
    reexports::{
        wayland_server::{
            Display,
            protocol::{
                wl_surface::WlSurface,
                wl_output,
            },
        },
    },
    wayland::{
        seat::Seat,
        output::{Mode, PhysicalProperties},
    },
    utils::{Size, Logical},
};

use crate::shell::{
    layout::Layout,
    output::Output,
    window::Kind,
};

pub struct Workspaces {
    display: Rc<RefCell<Display>>,
    spaces: LinkedHashMap<u8, Box<dyn Layout>>,
    outputs: Vec<Output>,
}

struct ActiveWorkspace(u8);

impl Workspaces {
    pub fn new(display: Rc<RefCell<Display>>) -> Workspaces {
        Workspaces {
            display,
            spaces: LinkedHashMap::new(),
            outputs: Vec::new(),
        }
    }

    fn create_workspace(&mut self, i: u8, size: Size<i32, Logical>) {
        slog_scope::info!("Creating workspace {}", i);
        self.spaces.insert(i, Box::new(super::layout::Floating::new(size)));
    }

    fn next_available(&mut self, size: Size<i32, Logical>) -> u8 {
        for i in 0..::std::u8::MAX {
            if let Some(space) = self.spaces.get_mut(&i) {
                let mut available = true;
                for output in &self.outputs {
                    if output.userdata().get::<ActiveWorkspace>().map(|x| x.0 as i32).unwrap_or(-1) == i as i32 {
                        available = false;
                    }
                }
                if available {
                    space.rearrange(&size);
                    return i;
                }
            } else {
                self.create_workspace(i, size);
                return i;
            }
        }
        0
    }

    pub fn arrange(&mut self) {
        // Recalculate the outputs location
        // TODO: handle vertical stacking monitors later
        let mut output_x = 0;
        for output in self.outputs.iter_mut() {
            output.set_location((output_x, 0).into());
            output_x += output.size().w;
        }
    }

    pub fn width(&self) -> i32 {
        self.outputs.iter().map(|x| x.size().w).sum()
    }
    
    pub fn add_output<N>(&mut self, name: N, physical: PhysicalProperties, mode: Mode) -> &Output
    where
        N: AsRef<str>,
    {
        // Append the output to the end of the existing
        // outputs by placing it after the current overall
        // width
        let location = (self.width(), 0);

        let output = Output::new(
            name,
            location.into(),
            &mut *self.display.borrow_mut(),
            physical,
            mode,
        );
        let logical_size = output.geometry().size;
        let workspace = self.next_available(logical_size);
        slog_scope::info!("New output: {:?}", output);
        slog_scope::debug!("Attaching workspace {} to output {}", workspace, output.name());
        output.userdata().insert_if_missing(|| ActiveWorkspace(workspace));
        self.outputs.push(output);

        // We call arrange here albeit the output is only appended and
        // this would not affect windows, but arrange could re-organize
        // outputs from a configuration.
        self.arrange();

        self.outputs.last().unwrap()
    }
    
    pub fn retain_outputs<F>(&mut self, f: F)
    where
        F: Fn(&Output) -> bool,
    {
        for output in self.outputs.iter().filter(|o| f(*o)) {
            if let Some(workspace) = output.userdata().get::<ActiveWorkspace>() {
                if self.spaces.get(&workspace.0).unwrap().is_empty() {
                    slog_scope::debug!("Destroying empty workspace: {}", workspace.0);
                    self.spaces.remove(&workspace.0);
                }
            }
        }
        self.outputs.retain(f);

        self.arrange();
    }

    pub fn toplevel_by_surface(&mut self, surface: &WlSurface) -> Option<Kind> {
        for (_, space) in self.spaces.iter_mut() {
            if let Some(window) = space.windows().find(|k| k.get_surface().map(|x| x == surface).unwrap_or(false)) {
                return Some(window);
            }
        }
        None
    }

    pub fn space_by_output_name<'a, N>(&'a mut self, name: N) -> Option<&'a mut Box<dyn Layout>>
    where
        N: AsRef<str>
    {
        let active = self.output_by_name(name).and_then(|x| x.userdata().get::<ActiveWorkspace>()).map(|x| x.0);
        if let Some(a) = active {
            self.spaces.get_mut(&a)
        } else {
            None
        }
    }

    pub fn space_by_seat(&mut self, seat: &Seat) -> Option<&mut Box<dyn Layout>> {
        if let Some(name) = seat.user_data().get::<ActiveOutput>() {
            let exists = self.space_by_output_name(&*name.0.borrow()).is_some();
            if exists {
                return self.space_by_output_name(&*name.0.borrow());
            }
        }
        self.spaces.iter_mut().map(|(_, v)| v).next()
    }

    pub fn space_by_surface(&mut self, surface: &WlSurface) -> Option<&mut Box<dyn Layout>> {
        for (_, space) in self.spaces.iter_mut() {
            if space.windows().any(|k| k.get_surface().map(|x| x == surface).unwrap_or(false)) {
                return Some(space);
            }
        }
        None
    }

    pub fn output<F>(&mut self, f: F) -> Option<&mut Output>
    where
        F: FnMut(&&mut Output) -> bool,
    {
        self.outputs.iter_mut().find(f)
    }

    pub fn output_by_wl(&mut self, output: &wl_output::WlOutput) -> Option<&mut Output> {
        self.output(|o| o.owns(output))
    }

    pub fn output_by_name<N>(&mut self, name: N) -> Option<&mut Output>
    where
        N: AsRef<str>,
    {
        self.output(|o| o.name() == name.as_ref())
    }
}