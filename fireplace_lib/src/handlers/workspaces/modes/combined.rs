use super::{AnyModeConfig, AnyModeWrap, Mode};
use callback::{AsSplit, IntoCallback, Split};
use handlers::geometry::InitialViewGeometry;
use handlers::keyboard::KeyPattern;
use handlers::store::Store;
use slog;
use slog_scope;
use wlc::{Button, ButtonState, Callback, Geometry, Key, KeyState, Modifiers, Point, ResizeEdge, ScrollAxis,
          TouchType, View, ViewPropertyUpdate, ViewState, ViewType, WeakView};
#[cfg(feature = "render")]
use wlc::render::RenderView;

/// A `Mode` that splits `View` by given filter rules onto two given `Mode`s
///
/// Both modes are placed above each other and `Predicates` may be used
/// to filter which `Mode` gets it.
///
/// A `View` may also be manually reassigned independenty from
/// the `Predicates`.
pub struct Combined {
    top: Box<AnyModeWrap>,
    bottom: Box<AnyModeWrap>,
    predicate: Predicate,
    keys: KeyPatterns,
    top_views: Vec<WeakView>,
    bottom_views: Vec<WeakView>,
    logger: slog::Logger,
}

/// Configuration for `Combined`
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
pub struct CombinedConfig {
    /// Configuration for the upper `Mode`
    pub top: Box<AnyModeConfig>,
    /// Configuration for the lower `Mode`
    pub bottom: Box<AnyModeConfig>,
    /// `Predicate` that decides if a new `View` shall be assigned to the
    /// upper `Mode` (if it matches) or not.
    pub predicate: Predicate,
    /// Key configuration
    pub keys: KeyPatterns,
}

/// A `Predicate` may filter given `Views`
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
pub enum Predicate {
    /// Negates a given `Predicate`
    #[serde(rename = "not")]
    Not(Box<Predicate>),
    /// Matches if any given `Predicate` matches
    #[serde(rename = "any")]
    Any(Vec<Predicate>),
    /// Matches if all given `Predicate`s are matching
    #[serde(rename = "all")]
    All(Vec<Predicate>),
    /// Matches if the given `ViewType` is a subset of the `View`'s `ViewType`
    #[serde(rename = "type")]
    Type(ViewType::Flags),
    /// Matches if the given String matches the `View`'s title
    #[serde(rename = "title")]
    Title(String),
    /// Matches if the given String matches the `View`'s class
    #[serde(rename = "class")]
    Class(String),
    /// Matches if the given String matches the `View`'s instance
    #[serde(rename = "instance")]
    Instance(String),
    /// Matches if the given String matches the `View`'s app id
    #[serde(rename = "appid")]
    AppId(String),
}

impl Predicate {
    /// Returns true if a given `View` matches this `Predicate`
    pub fn matches(&self, view: &View) -> bool {
        match *self {
            Predicate::Not(ref predicate) => !predicate.matches(view),
            Predicate::Any(ref predicates) => predicates.iter().any(|x| x.matches(view)),
            Predicate::All(ref predicates) => predicates.iter().all(|x| x.matches(view)),
            Predicate::Type(ref flags) => view.view_type().contains(*flags),
            Predicate::Title(ref title) => &*view.title() == title,
            Predicate::Class(ref class) => &*view.class() == class,
            Predicate::Instance(ref instance) => &*view.instance() == instance,
            Predicate::AppId(ref appid) => &*view.app_id() == appid,
        }
    }
}

/// `KeyPattern`s toggling modes
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Deserialize)]
pub struct KeyPatterns {
    /// Toggle the current `Mode` of a `View`
    pub toggle: Option<KeyPattern>,
}

impl AsSplit for Combined {
    type Callback1 = AnyModeWrap;
    type Callback2 = AnyModeWrap;

    fn first_child(&mut self) -> Option<&mut Self::Callback1> {
        Some(&mut self.top)
    }

    fn second_child(&mut self) -> Option<&mut Self::Callback2> {
        Some(&mut self.bottom)
    }
}

/// Convinient type alias for a wrapped `Combined`
pub type CombinedWrap = Split<Combined>;

impl Mode for Split<Combined> {
    type Arguments = CombinedConfig;

    fn new(arguments: Self::Arguments) -> Self {
        slog_scope::scope(slog_scope::logger().new(o!("instance" => "Combined")),
                          move || {
            Combined {
                    top: Box::new(AnyModeWrap::new(*arguments.top)),
                    bottom: Box::new(AnyModeWrap::new(*arguments.bottom)),
                    predicate: arguments.predicate,
                    keys: arguments.keys,
                    top_views: Vec::new(),
                    bottom_views: Vec::new(),
                    logger: slog_scope::logger().clone(),
                }
                .into_callback()
        })
    }

    fn len(&self) -> usize {
        self.top_views.len() + self.bottom_views.len()
    }
}

impl Callback for Split<Combined> {
    fn view_created(&mut self, view: &View) -> bool {
        slog_scope::scope(self.logger.clone(),
                          || if (&**self).predicate.matches(view) {
                              self.top_views.push(view.weak_reference());
                              self.top.view_created(view)
                          } else {
                              self.bottom_views.push(view.weak_reference());
                              if self.bottom.view_created(view) {
                                  for top_view in &self.top_views {
                                      top_view.run(|top_view| {
                        view.send_below(top_view);
                        info!(self.logger, "sending {:?} below {:?}", view, top_view);
                    });
                                  }
                                  true
                              } else {
                                  false
                              }
                          })
    }

    fn view_destroyed(&mut self, view: &View) {
        slog_scope::scope(self.logger.clone(), || {
            let top_contains = self.top_views.contains(&view.weak_reference());
            let bottom_contains = self.bottom_views.contains(&view.weak_reference());
            if top_contains {
                self.top.view_destroyed(view);
                self.top_views.retain(|x| x != &view.weak_reference());
            } else if bottom_contains {
                self.bottom.view_destroyed(view);
                self.bottom_views.retain(|x| x != &view.weak_reference());
            }
        })
    }

    fn view_focus(&mut self, view: &View, focus: bool) {
        slog_scope::scope(self.logger.clone(),
                          || if self.top_views.contains(&view.weak_reference()) {
                              self.top.view_focus(view, focus);
                          } else if self.bottom_views.contains(&view.weak_reference()) {
            self.bottom.view_focus(view, focus);
        })
    }

    fn view_request_geometry(&mut self, view: &View, geometry: Geometry) {
        slog_scope::scope(self.logger.clone(),
                          || if self.top_views.contains(&view.weak_reference()) {
                              self.top.view_request_geometry(view, geometry);
                          } else if self.bottom_views.contains(&view.weak_reference()) {
            self.bottom.view_request_geometry(view, geometry);
        })
    }

    fn view_request_state(&mut self, view: &View, state: ViewState::Flags, toggle: bool) {
        slog_scope::scope(self.logger.clone(),
                          || if self.top_views.contains(&view.weak_reference()) {
                              self.top.view_request_state(view, state, toggle);
                          } else if self.bottom_views.contains(&view.weak_reference()) {
            self.bottom.view_request_state(view, state, toggle);
        })
    }

    fn view_request_move(&mut self, view: &View, origin: Point) {
        slog_scope::scope(self.logger.clone(),
                          || if self.top_views.contains(&view.weak_reference()) {
                              self.top.view_request_move(view, origin);
                          } else if self.bottom_views.contains(&view.weak_reference()) {
            self.bottom.view_request_move(view, origin);
        })
    }

    fn view_request_resize(&mut self, view: &View, edges: ResizeEdge::Flags, origin: Point) {
        slog_scope::scope(self.logger.clone(),
                          || if self.top_views.contains(&view.weak_reference()) {
                              self.top.view_request_resize(view, edges, origin);
                          } else if self.bottom_views.contains(&view.weak_reference()) {
            self.bottom.view_request_resize(view, edges, origin);
        })
    }

    #[cfg(feature = "render")]
    fn view_render_pre(&mut self, view: &mut RenderView) {
        slog_scope::scope(self.logger.clone(),
                          || if self.top_views.contains(&view.weak_reference()) {
                              self.top.view_render_pre(view);
                          } else if self.bottom_views.contains(&view.weak_reference()) {
            self.bottom.view_render_pre(view);
        })
    }

    #[cfg(not(feature = "render"))]
    fn view_render_pre(&mut self, view: &View) {
        slog_scope::scope(self.logger.clone(),
                          || if self.top_views.contains(&view.weak_reference()) {
                              self.top.view_render_pre(view);
                          } else if self.bottom_views.contains(&view.weak_reference()) {
            self.bottom.view_render_pre(view);
        })
    }

    #[cfg(feature = "render")]
    fn view_render_post(&mut self, view: &mut RenderView) {
        slog_scope::scope(self.logger.clone(),
                          || if self.top_views.contains(&view.weak_reference()) {
                              self.top.view_render_post(view);
                          } else if self.bottom_views.contains(&view.weak_reference()) {
            self.bottom.view_render_post(view);
        })
    }

    #[cfg(not(feature = "render"))]
    fn view_render_post(&mut self, view: &View) {
        slog_scope::scope(self.logger.clone(),
                          || if self.top_views.contains(&view.weak_reference()) {
                              self.top.view_render_post(view);
                          } else if self.bottom_views.contains(&view.weak_reference()) {
            self.bottom.view_render_post(view);
        })
    }

    fn view_properties_updated(&mut self, view: &View, mask: ViewPropertyUpdate::Flags) {
        slog_scope::scope(self.logger.clone(),
                          || if self.top_views.contains(&view.weak_reference()) {
                              self.top.view_properties_updated(view, mask);
                          } else if self.bottom_views.contains(&view.weak_reference()) {
            self.bottom.view_properties_updated(view, mask);
        })
    }

    fn keyboard_key(&mut self, view: Option<&View>, time: u32, modifiers: Modifiers, key: Key,
                    state: KeyState)
                    -> bool {
        slog_scope::scope(self.logger.clone(), || {
            if let Some(view) = view {
                match Some(KeyPattern::new(state, modifiers.mods, key)) {
                    x if x == self.keys.toggle => {
                        // reset initial geometry
                        let initial = view.get::<InitialViewGeometry>().unwrap();
                        view.set_geometry(ResizeEdge::Null, *initial.read().unwrap());

                        let top_contains = self.top_views.contains(&view.weak_reference());
                        let bottom_contains = self.bottom_views.contains(&view.weak_reference());

                        if top_contains {
                            debug!(slog_scope::logger(), "Toggling mode for {:?}", view);
                            self.top.view_destroyed(view);
                            if self.bottom.view_created(view) {
                                let src_position = self.top_views
                                    .iter()
                                    .position(|x| x == &view.weak_reference())
                                    .unwrap();
                                let src_view = self.top_views.remove(src_position);

                                for top_view in &mut self.top_views {
                                    src_view.run(|src_view| {
                                                     top_view.run(|top_view| src_view.send_below(top_view))
                                                 });
                                }

                                self.bottom_views.push(src_view);
                            } else {
                                warn!(slog_scope::logger(),
                                      "Toggling mode failed, switching back to old view");
                                if !self.top.view_created(view) {
                                    error!(slog_scope::logger(),
                                           "Old Mode failed as well, closing view");
                                    view.close();
                                }
                            }
                            return true;
                        } else if bottom_contains {
                            debug!(slog_scope::logger(), "Toggling mode for {:?}", view);
                            self.bottom.view_destroyed(view);
                            if self.top.view_created(view) {
                                let src_position = self.bottom_views
                                    .iter()
                                    .position(|x| x == &view.weak_reference())
                                    .unwrap();

                                let src_view = self.bottom_views.remove(src_position);
                                src_view.run(|src_view| src_view.bring_to_front());
                                self.top_views.push(src_view);
                            } else {
                                warn!(slog_scope::logger(),
                                      "Toggling mode failed, switching back to old view");
                                if !self.top.view_created(view) {
                                    error!(slog_scope::logger(),
                                           "Old Mode failed as well, closing view");
                                    view.close();
                                }
                            }
                            return true;
                        }
                    }
                    _ => {
                        return if self.top_views.contains(&view.weak_reference()) {
                                   self.top.keyboard_key(Some(view), time, modifiers, key, state)
                               } else if self.bottom_views.contains(&view.weak_reference()) {
                            self.bottom.keyboard_key(Some(view), time, modifiers, key, state)
                        } else {
                            false
                        };
                    }
                }
            }

            let mut result = false;
            if let Some(child) = (**self).first_child() {
                result = child.keyboard_key(view, time, modifiers, key, state) || result;
            }
            if let Some(child) = (**self).second_child() {
                result = child.keyboard_key(view, time, modifiers, key, state) || result;
            }
            result
        })
    }

    fn pointer_button(&mut self, view: Option<&View>, time: u32, modifiers: Modifiers, button: Button,
                      state: ButtonState, origin: Point)
                      -> bool {
        slog_scope::scope(self.logger.clone(), || if let Some(view) = view {
            if self.top_views.contains(&view.weak_reference()) {
                self.top.pointer_button(Some(view), time, modifiers, button, state, origin)
            } else if self.bottom_views.contains(&view.weak_reference()) {
                self.bottom.pointer_button(Some(view), time, modifiers, button, state, origin)
            } else {
                false
            }
        } else {
            let mut result = false;
            if let Some(child) = (**self).first_child() {
                result = child.pointer_button(view, time, modifiers, button, state, origin) || result;
            }
            if let Some(child) = (**self).second_child() {
                result = child.pointer_button(view, time, modifiers, button, state, origin) || result;
            }
            result
        })
    }

    fn pointer_scroll(&mut self, view: Option<&View>, time: u32, modifiers: Modifiers,
                      axis: ScrollAxis::Flags, amount: [f64; 2])
                      -> bool {
        slog_scope::scope(self.logger.clone(), || if let Some(view) = view {
            if self.top_views.contains(&view.weak_reference()) {
                self.top.pointer_scroll(Some(view), time, modifiers, axis, amount)
            } else if self.bottom_views.contains(&view.weak_reference()) {
                self.bottom.pointer_scroll(Some(view), time, modifiers, axis, amount)
            } else {
                false
            }
        } else {
            let mut result = false;
            if let Some(child) = (**self).first_child() {
                result = child.pointer_scroll(view, time, modifiers, axis, amount) || result;
            }
            if let Some(child) = (**self).second_child() {
                result = child.pointer_scroll(view, time, modifiers, axis, amount) || result;
            }
            result
        })
    }

    fn pointer_motion(&mut self, view: Option<&View>, time: u32, origin: Point) -> bool {
        slog_scope::scope(self.logger.clone(), || if let Some(view) = view {
            if self.top_views.contains(&view.weak_reference()) {
                self.top.pointer_motion(Some(view), time, origin)
            } else if self.bottom_views.contains(&view.weak_reference()) {
                self.bottom.pointer_motion(Some(view), time, origin)
            } else {
                false
            }
        } else {
            let mut result = false;
            if let Some(child) = (**self).first_child() {
                result = child.pointer_motion(view, time, origin) || result;
            }
            if let Some(child) = (**self).second_child() {
                result = child.pointer_motion(view, time, origin) || result;
            }
            result
        })
    }

    fn touch(&mut self, view: Option<&View>, time: u32, modifiers: Modifiers, touch_type: TouchType,
             slot: i32, origin: Point)
             -> bool {
        slog_scope::scope(self.logger.clone(), || if let Some(view) = view {
            if self.top_views.contains(&view.weak_reference()) {
                self.top.touch(Some(view), time, modifiers, touch_type, slot, origin)
            } else if self.bottom_views.contains(&view.weak_reference()) {
                self.bottom.touch(Some(view), time, modifiers, touch_type, slot, origin)
            } else {
                false
            }
        } else {
            let mut result = false;
            if let Some(child) = (**self).first_child() {
                result = child.touch(view, time, modifiers, touch_type, slot, origin) || result;
            }
            if let Some(child) = (**self).second_child() {
                result = child.touch(view, time, modifiers, touch_type, slot, origin) || result;
            }
            result
        })
    }
}
