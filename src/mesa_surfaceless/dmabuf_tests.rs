//! End-to-end DMABuf export tests for the Mesa surfaceless backend.
//!
//! These run against a real EGL device when one is available. If no device can
//! be created, or the `EGL_MESA_image_dma_buf_export` extension is missing, the
//! test skips gracefully so CI without a GPU/extension stays green.

#![allow(missing_docs)]

use super::connection::Connection;
use super::device::Device;
use crate::{
    gl, ContextAttributeFlags, ContextAttributes, Error, GLVersion, Gl, SurfaceAccess, SurfaceType,
};

use euclid::default::Size2D;
use glow::HasContext;
use std::os::fd::AsRawFd;

#[test]
fn export_generic_surface_to_dmabuf() {
    let connection = Connection::new().expect("Failed to create connection");
    let adapter = connection
        .create_adapter(Default::default())
        .expect("Failed to create adapter");
    let mut device = match connection.create_device(&adapter) {
        Ok(device) => device,
        Err(Error::RequiredExtensionUnavailable) => {
            eprintln!("Skipping: required EGL extension unavailable on this host.");
            return;
        }
        Err(err) => panic!("Failed to create device: {err:?}"),
    };

    let descriptor = device
        .create_context_descriptor(&ContextAttributes {
            version: GLVersion::new(3, 0),
            flags: ContextAttributeFlags::empty(),
        })
        .expect("Failed to create context descriptor");
    let mut context = device
        .create_context(&descriptor, None)
        .expect("Failed to create context");

    let gl =
        unsafe { Gl::from_loader_function(|symbol| device.get_proc_address(&context, symbol)) };

    let surface = device
        .create_surface(
            &context,
            SurfaceAccess::GPUOnly,
            SurfaceType::Generic {
                size: Size2D::new(64, 48),
            },
        )
        .expect("Failed to create surface");
    device
        .bind_surface_to_context(&mut context, surface)
        .expect("Failed to bind surface");
    device
        .make_context_current(&context)
        .expect("Failed to make context current");

    unsafe {
        let framebuffer_object = device
            .context_surface_info(&context)
            .unwrap()
            .unwrap()
            .framebuffer_object;
        gl.bind_framebuffer(gl::FRAMEBUFFER, framebuffer_object);
        gl.clear_color(0.0, 1.0, 0.0, 1.0);
        gl.clear(gl::COLOR_BUFFER_BIT);
        gl.finish();
    }

    // Unbind the surface so we can inspect it, then export it.
    let mut surface = device
        .unbind_surface_from_context(&mut context)
        .expect("Failed to unbind surface")
        .expect("Expected a bound surface");

    match device.export_surface_to_dmabuf(&surface) {
        Ok(descriptor) => {
            assert!(
                !descriptor.planes.is_empty(),
                "a successful export must report at least one plane"
            );
            assert_eq!(
                descriptor.fds.len(),
                descriptor.planes.len(),
                "there must be one fd per plane"
            );
            assert_eq!(descriptor.width, 64);
            assert_eq!(descriptor.height, 48);
            assert_ne!(descriptor.drm_fourcc, 0, "fourcc should be a real format");

            for fd in &descriptor.fds {
                assert!(
                    fd.as_raw_fd() >= 0,
                    "each exported fd must be a valid file descriptor"
                );
            }

            eprintln!(
                "Exported dmabuf: fourcc=0x{:08x} modifier=0x{:016x} planes={}",
                descriptor.drm_fourcc,
                descriptor.drm_modifier,
                descriptor.planes.len()
            );
        }
        Err(Error::UnsupportedOnThisPlatform) => {
            eprintln!(
                "Skipping assertions: EGL_MESA_image_dma_buf_export unavailable on this host."
            );
        }
        Err(err) => panic!("export_surface_to_dmabuf failed unexpectedly: {err:?}"),
    }

    device
        .destroy_surface(&mut context, &mut surface)
        .expect("Failed to destroy surface");
    device
        .destroy_context(&mut context)
        .expect("Failed to destroy context");
}
