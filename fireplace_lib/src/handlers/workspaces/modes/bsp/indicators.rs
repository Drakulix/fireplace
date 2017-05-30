use conrod::{Borderable, Colorable, Positionable, Sizeable, Widget};
use conrod::UiCell;
use conrod::color;
use conrod::widget::Canvas;
use conrod::widget::id::Id;
use handlers::geometry::UsableViewGeometry;
use handlers::render::conrod::ConrodRenderer;
use handlers::render::conrod::provider::ConrodProvider;

use handlers::store::Store;

use std::sync::{Arc, RwLock};
use utils::coordinates::*;

use wlc::{Callback, Output, View, ViewState, WeakView};

/// A handler used to display border for indication of focus around `View`s
///
/// ## Dependencies
///
/// - [`FocusHandler`](../../../struct.FocusHandler.html)
/// - [`StoreHandler`](../../../struct.StoreHandler.html)
/// - [`ConrodHandler`](../../../render/conrod/struct.ConrodHandler.html)
///
#[derive(Default)]
pub struct IndicatorsHandler {
    width: u32,
    views: Arc<RwLock<Vec<WeakView>>>,
}

impl IndicatorsHandler {
    /// Initialize a new `IndicatorsHandler`
    pub fn new(config: IndicatorConfig) -> IndicatorsHandler {
        IndicatorsHandler {
            width: config.width,
            views: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

/// Configuration for `IndicatorsHandler`
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
pub struct IndicatorConfig {
    /// Thicknes of the indicator borders in screen points
    #[serde(default = "::handlers::workspaces::modes::bsp::indicators::default_width")]
    pub width: u32,
}

impl Default for IndicatorConfig {
    fn default() -> IndicatorConfig {
        IndicatorConfig { width: default_width() }
    }
}

pub fn default_width() -> u32 {
    4
}

impl Callback for IndicatorsHandler {
    fn output_context_created(&mut self, output: &Output) {
        if let Some(lock) = output.get::<ConrodRenderer>() {
            let mut ui = lock.write().unwrap();
            ui.background
                .register(Indicator::new(self.width, self.views.clone()));
        }
    }

    fn view_created(&mut self, view: &View) -> bool {
        if let Some(lock) = view.get::<UsableViewGeometry>() {
            let mut scissor = lock.write().unwrap();
            scissor.up += (self.width * view.output().scale()) as usize;
            scissor.down += (self.width * view.output().scale()) as usize;
            scissor.left += (self.width * view.output().scale()) as usize;
            scissor.right += (self.width * view.output().scale()) as usize;
        }

        self.views.write().unwrap().push(view.weak_reference());

        true
    }

    fn view_destroyed(&mut self, view: &View) {
        self.views.write().unwrap().retain(|x| x != view);

        if let Some(lock) = view.get::<UsableViewGeometry>() {
            let mut scissor = lock.write().unwrap();
            scissor.up -= (self.width * view.output().scale()) as usize;
            scissor.down -= (self.width * view.output().scale()) as usize;
            scissor.left -= (self.width * view.output().scale()) as usize;
            scissor.right -= (self.width * view.output().scale()) as usize;
        }
    }
}

/// A provider rendering indicator borders for `View`s of one `Output`
pub struct Indicator {
    ids: Vec<[Id; 4]>,
    width: u32,
    views: Arc<RwLock<Vec<WeakView>>>,
}

impl Indicator {
    fn new(width: u32, views: Arc<RwLock<Vec<WeakView>>>) -> Indicator {
        Indicator {
            ids: Vec::new(),
            width: width,
            views: views,
        }
    }
}

impl ConrodProvider for Indicator {
    fn render(&mut self, output: &Output, ui: &mut UiCell) {
        let views = self.views.read().unwrap();
        while views.len() > self.ids.len() {
            self.ids
                .push([ui.widget_id_generator().next(),
                       ui.widget_id_generator().next(),
                       ui.widget_id_generator().next(),
                       ui.widget_id_generator().next()]);
        }

        let dim = ui.window_dim();

        for (ids, view) in self.ids.iter().zip(views.iter()) {
            view.run(|view| {
                let indicator_color = if view.state().contains(ViewState::Activated) {
                    color::LIGHT_BLUE
                } else {
                    color::DARK_CHARCOAL
                };

                if view.output() != output {
                    return;
                }
                if view.visibility() != output.visibility() {
                    return;
                }

                let geo = view.geometry();

                // left
                Canvas::new()
                    .x_y(x_wlc_to_conrod((geo.origin.x - self.width as i32) * output.scale() as i32,
                                         self.width * output.scale(),
                                         dim[0] as u32),
                         y_wlc_to_conrod(geo.origin.y * output.scale() as i32,
                                         geo.size.h * output.scale(),
                                         dim[1] as u32))
                    .h((geo.size.h * output.scale()) as f64)
                    .w((self.width * output.scale()) as f64)
                    .color(indicator_color)
                    .border(0.0)
                    .set(ids[0], ui);

                // right
                Canvas::new()
                    .x_y(x_wlc_to_conrod((geo.origin.x + geo.size.w as i32) * output.scale() as i32,
                                         self.width * output.scale(),
                                         dim[0] as u32),
                         y_wlc_to_conrod(geo.origin.y * output.scale() as i32,
                                         geo.size.h * output.scale(),
                                         dim[1] as u32))
                    .h((geo.size.h * output.scale()) as f64)
                    .w((self.width * output.scale()) as f64)
                    .color(indicator_color)
                    .border(0.0)
                    .set(ids[1], ui);

                // up
                Canvas::new()
                    .x_y(x_wlc_to_conrod((geo.origin.x - self.width as i32) * output.scale() as i32,
                                         (geo.size.w + self.width * 2) * output.scale(),
                                         dim[0] as u32),
                         y_wlc_to_conrod((geo.origin.y - self.width as i32) * output.scale() as i32,
                                         self.width * output.scale(),
                                         dim[1] as u32))
                    .h((self.width * output.scale()) as f64)
                    .w(((geo.size.w + self.width * 2) * output.scale()) as f64)
                    .color(indicator_color)
                    .border(0.0)
                    .set(ids[2], ui);

                // down
                Canvas::new()
                    .x_y(x_wlc_to_conrod((geo.origin.x - self.width as i32) * output.scale() as i32,
                                         (geo.size.w + self.width * 2) * output.scale(),
                                         dim[0] as u32),
                         y_wlc_to_conrod((geo.origin.y + geo.size.h as i32) * output.scale() as i32,
                                         self.width * output.scale(),
                                         dim[1] as u32))
                    .h((self.width * output.scale()) as f64)
                    .w(((geo.size.w + self.width * 2) * output.scale()) as f64)
                    .color(indicator_color)
                    .border(0.0)
                    .set(ids[3], ui);
            });
        }
    }
}
