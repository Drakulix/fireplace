use smithay::{
    backend::{
        allocator::dmabuf::Dmabuf,
        drm::{DrmError, DrmSurface, GbmBufferedSurface, GbmBufferedSurfaceError::DrmError as GbmDrmError},
        egl::{EGLDisplay, EGLContext, surface::EGLSurface},
        renderer::{Bind, Renderer, Transform},
        SwapBuffersError
    },
    reexports::{
        drm::control::crtc,
        gbm::{Device as GbmDevice},
    },
};

use std::{
    rc::Rc,
};

mod eglstream;
pub use self::eglstream::*;
use super::SessionFd;

pub enum RenderSurface {
    Gbm(GbmBufferedSurface<SessionFd>),
    Egl(Rc<EGLSurface>, crtc::Handle),
}
use RenderSurface::*;

impl RenderSurface {
    pub fn new_gbm(surf: DrmSurface<SessionFd>, fd: SessionFd, ctx: &EGLContext) -> anyhow::Result<RenderSurface> {
        let gbm_device = GbmDevice::new(fd)?;
        let gbm_surface = GbmBufferedSurface::new(surf, gbm_device, ctx.dmabuf_render_formats().clone(), None)?;
        Ok(RenderSurface::Gbm(gbm_surface))
    }

    pub fn new_eglstream(surf: DrmSurface<SessionFd>, disp: &EGLDisplay, ctx: &EGLContext) -> anyhow::Result<RenderSurface> {
        let stream_surface = EglStreamSurface::new(surf, slog_scope::logger());
        let crtc = stream_surface.crtc();
        let egl_surface = Rc::new(EGLSurface::new(
                &disp,
                ctx.pixel_format().unwrap(),
                ctx.config_id(),
                stream_surface,
                None,
            )?);
        Ok(RenderSurface::Egl(egl_surface, crtc))
    }

    pub fn bind<B: Bind<Dmabuf> + Bind<Rc<EGLSurface>>>(&mut self, renderer: &mut B) -> Result<(), B::Error> {
        match self {
            Gbm(surf) => {
                let dmabuf = surf.next_buffer().unwrap();
                renderer.bind(dmabuf)
            },
            Egl(surf, _) => {
                renderer.bind(surf.clone())
            },
        }
    }

    pub fn queue_buffer<B, E>(&mut self, renderer: &mut B) -> Result<(), SwapBuffersError>
    where
        B: Bind<Rc<EGLSurface>> + Renderer<Error=E>,
        E: Into<SwapBuffersError> + std::error::Error,
    {
        match self {
            Gbm(surf) => { surf.queue_buffer().map_err(Into::into) },
            Egl(surf, _) => {
                renderer.bind(surf.clone()).map_err(Into::into)?;
                renderer.render((0, 0).into(), smithay::backend::renderer::Transform::Normal, |_,_| {}).map_err(Into::into)?;
                surf.swap_buffers().map_err(Into::into)
            }
        }
    }

    pub fn frame_submitted(&mut self) -> Result<(), DrmError> {
        match self {
            // yeah, its a hack, i'll fix it later
            Gbm(surf) => surf.frame_submitted().map_err(|e| match e { GbmDrmError(e) => e, _ => unreachable!() }),
            _ => Ok(()), // we do not need to release frames for Eglstreams
        }
    }
    
    pub fn crtc(&self) -> crtc::Handle {
        match self {
            Gbm(surf) => surf.crtc(),
            Egl(_, crtc) => *crtc,
        }
    }

    pub fn transform(&self, transform: Transform) -> Transform {
        match self {
            Gbm(_) => match transform {
                Transform::Normal => Transform::Flipped180,
                Transform::_90 => Transform::Flipped270,
                Transform::_180 => Transform::Flipped,
                Transform::_270 => Transform::Flipped90,
                Transform::Flipped => Transform::_180,
                Transform::Flipped90 => Transform::_270,
                Transform::Flipped180 => Transform::Normal,
                Transform::Flipped270 => Transform::_90,
            },
            Egl(_, _) => transform,
        }
    }
}