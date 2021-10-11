use crate::handler::ActiveOutput;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use linked_hash_map::LinkedHashMap;
use smithay::{
    reexports::wayland_server::{
        protocol::{wl_output, wl_surface::WlSurface},
        Display,
    },
    utils::{Logical, Size},
    wayland::{
        output::{Mode, PhysicalProperties},
        seat::Seat,
    },
};

use crate::shell::{layout::Layout, output::Output, window::Kind};

pub struct Workspaces {
    display: Rc<RefCell<Display>>,
    spaces: LinkedHashMap<u8, Box<dyn Layout>>,
    outputs: Vec<Output>,
}

struct ActiveWorkspace(Cell<u8>);

impl ActiveWorkspace {
    fn new(val: u8) -> ActiveWorkspace {
        ActiveWorkspace(Cell::new(val))
    }
}

impl Workspaces {
    pub fn new(display: Rc<RefCell<Display>>) -> Workspaces {
        Workspaces {
            display,
            spaces: LinkedHashMap::new(),
            outputs: Vec::new(),
        }
    }

    fn next_available(&mut self, size: Size<i32, Logical>) -> u8 {
        for i in 1..::std::u8::MAX {
            if let Some(space) = self.spaces.get_mut(&i) {
                let mut available = true;
                for output in &self.outputs {
                    if output
                        .userdata()
                        .get::<ActiveWorkspace>()
                        .map(|x| x.0.get() as i32)
                        .unwrap()
                        == i as i32
                    {
                        available = false;
                    }
                }
                if available {
                    space.rearrange(&size);
                    return i;
                }
            } else {
                self.spaces
                    .insert(i, Box::new(super::layout::Floating::new(size)));
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
        slog_scope::debug!(
            "Attaching workspace {} to output {}",
            workspace,
            output.name()
        );
        output
            .userdata()
            .insert_if_missing(|| ActiveWorkspace::new(workspace));
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
        for output in self.outputs.iter().filter(|o| !f(*o)) {
            let workspace = output.userdata().get::<ActiveWorkspace>().unwrap().0.get();
            if self.spaces.get(&workspace).unwrap().is_empty() {
                slog_scope::debug!("Destroying empty workspace: {}", workspace);
                self.spaces.remove(&workspace);
            }
        }
        self.outputs.retain(f);
        self.arrange();
    }

    pub fn remove_output_by_name(&mut self, name: &str) {
        self.retain_outputs(|o| o.name() != name);
    }

    pub fn num_outputs(&self) -> usize {
        self.outputs.len()
    }

    pub fn toplevel_by_surface(&mut self, surface: &WlSurface) -> Option<Kind> {
        for (_, space) in self.spaces.iter_mut() {
            if let Some(window) = space
                .windows()
                .find(|k| k.get_surface().map(|x| x == surface).unwrap_or(false))
            {
                return Some(window);
            }
        }
        None
    }

    pub fn idx_by_output_name<N: AsRef<str>>(&self, name: N) -> Option<u8> {
        self.outputs
            .iter()
            .find(|o| o.name() == name.as_ref())
            .and_then(|x| x.userdata().get::<ActiveWorkspace>())
            .map(|x| x.0.get())
    }

    pub fn spaces<'a>(&'a mut self) -> impl Iterator<Item=&'a mut Box<dyn Layout>>
    {
        self.spaces.iter_mut().map(|(_, layout)| layout)
    }

    pub fn space_by_output_name<'a, N>(&'a mut self, name: N) -> Option<&'a mut Box<dyn Layout>>
    where
        N: AsRef<str>,
    {
        let active = self.idx_by_output_name(name);
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
            if space
                .windows()
                .any(|k| k.get_surface().map(|x| x == surface).unwrap_or(false))
            {
                return Some(space);
            }
        }
        None
    }

    pub fn space_by_idx(&mut self, idx: u8) -> &mut Box<dyn Layout> {
        self.spaces
            .entry(idx)
            .or_insert(Box::new(super::layout::Floating::new((0, 0))))
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

    pub fn switch_workspace(&mut self, seat: &Seat, idx: u8) {
        let output_name = &seat.user_data().get::<ActiveOutput>().unwrap().0;
        let current_idx = self.idx_by_output_name(&*output_name.borrow()).unwrap();
        if current_idx != idx {
            if let Some(output) =
                self.output(|o| o.userdata().get::<ActiveWorkspace>().unwrap().0.get() == idx)
            {
                *output_name.borrow_mut() = String::from(output.name());
                if let Some(ptr) = seat.get_pointer() {
                    let (w, h) = output.size().into();
                    ptr.unset_grab();
                    ptr.motion((w as f64 / 2.0, h as f64 / 2.0).into(), None, 0.into(), 0);
                }
            } else {
                let output = self.output_by_name(&*output_name.borrow()).unwrap();
                slog_scope::debug!("Attaching workspace {} to output {}", idx, output.name());
                output
                    .userdata()
                    .get::<ActiveWorkspace>()
                    .unwrap()
                    .0
                    .set(idx);
                let size = output.size();
                let _ = self
                    .spaces
                    .entry(idx)
                    .or_insert(Box::new(super::layout::Floating::new(size)));
            }
        }
        if self.space_by_idx(current_idx).is_empty() && self.output(|o| o.userdata().get::<ActiveWorkspace>().unwrap().0.get() == current_idx).is_none() { 
            slog_scope::debug!("Destroying empty workspace: {}", current_idx);
            self.spaces.remove(&current_idx);
        }
    }
}
