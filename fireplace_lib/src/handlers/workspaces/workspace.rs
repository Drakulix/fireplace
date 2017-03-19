use callback::{AsWrapper, Wrapper};
use handlers::workspaces::modes::{AnyModeConfig, AnyModeWrap, Mode};
use slog;
use slog_scope;
use wlc::*;

pub struct Workspace {
    pub number: u8,
    pub name: String,
    output: Option<WeakOutput>,
    views: Vec<WeakView>,
    last_focus: Option<WeakView>,
    mode: AnyModeWrap,
    logger: slog::Logger,
}

impl PartialEq for Workspace {
    fn eq(&self, other: &Workspace) -> bool {
        self.number == other.number
    }
}
impl Eq for Workspace {}

impl AsWrapper for Workspace {
    fn child(&mut self) -> Option<&mut Callback> {
        Some(&mut self.mode)
    }
}

impl Callback for Wrapper<Workspace> {
    fn output_created(&mut self, output: &Output) -> bool {
        slog_scope::scope(self.logger.clone(), || {
            debug!(slog_scope::logger(), "Activating");

            self.output = Some(output.weak_reference());
            self.mode.output_created(output);
            self.mode.output_resolution(output, output.resolution(), output.resolution());
            output.set_visibility(Visibility::Flags::from_bits_truncate(1 << self.number));
            for view in &self.views {
                view.run(|view| if view.output() != output {
                             view.set_output(output);
                         });
            }

            true
        })
    }

    fn output_destroyed(&mut self, output: &Output) {
        slog_scope::scope(self.logger.clone(), || {
            debug!(slog_scope::logger(), "Deactivating");
            self.mode.output_destroyed(output);
            self.output = None;
        })
    }

    fn view_created(&mut self, view: &View) -> bool {
        slog_scope::scope(self.logger.clone(), || {
            info!(slog_scope::logger(), "Adding {:?}", view);
            view.set_visibility(Visibility::Flags::from_bits_truncate(1 << self.number));
            self.views.push(view.weak_reference());
            if self.output != Some(view.output().weak_reference()) {
                if let Some(output) = self.output.as_ref() {
                    output.run(|output| { view.set_output(output); });
                }
            }
            self.mode.view_created(view)
        })
    }

    fn view_focus(&mut self, view: &View, focus: bool) {
        if focus {
            self.last_focus = Some(view.weak_reference());
        }
        self.mode.view_focus(view, focus);
    }

    fn view_destroyed(&mut self, view: &View) {
        slog_scope::scope(self.logger.clone(), || {
            info!(slog_scope::logger(), "Removing {:?}", view);
            self.views.retain(|x| x != &view.weak_reference());
            if self.last_focus == Some(view.weak_reference()) {
                self.last_focus = None;
                if let Some(view) = self.views.iter().last() {
                    view.run(|view| view.focus());
                }
            }
            self.mode.view_destroyed(view)
        })
    }
}

impl Workspace {
    pub fn new(num: u8, name: String, arguments: AnyModeConfig) -> Workspace {
        slog_scope::scope(slog_scope::logger()
                              .new(o!("instance" => format!("Workspace {} {}", name.clone(), num))),
                          || {
            let workspace = Workspace {
                number: num,
                name: name,
                output: None,
                views: Vec::new(),
                last_focus: None,
                mode: AnyModeWrap::new(arguments),
                logger: slog_scope::logger(),
            };
            debug!(workspace.logger, "Created");
            workspace
        })
    }

    pub fn active(&self) -> bool {
        self.output
            .as_ref()
            .and_then(|x| x.run(|_| {}))
            .is_some()
    }

    pub fn len(&self) -> usize {
        self.mode.len()
    }

    pub fn is_empty(&self) -> bool {
        self.mode.is_empty()
    }

    pub fn restore_focus(&self) {
        if self.last_focus
               .as_ref()
               .map(|view| view.run(|view| view.focus()))
               .is_none() {
            if let Some(view) = self.views.iter().last() {
                view.run(|view| { view.focus(); });
            } else {
                View::set_no_focus();
            }
        }
    }

    pub fn output(&self) -> Option<WeakOutput> {
        self.output.clone()
    }
}
