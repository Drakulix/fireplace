use crate::state::Fireplace;
use smithay::{
    backend::input::{InputEvent, InputBackend, Device, DeviceCapability, KeyState},
    wayland::{
        seat::{Seat, XkbConfig},
        SERIAL_COUNTER as SCOUNTER,
    },
    reexports::wayland_server::Display,
};
use std::{
    cell::RefCell,
    collections::HashMap,
};

pub mod keyboard;

pub struct ActiveOutput(pub RefCell<String>);

struct Devices(RefCell<HashMap<String, Vec<DeviceCapability>>>);

impl Devices {
    fn new() -> Devices {
        Devices(RefCell::new(HashMap::new()))
    }

    fn add_device<D: Device>(&self, device: &D) -> Vec<DeviceCapability> {
        let id = device.id();
        let mut map = self.0.borrow_mut();
        let caps = [DeviceCapability::Keyboard, DeviceCapability::Pointer]
            .iter()
            .cloned()
            .filter(|c| device.has_capability(*c))
            .collect::<Vec<_>>();
        let new_caps = caps.iter()
            .cloned()
            .filter(|c| map.values().flatten().all(|has| *c != *has))
            .collect::<Vec<_>>();
        map.insert(id, caps);
        new_caps
    }

    fn has_device<D: Device>(&self, device: &D) -> bool {
        self.0.borrow().contains_key(&device.id())
    }

    fn remove_device<D: Device>(&self, device: &D) -> Vec<DeviceCapability> {
        let id = device.id();
        let mut map = self.0.borrow_mut();
        map.remove(&id)
        .unwrap_or(Vec::new())
        .into_iter()
        .filter(|c| map.values().flatten().all(|has| *c != *has))
        .collect()
    }
}

pub fn add_seat(display: &mut Display, name: String) -> Seat {
    let (seat, _) = Seat::new(display, name, None);
    let userdata = seat.user_data();
    userdata.insert_if_missing(|| Devices::new());
    seat 
}

impl Fireplace {    
    pub fn process_input_event<B: InputBackend>(&mut self, event: InputEvent<B>) {
        use smithay::backend::input::{Event};

        match event {
            InputEvent::DeviceAdded { device } => {
                let seat = &mut self.last_active_seat;
                let userdata = seat.user_data();
                let devices = userdata.get::<Devices>().unwrap();
                for cap in devices.add_device(&device) {
                    match cap {
                        DeviceCapability::Keyboard => {
                            let _ = seat.add_keyboard(XkbConfig::default(), 200, 25, |seat, focus| {

                            });
                        },
                        DeviceCapability::Pointer => {
                            let output = String::from(self.workspaces.borrow_mut().output(|_| true).map(|x| x.name()).unwrap_or("headless"));
                            seat.user_data().insert_if_missing(|| ActiveOutput(RefCell::new(output)));
                            seat.add_pointer(|status| {

                            });
                        },
                        _ => {},
                    }
                }
            },
            InputEvent::DeviceRemoved { device } => {
                for seat in &mut self.seats {
                    let userdata = seat.user_data();
                    let devices = userdata.get::<Devices>().unwrap();
                    if devices.has_device(&device) {
                        for cap in devices.remove_device(&device) {
                            match cap {
                                DeviceCapability::Keyboard => {
                                    seat.remove_keyboard();
                                },
                                DeviceCapability::Pointer => {
                                    seat.remove_pointer();
                                },
                                _ => {},
                            }
                        }
                        break;
                    }
                }
            },
            InputEvent::Keyboard { event, .. } => {
                use smithay::backend::input::KeyboardKeyEvent;

                let device = event.device();
                for seat in self.seats.clone().iter() {
                    let userdata = seat.user_data();
                    let devices = userdata.get::<Devices>().unwrap();
                    if devices.has_device(&device) {
                        let keycode = event.key_code();
                        let state = event.state();
                        slog_scope::debug!("key"; "keycode" => keycode, "state" => format!("{:?}", state));
                        let serial = SCOUNTER.next_serial();
                        let time = Event::time(&event);
                        seat
                        .get_keyboard()
                        .unwrap()
                        .input(keycode, state, serial, time, |modifiers, keysym| {
                            slog_scope::trace!("keysym";
                                "state" => format!("{:?}", state),
                                "mods" => format!("{:?}", modifiers),
                                "keysym" => ::xkbcommon::xkb::keysym_get_name(keysym)
                            );

                            // If the key is pressed and triggered a action
                            // we will not forward the key to the client.
                            // Additionally add the key to the suppressed keys
                            // so that we can decide on a release if the key
                            // should be forwarded to the client or not.
                            if let KeyState::Pressed = state {
                                if let Some(command) = self.config.keys.iter()
                                    .find(|(_, p)| p.modifiers == *modifiers && p.key == keysym)
                                    .map(|(c, _)| c)
                                    .cloned()
                                {
                                    self.process_global_command(&command);
                                    self.suppressed_keys.push(keysym);
                                    return false;
                                }
                                if let Some(command) = self.config.view.keys.iter()
                                    .find(|(_, p)| p.modifiers == *modifiers && p.key == keysym)
                                    .map(|(c, _)| c)
                                    .cloned()
                                {
                                    self.process_view_command(&command, seat);
                                    self.suppressed_keys.push(keysym);
                                    return false;
                                }
                                if let Some(command) = self.config.exec.keys.iter()
                                    .find(|(_, p)| p.modifiers == *modifiers && p.key == keysym)
                                    .map(|(c, _)| c)
                                    .cloned()
                                {
                                    if let Err(err) = self.process_exec_command(&command) {
                                        slog_scope::warn!("Failed to spawn process: {}", err);
                                    }
                                    self.suppressed_keys.push(keysym);
                                    return false;
                                }
                                true
                            } else {
                                let suppressed = self.suppressed_keys.contains(&keysym);
                                if suppressed {
                                    self.suppressed_keys.retain(|k| *k != keysym);
                                }
                                !suppressed
                            }
                        });
                        
                        
                        break;
                    }
                }

            },
            InputEvent::PointerMotion { event, .. } => {
                use smithay::backend::input::PointerMotionEvent;

                let device = event.device();
                for seat in self.seats.clone().iter() {
                    let userdata = seat.user_data();
                    let devices = userdata.get::<Devices>().unwrap();
                    if devices.has_device(&device) {
                        let mut current_output_name = userdata.get::<ActiveOutput>().unwrap().0.borrow_mut();
                        let mut workspaces = self.workspaces.borrow_mut();

                        let serial = SCOUNTER.next_serial();

                        // clamp coordinates
                        let mut location = seat.get_pointer().unwrap().current_location();
                        let output_name = {
                            location += event.delta();
                            location.x = f64::min(f64::max(0.0, location.x), workspaces.width() as f64);
                            let output = workspaces.output(|o| {
                                let geo = o.geometry();
                                geo.loc.x as f64 <= location.x && (geo.loc.x + geo.size.w) as f64 > location.x
                            }).unwrap();
                            location.y = f64::min(f64::max(0.0, location.y), output.size().h as f64);
                            String::from(output.name())
                        };

                        let space = workspaces.space_by_output_name(&output_name).unwrap();
                        let under = space.surface_under(location);
                        seat.get_pointer().unwrap()
                            .motion(location, under, serial, event.time());

                        *current_output_name = output_name;
                        break;
                    }
                }
                 
            },
            InputEvent::PointerMotionAbsolute { event, .. } => {
                use smithay::backend::input::PointerMotionAbsoluteEvent;

                let device = event.device();
                for seat in self.seats.clone().iter() {
                    let userdata = seat.user_data();
                    let devices = userdata.get::<Devices>().unwrap();
                    if devices.has_device(&device) {
                        let output_name = userdata.get::<ActiveOutput>().unwrap().0.borrow();
                        let mut workspaces = self.workspaces.borrow_mut();
                        let output = workspaces.output_by_name(&*output_name).unwrap();
                        let output_size = output.size();
                        let pos = output.location().to_f64() + event.position_transformed(output_size);
                        let serial = SCOUNTER.next_serial();
                        let space = workspaces.space_by_output_name(&*output_name).unwrap();
                        let under = space.surface_under(pos);
                        seat.get_pointer().unwrap().motion(pos, under, serial, event.time());
                        break;
                    }
                }
            },
            InputEvent::PointerButton { event, .. } => {
                use smithay::{
                    backend::input::{PointerButtonEvent, MouseButton, ButtonState},
                    reexports::wayland_server::protocol::wl_pointer,
                };

                let device = event.device();
                for seat in self.seats.clone().iter() {
                    let userdata = seat.user_data();
                    let devices = userdata.get::<Devices>().unwrap();
                    if devices.has_device(&device) {
                        let serial = SCOUNTER.next_serial();
                        let button = match event.button() {
                            MouseButton::Left => 0x110,
                            MouseButton::Right => 0x111,
                            MouseButton::Middle => 0x112,
                            MouseButton::Other(b) => b as u32,
                        };
                        let state = match event.state() {
                            ButtonState::Pressed => {
                                // change the keyboard focus unless the pointer is grabbed
                                if !seat.get_pointer().unwrap().is_grabbed() {
                                    let mut workspaces = self.workspaces.borrow_mut();
                                    let space = workspaces.space_by_seat(&seat).unwrap();
                                    let pos = seat.get_pointer().unwrap().current_location();
                                    let under = space.surface_under(pos);
                                    if let Some(&(ref under, _)) = under.as_ref() {
                                        space.on_focus(under);
                                    }
                                    if let Some(keyboard) = seat.get_keyboard() {
                                        keyboard.set_focus(under.as_ref().map(|&(ref s, _)| s), serial);
                                    }
                                }
                                wl_pointer::ButtonState::Pressed
                            }
                            ButtonState::Released => wl_pointer::ButtonState::Released,
                        };
                        seat.get_pointer().unwrap().button(button, state, serial, event.time());
                        break;
                    }
                }
            },
            InputEvent::PointerAxis { event, .. } => {
                use smithay::{
                    backend::input::{PointerAxisEvent, AxisSource, Axis},
                    reexports::wayland_server::protocol::wl_pointer,
                    wayland::seat::AxisFrame,
                };

                let device = event.device();
                for seat in self.seats.clone().iter() {
                    let userdata = seat.user_data();
                    let devices = userdata.get::<Devices>().unwrap();
                    if devices.has_device(&device) {
                        let source = match event.source() {
                            AxisSource::Continuous => wl_pointer::AxisSource::Continuous,
                            AxisSource::Finger => wl_pointer::AxisSource::Finger,
                            AxisSource::Wheel | AxisSource::WheelTilt => wl_pointer::AxisSource::Wheel,
                        };
                        let horizontal_amount = event
                            .amount(Axis::Horizontal)
                            .unwrap_or_else(|| event.amount_discrete(Axis::Horizontal).unwrap() * 3.0);
                        let vertical_amount = event
                            .amount(Axis::Vertical)
                            .unwrap_or_else(|| event.amount_discrete(Axis::Vertical).unwrap() * 3.0);
                        let horizontal_amount_discrete = event.amount_discrete(Axis::Horizontal);
                        let vertical_amount_discrete = event.amount_discrete(Axis::Vertical);

                        {
                            let mut frame = AxisFrame::new(event.time()).source(source);
                            if horizontal_amount != 0.0 {
                                frame = frame.value(wl_pointer::Axis::HorizontalScroll, horizontal_amount);
                                if let Some(discrete) = horizontal_amount_discrete {
                                    frame = frame.discrete(wl_pointer::Axis::HorizontalScroll, discrete as i32);
                                }
                            } else if source == wl_pointer::AxisSource::Finger {
                                frame = frame.stop(wl_pointer::Axis::HorizontalScroll);
                            }
                            if vertical_amount != 0.0 {
                                frame = frame.value(wl_pointer::Axis::VerticalScroll, vertical_amount);
                                if let Some(discrete) = vertical_amount_discrete {
                                    frame = frame.discrete(wl_pointer::Axis::VerticalScroll, discrete as i32);
                                }
                            } else if source == wl_pointer::AxisSource::Finger {
                                frame = frame.stop(wl_pointer::Axis::VerticalScroll);
                            }
                            seat.get_pointer().unwrap().axis(frame);
                        }
                        break;
                    }
                }
            },
            _ => {},
        }
    }

    pub fn process_global_command(&mut self, command: &str) {
        match command {
            "terminate" => {
                self.should_stop = true;
            },
            _ => { slog_scope::debug!("Unknown global command: {}", command); }
        }
    }

    pub fn process_view_command(&mut self, command: &str, seat: &Seat) {
        match command {
            "close" => {
                let mut workspaces = self.workspaces.borrow_mut();
                let space = workspaces.space_by_seat(&seat).unwrap();
                if let Some(window) = space.focused_window() {
                    window.send_close();
                } 
            },
            _ => { slog_scope::debug!("Unknown view command: {}", command); }
        }
    }

    pub fn process_exec_command(&mut self, command: &str) -> std::io::Result<()> {
        std::process::Command::new("/bin/sh")
            .arg("-c")
            .arg(command)
            .spawn()
            .map(|_| ())
    }

    pub fn last_active_seat(&self) -> &Seat {
        &self.last_active_seat
    }
}