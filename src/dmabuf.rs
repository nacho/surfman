//! Linux DMABuf export descriptors.
//!
//! A [`DmabufDescriptor`] describes a GPU surface exported as one or more Linux
//! DMABuf file descriptors, together with the metadata (DRM format, modifier,
//! and per-plane layout) a consumer needs to import the buffer — for example
//! via `zwp_linux_dmabuf_v1`, EGL's `EGL_EXT_image_dma_buf_import`, or GTK's
//! `GdkDmabufTextureBuilder`.
//!
//! Descriptors are produced by the EGL-based backends' `export_surface_to_dmabuf`
//! device method (available on the Wayland and Mesa surfaceless backends when the
//! `EGL_MESA_image_dma_buf_export` extension is present). The owned file
//! descriptors can be sent across a process boundary (for example over a Unix
//! socket) to enable zero-copy sharing of rendered frames.

use std::os::fd::OwnedFd;

/// The layout of a single plane within an exported DMABuf.
///
/// Each plane corresponds to one of the file descriptors in the owning
/// [`DmabufDescriptor::fds`] list, at the same index.
#[derive(Debug)]
pub struct DmabufPlane {
    /// The byte offset from the start of the buffer to the first pixel of this
    /// plane.
    pub offset: i32,
    /// The number of bytes between consecutive rows of this plane.
    pub stride: i32,
}

/// A GPU surface exported as a set of Linux DMABuf file descriptors plus the
/// metadata required to import it in another context or process.
///
/// The `fds`, and the entries of `planes`, are parallel: plane `i` is backed by
/// `fds[i]` and described by `planes[i]`. For single-plane formats both lists
/// have length one.
///
/// The file descriptors are owned; dropping the descriptor closes them.
#[derive(Debug)]
pub struct DmabufDescriptor {
    /// The owned DMABuf file descriptors, one per plane.
    pub fds: Vec<OwnedFd>,
    /// The width of the surface, in pixels.
    pub width: i32,
    /// The height of the surface, in pixels.
    pub height: i32,
    /// The DRM FourCC format code of the buffer (for example `DRM_FORMAT_ABGR8888`).
    pub drm_fourcc: u32,
    /// The DRM format modifier describing the tiling/compression layout of the
    /// buffer. `DRM_FORMAT_MOD_INVALID` indicates an implicit/unknown modifier.
    pub drm_modifier: u64,
    /// The per-plane layout, parallel to [`DmabufDescriptor::fds`].
    pub planes: Vec<DmabufPlane>,
}

#[cfg(test)]
mod test {
    use super::{DmabufDescriptor, DmabufPlane};

    #[test]
    fn dmabuf_descriptor_can_be_constructed() {
        let plane = DmabufPlane {
            offset: 0,
            stride: 256,
        };
        assert_eq!(plane.offset, 0);
        assert_eq!(plane.stride, 256);

        let descriptor = DmabufDescriptor {
            fds: Vec::new(),
            width: 64,
            height: 48,
            drm_fourcc: 0x3432_4258, // DRM_FORMAT_ABGR8888 ('AB24')
            drm_modifier: 0x00ff_ffff_ffff_ffff,
            planes: vec![plane],
        };

        assert!(descriptor.fds.is_empty());
        assert_eq!(descriptor.width, 64);
        assert_eq!(descriptor.height, 48);
        assert_eq!(descriptor.planes.len(), 1);
    }
}
