use handlers::geometry::UsableScreenGeometry;

use handlers::store::{Store, StoreKey};
use handlers::workspaces::modes::Mode;
use slog;
use slog_scope;

use std::cmp;
use wlc::{Button, ButtonState, Callback, Geometry, Modifiers, Output, Point, ResizeEdge, Size, View,
          ViewType, ViewState, WeakView, input};

/// A `Mode` that does traditional `View` management.
///
/// `View`s are "floating" on the `Output`, may obstruct each other and
/// their requests for placement (like maximizing) are respected.
pub struct Floating {
    views: Vec<WeakView>,
    active_view: Option<WeakView>,
    geo: Geometry,
    logger: slog::Logger,
}

/// Configuration for `Floating`
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Deserialize)]
pub struct FloatingConfig {}

struct InteractiveMove;
struct InteractiveResize;
struct Maximized;

impl StoreKey for InteractiveMove {
    type Value = Point;
}
impl StoreKey for InteractiveResize {
    type Value = ResizeEdge::Flags;
}
impl StoreKey for Maximized {
    type Value = Geometry;
}

impl Callback for Floating {
    fn view_created(&mut self, view: &View) -> bool {
        debug!(self.logger, "Register View: {:?}", view);

        if let Some(pos) = view.positioner() {
            let mut size_req = pos.size();
            if size_req.w == 0 || size_req.h == 0 {
                size_req = view.geometry().size;
            }

            let mut geometry = Geometry {
                origin: pos.anchor_rect().origin,
                size: size_req,
            };

            if let Some(parent) = view.parent() {
                let parent_geometry = parent.geometry();
                geometry.origin.x += parent_geometry.origin.x;
                geometry.origin.y += parent_geometry.origin.y;
            }

            view.set_geometry(ResizeEdge::Null, geometry);
        } else {
            // fix out of bounds
            {
                let view_geometry = view.geometry();
                if !(view_geometry.origin > self.geo.origin) || (view_geometry.size > self.geo.size) {
                    view.set_geometry(ResizeEdge::Null,
                                      Geometry {
                                          origin: Point {
                                              x: cmp::max(self.geo.origin.x, view_geometry.origin.x),
                                              y: cmp::max(self.geo.origin.y, view_geometry.origin.y),
                                          },
                                          size: Size {
                                              w: cmp::min(self.geo.size.w, view_geometry.size.w),
                                              h: cmp::min(self.geo.size.h, view_geometry.size.h),
                                          },
                                      });
                }
            }
            // center
            if !view.view_type().intersects(ViewType::Unmanaged | ViewType::OverrideRedirect)
            {
                let view_geometry = view.geometry();
                view.set_geometry(ResizeEdge::Null, Geometry {
                    origin: Point {
                        x: (self.geo.size.w as i32 / 2) - (view_geometry.size.w as i32 / 2),
                        y: (self.geo.size.h as i32 / 2) - (view_geometry.size.h as i32 / 2),
                    },
                    size: view_geometry.size,
                });
            }
        }

        self.views.push(view.weak_reference());
        true
    }

    fn view_focus(&mut self, view: &View, focus: bool) {
        if focus {
            view.bring_to_front();
        }
    }

    fn view_destroyed(&mut self, view: &View) {
        self.views.retain(|x| x != &view.weak_reference());
    }

    fn view_request_geometry(&mut self, view: &View, geometry: Geometry) {
        view.set_geometry(ResizeEdge::Null,
                          Geometry {
                              origin: Point {
                                  x: cmp::max(self.geo.origin.x, geometry.origin.x),
                                  y: cmp::max(self.geo.origin.y, geometry.origin.y),
                              },
                              size: Size {
                                  w: cmp::min(self.geo.size.w, geometry.size.w),
                                  h: cmp::min(self.geo.size.h, geometry.size.h),
                              },
                          });
    }

    fn view_request_state(&mut self, view: &View, state: ViewState::Flags, toggle: bool) {
        match state {
            x if x.contains(ViewState::Moving) && !toggle => view.set_state(ViewState::Moving, false),
            x if x.contains(ViewState::Moving) && toggle => info!(self.logger, "Moving true?"),
            x if x.contains(ViewState::Resizing) && !toggle => view.set_state(ViewState::Resizing, false),
            x if x.contains(ViewState::Resizing) && toggle => info!(self.logger, "Resizing true?"),
            x if x.contains(ViewState::Maximized) => {
                if toggle {
                    debug!(self.logger, "Maximizing {:?}", view);
                    let old_geo = view.geometry();
                    view.set_geometry(ResizeEdge::Null, self.geo);
                    view.set_state(state, true);
                    view.insert::<Maximized>(old_geo);
                } else {
                    if let Some(old_geo) = view.remove::<Maximized>() {
                        view.set_geometry(ResizeEdge::Null, old_geo);
                    }
                    view.set_state(state, false);
                }
            }
            _ => {}
        };
    }

    fn view_request_move(&mut self, view: &View, _origin: Point) {
        debug!(self.logger, "Start moving View: {:?}", view);
        view.set_state(ViewState::Moving, true);
        let pointer = input::pointer::position();
        let geo = view.geometry();
        let relative = Point {
            x: geo.origin.x - pointer.x,
            y: geo.origin.y - pointer.y,
        };
        view.insert::<InteractiveMove>(relative);
        self.active_view = Some(view.weak_reference());
    }

    fn view_request_resize(&mut self, view: &View, edges: ResizeEdge::Flags, _origin: Point) {
        debug!(self.logger, "Start resizing View: {:?}", view);
        view.set_state(ViewState::Resizing, true);
        view.insert::<InteractiveResize>(edges);
        self.active_view = Some(view.weak_reference());
    }

    fn output_resolution(&mut self, output: &Output, from: Size, to: Size) {
        if to < from {
            for view in &mut self.views {
                view.run(|view| {
                    let geo = view.geometry();
                    let width = if geo.origin.x as u32 + geo.size.w > to.w {
                        to.w as i32 - geo.origin.x
                    } else {
                        geo.size.w as i32
                    } as u32;
                    let height = if geo.origin.y as u32 + geo.size.h > to.h {
                        to.h as i32 - geo.origin.y
                    } else {
                        geo.size.h as i32
                    } as u32;
                    view.set_geometry(ResizeEdge::Null,
                                      Geometry {
                                          origin: Point {
                                              x: geo.origin.x,
                                              y: geo.origin.y,
                                          },
                                          size: Size {
                                              w: width,
                                              h: height,
                                          },
                                      });
                });
            }
        }

        let lock = output.get::<UsableScreenGeometry>();
        self.geo = match lock.as_ref().and_then(|x| x.read().ok()) {
            Some(size) => *size,
            None => {
                Geometry {
                    origin: Point { x: 0, y: 0 },
                    size: to,
                }
            }
        };
    }

    fn pointer_button(&mut self, _view: Option<&View>, _time: u32, _modifiers: Modifiers, button: Button,
                      state: ButtonState, _origin: Point)
                      -> bool {
        if let Some(ref view) = self.active_view {
            view.run(|view| if state == ButtonState::Released && button == Button::Left {
                view.set_state(ViewState::Resizing, false);
                view.set_state(ViewState::Moving, false);
            });
        }
        false
    }

    fn pointer_motion(&mut self, _view: Option<&View>, _time: u32, origin: Point) -> bool {
        let still_active = if let Some(ref view) = self.active_view {
            view.run(|view| {
                    if !view.state().contains(ViewState::Moving) {
                        view.remove::<InteractiveMove>();
                    }
                    if !view.state().contains(ViewState::Resizing) {
                        view.remove::<InteractiveResize>();
                    }
                    view.get::<InteractiveMove>().is_some() || view.get::<InteractiveResize>().is_some()
                })
                .unwrap_or(false)
        } else {
            false
        };

        if !still_active {
            self.active_view = None;
            return false;
        }

        if let Some(ref view) = self.active_view {
            view.run(|view| {
                    let lock = view.get::<InteractiveMove>();
                    if let Some(relative) = lock.as_ref().and_then(|x| x.read().ok()) {
                        let old = view.geometry();
                        view.set_geometry(ResizeEdge::Null,
                                          Geometry {
                                              origin: Point {
                                                  x: cmp::max(origin.x + (*relative).x, self.geo.origin.x),
                                                  y: cmp::max(origin.y + (*relative).y, self.geo.origin.y),
                                              },
                                              size: old.size,
                                          });
                        return true;
                    }

                    let lock = view.get::<InteractiveResize>();
                    if let Some(edges) = lock.as_ref().and_then(|x| x.read().ok()) {
                        if edges.contains(ResizeEdge::Bottom) {
                            let old = view.geometry();
                            view.set_geometry(*edges,
                                              Geometry {
                                                  origin: old.origin,
                                                  size: Size {
                                                      w: old.size.w,
                                                      h: cmp::max(origin.y - old.origin.y, 1) as u32,
                                                  },
                                              });
                        }
                        if edges.contains(ResizeEdge::Top) {
                            let old = view.geometry();
                            view.set_geometry(*edges,
                                              Geometry {
                                                  origin: Point {
                                                      x: old.origin.x,
                                                      y: cmp::max(self.geo.origin.y, origin.y),
                                                  },
                                                  size: Size {
                                                      w: old.size.w,
                                                      h: cmp::max(old.size.h as i32 +
                                                                  (old.origin.y -
                                                                   cmp::max(self.geo.origin.y, origin.y)),
                                                                  1) as
                                                         u32,
                                                  },
                                              });
                        }
                        if edges.contains(ResizeEdge::Left) {
                            let old = view.geometry();
                            view.set_geometry(*edges,
                                              Geometry {
                                                  origin: Point {
                                                      x: cmp::max(self.geo.origin.x, origin.x),
                                                      y: old.origin.y,
                                                  },
                                                  size: Size {
                                                      w: cmp::max(old.size.w as i32 +
                                                                  (old.origin.x -
                                                                   cmp::max(self.geo.origin.x, origin.x)),
                                                                  1) as
                                                         u32,
                                                      h: old.size.h,
                                                  },
                                              });
                        }
                        if edges.contains(ResizeEdge::Right) {
                            let old = view.geometry();
                            view.set_geometry(*edges,
                                              Geometry {
                                                  origin: old.origin,
                                                  size: Size {
                                                      w: cmp::max(origin.x - old.origin.x, 1) as u32,
                                                      h: old.size.h,
                                                  },
                                              });
                        }
                        return true;
                    };

                    false
                })
                .unwrap_or(false)
        } else {
            false
        }
    }
}

impl Mode for Floating {
    type Arguments = FloatingConfig;

    fn new(_: FloatingConfig) -> Floating {
        let mode = Floating {
            views: Vec::new(),
            active_view: None,
            geo: Geometry {
                origin: Point { x: 0, y: 0 },
                size: Size { w: 0, h: 0 },
            },
            logger: slog_scope::logger().new(o!("instance" => "Floating")),
        };
        debug!(mode.logger, "Created");
        mode
    }

    fn len(&self) -> usize {
        self.views.len()
    }
}
