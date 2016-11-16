use callback::{AsWrapper, IntoCallback, Wrapper};
use handlers::render::gl::GLInit;

use handlers::store::{Store, StoreKey};
use opengles_graphics::{GlGraphics, OpenGL, gl};
use slog;
use slog_scope;
use wlc::{Callback, Output};
use wlc::render::{RenderOutput, RenderView};

/// Handler to draw using pistons `graphics` API for 2D drawing
/// inside the compositor.
///
/// It uses and initializes the unstable `GLInit` handler internally,
/// backing up and restoring the OpenGL state, so you do not need to
/// worry about any of this and may use this stable API for rendering.
///
/// The `GlGraphics` struct is exposed via any `Output`s `Store` to
/// do the rendering.
///
/// ## Dependencies
///
/// - [`StoreHandler`](./struct.StoreHandler.html)
///
pub struct GraphicsRenderer<C: Callback> {
    child: C,
    logger: slog::Logger,
}

impl StoreKey for GlGraphics {
    type Value = GlGraphics;
}

impl<C: Callback + 'static> AsWrapper for GraphicsRenderer<C> {
    fn child(&mut self) -> Option<&mut Callback> {
        Some(&mut self.child)
    }
}

impl<C: Callback + 'static> Callback for Wrapper<GraphicsRenderer<C>> {
    fn output_context_created(&mut self, output: &Output) {
        output.insert::<GlGraphics>(GlGraphics::new(OpenGL::V2_1));
        debug!(self.logger, "GlGraphics initialized");
        self.child.output_context_created(output)
    }

    fn output_context_destroyed(&mut self, output: &Output) {
        self.child.output_context_destroyed(output);
        output.remove::<GlGraphics>();
    }

    fn output_render_pre(&mut self, output: &mut RenderOutput) {
        {
            let graphics = output.get::<GlGraphics>().unwrap();
            let mut lock = graphics.write().unwrap();
            lock.clear_draw_state();
            lock.clear_program();
        }
        unsafe {
            self.wrap(move |self_ref: &mut GraphicsRenderer<C>| self_ref.child.output_render_pre(output))
        }
    }

    fn output_render_post(&mut self, output: &mut RenderOutput) {
        {
            let graphics = output.get::<GlGraphics>().unwrap();
            let mut lock = graphics.write().unwrap();
            lock.clear_draw_state();
            lock.clear_program();
        }
        unsafe {
            self.wrap(move |self_ref: &mut GraphicsRenderer<C>| self_ref.child.output_render_post(output))
        }
    }

    fn view_render_pre(&mut self, view: &mut RenderView) {
        unsafe { self.wrap(move |self_ref: &mut GraphicsRenderer<C>| self_ref.child.view_render_pre(view)) }
    }

    fn view_render_post(&mut self, view: &mut RenderView) {
        unsafe { self.wrap(move |self_ref: &mut GraphicsRenderer<C>| self_ref.child.view_render_post(view)) }
    }
}

impl<C: Callback + 'static> GraphicsRenderer<C> {
    /// Initialize a new `GraphicsRenderer`
    pub fn new<I: IntoCallback<C>>(renderer: I) -> GLInit<Wrapper<GraphicsRenderer<C>>> {
        let renderer = GraphicsRenderer {
            child: renderer.into_callback(),
            logger: slog_scope::logger().new(o!("handler" => "GraphicsRenderer")),
        };
        unsafe { GLInit::new::<GraphicsRenderer<C>>(renderer) }
    }

    unsafe fn wrap<F, R>(&mut self, func: F) -> R
        where F: FnOnce(&mut Self) -> R
    {
        // backup modified opengl state
        //

        trace!(self.logger, "Backup GLState");

        let mut viewport: [gl::types::GLint; 4] = [-1, -1, -1, -1];
        gl::GetIntegerv(gl::VIEWPORT, viewport.as_mut_ptr());

        let mut program: gl::types::GLint = -1;
        gl::GetIntegerv(gl::CURRENT_PROGRAM, &mut program as *mut _);

        let mut vao: gl::types::GLint = -1;
        gl::GetIntegerv(gl::VERTEX_ARRAY_BINDING, &mut vao as *mut _);

        let mut vbo: gl::types::GLint = -1;
        gl::GetIntegerv(gl::ARRAY_BUFFER_BINDING, &mut vbo as *mut _);

        let is_cullface = gl::IsEnabled(gl::CULL_FACE) == gl::TRUE;

        let mut texture: gl::types::GLint = -1;
        gl::GetIntegerv(gl::TEXTURE_BINDING_2D, &mut texture as *mut _);

        let scissor = match gl::IsEnabled(gl::SCISSOR_TEST) {
            gl::TRUE => {
                let mut scissor: [gl::types::GLint; 4] = [0, 0, 0, 0];
                gl::GetIntegerv(gl::SCISSOR_BOX, scissor.as_mut_ptr());
                Some(scissor)
            }
            _ => None,
        };

        let stencil = match gl::IsEnabled(gl::STENCIL_TEST) {
            gl::TRUE => {
                let mut stencil_func: gl::types::GLint = -1;
                gl::GetIntegerv(gl::STENCIL_FUNC, &mut stencil_func as *mut _);

                let mut stencil_ref: gl::types::GLint = -1;
                gl::GetIntegerv(gl::STENCIL_REF, &mut stencil_ref as *mut _);

                let mut stencil_value_mask: gl::types::GLint = -1;
                gl::GetIntegerv(gl::STENCIL_VALUE_MASK, &mut stencil_value_mask as *mut _);

                let mut stencil_writemask: gl::types::GLint = -1;
                gl::GetIntegerv(gl::STENCIL_WRITEMASK, &mut stencil_writemask as *mut _);

                let mut stencil_op_fail: gl::types::GLint = -1;
                gl::GetIntegerv(gl::STENCIL_FAIL, &mut stencil_op_fail as *mut _);

                let mut stencil_op_pass_depth_fail: gl::types::GLint = -1;
                gl::GetIntegerv(gl::STENCIL_PASS_DEPTH_FAIL,
                                &mut stencil_op_pass_depth_fail as *mut _);

                let mut stencil_op_pass_depth_pass: gl::types::GLint = -1;
                gl::GetIntegerv(gl::STENCIL_PASS_DEPTH_PASS,
                                &mut stencil_op_pass_depth_pass as *mut _);

                Some((stencil_func,
                      stencil_ref,
                      stencil_value_mask,
                      stencil_writemask,
                      stencil_op_fail,
                      stencil_op_pass_depth_fail,
                      stencil_op_pass_depth_pass))
            }
            _ => None,
        };
        let blend = match gl::IsEnabled(gl::BLEND) {
            gl::TRUE => {
                let mut blend_color: [gl::types::GLfloat; 4] = [0.0, 0.0, 0.0, 0.0];
                gl::GetFloatv(gl::BLEND_COLOR, blend_color.as_mut_ptr());

                let mut blend_equation_rgb: gl::types::GLint = -1;
                gl::GetIntegerv(gl::BLEND_EQUATION_RGB, &mut blend_equation_rgb as *mut _);

                let mut blend_equation_alpha: gl::types::GLint = -1;
                gl::GetIntegerv(gl::BLEND_EQUATION_ALPHA,
                                &mut blend_equation_alpha as *mut _);

                let mut blend_src_rgb: gl::types::GLint = -1;
                gl::GetIntegerv(gl::BLEND_SRC_RGB, &mut blend_src_rgb as *mut _);

                let mut blend_dst_rgb: gl::types::GLint = -1;
                gl::GetIntegerv(gl::BLEND_DST_RGB, &mut blend_dst_rgb as *mut _);

                let mut blend_src_alpha: gl::types::GLint = -1;
                gl::GetIntegerv(gl::BLEND_SRC_ALPHA, &mut blend_src_alpha as *mut _);

                let mut blend_dst_alpha: gl::types::GLint = -1;
                gl::GetIntegerv(gl::BLEND_DST_ALPHA, &mut blend_dst_alpha as *mut _);

                Some((blend_color,
                      blend_equation_rgb,
                      blend_equation_alpha,
                      blend_src_rgb,
                      blend_dst_rgb,
                      blend_src_alpha,
                      blend_dst_alpha))
            }
            _ => None,
        };

        // Render
        //
        trace!(self.logger, "Rendering");

        let result = func(self);

        trace!(self.logger, "Rendering done");

        // And restore it
        //

        match blend {
            Some((blend_color,
                  blend_equation_rgb,
                  blend_equation_alpha,
                  blend_src_rgb,
                  blend_dst_rgb,
                  blend_src_alpha,
                  blend_dst_alpha)) => {
                gl::Enable(gl::BLEND);
                gl::BlendColor(blend_color[0],
                               blend_color[1],
                               blend_color[2],
                               blend_color[3]);
                gl::BlendEquationSeparate(blend_equation_rgb as gl::types::GLuint,
                                          blend_equation_alpha as gl::types::GLuint);
                gl::BlendFuncSeparate(blend_src_rgb as gl::types::GLuint,
                                      blend_dst_rgb as gl::types::GLuint,
                                      blend_src_alpha as gl::types::GLuint,
                                      blend_dst_alpha as gl::types::GLuint);
            }
            None => gl::Disable(gl::BLEND),
        };

        match stencil {
            Some((stencil_func,
                  stencil_ref,
                  stencil_value_mask,
                  stencil_writemask,
                  stencil_op_fail,
                  stencil_op_pass_depth_fail,
                  stencil_op_pass_depth_pass)) => {
                gl::Enable(gl::STENCIL_TEST);
                gl::StencilFunc(stencil_func as gl::types::GLuint,
                                stencil_ref,
                                stencil_value_mask as gl::types::GLuint);
                gl::StencilMask(stencil_writemask as gl::types::GLuint);
                gl::StencilOp(stencil_op_fail as gl::types::GLuint,
                              stencil_op_pass_depth_fail as gl::types::GLuint,
                              stencil_op_pass_depth_pass as gl::types::GLuint);
            }
            None => gl::Disable(gl::STENCIL_TEST),
        };

        match scissor {
            Some(scissor) => {
                gl::Enable(gl::SCISSOR_TEST);
                gl::Scissor(scissor[0], scissor[1], scissor[2], scissor[3]);
            }
            None => gl::Disable(gl::SCISSOR_TEST),
        };

        gl::BindTexture(gl::TEXTURE_2D, texture as gl::types::GLuint);

        if is_cullface {
            gl::Enable(gl::CULL_FACE)
        } else {
            gl::Disable(gl::CULL_FACE)
        };

        gl::BindBuffer(gl::ARRAY_BUFFER, vbo as gl::types::GLuint);

        gl::BindVertexArray(vao as gl::types::GLuint);

        gl::UseProgram(program as gl::types::GLuint);

        gl::Viewport(viewport[0], viewport[1], viewport[2], viewport[3]);

        trace!(self.logger, "GLState Restored");

        result
    }
}
