use crate::types::{Dimensions2D, ResourceFormat, ResourceState};
use std::ffi::c_void;

/// Safe, zero-dependency abstraction of a GPU resource (Image or Buffer) for FSR.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FsrResource {
    /// Raw GPU resource handle (e.g. `VkImage`, `VkImageView`, or `ID3D12Resource*` cast to `*mut c_void`).
    pub resource: *mut c_void,
    /// Dimensions of the resource.
    pub dimensions: Dimensions2D,
    /// Format of the resource.
    pub format: ResourceFormat,
    /// Current state of the resource in the GPU pipeline.
    pub state: ResourceState,
    /// Number of mip levels (0 or 1 for standard textures).
    pub mip_count: u32,
    /// Depth or array layer count.
    pub depth: u32,
}

impl Default for FsrResource {
    fn default() -> Self {
        Self {
            resource: std::ptr::null_mut(),
            dimensions: Dimensions2D::default(),
            format: ResourceFormat::Unknown,
            state: ResourceState::COMMON,
            mip_count: 1,
            depth: 1,
        }
    }
}

impl FsrResource {
    /// Create a null/empty resource descriptor.
    #[inline]
    #[must_use]
    pub const fn null() -> Self {
        Self {
            resource: std::ptr::null_mut(),
            dimensions: Dimensions2D {
                width: 0,
                height: 0,
            },
            format: ResourceFormat::Unknown,
            state: ResourceState::COMMON,
            mip_count: 0,
            depth: 0,
        }
    }

    /// Construct a 2D Texture resource from a raw pointer handle.
    #[inline]
    pub fn texture2d(
        handle: *mut c_void,
        width: u32,
        height: u32,
        format: ResourceFormat,
        state: ResourceState,
    ) -> Self {
        Self {
            resource: handle,
            dimensions: Dimensions2D::new(width, height),
            format,
            state,
            mip_count: 1,
            depth: 1,
        }
    }

    /// Construct a 2D Texture resource from a `u64` raw handle (e.g. `ash::vk::Image::as_raw()`).
    #[inline]
    pub fn texture2d_from_u64(
        handle: u64,
        width: u32,
        height: u32,
        format: ResourceFormat,
        state: ResourceState,
    ) -> Self {
        Self::texture2d(handle as usize as *mut c_void, width, height, format, state)
    }

    /// Convert to the raw C-ABI `fsr_sys::FfxApiResource` structure.
    #[inline]
    #[must_use]
    pub fn to_sys(self) -> fsr_sys::FfxApiResource {
        fsr_sys::FfxApiResource {
            resource: self.resource,
            description: fsr_sys::FfxApiResourceDescription {
                type_: fsr_sys::FFX_API_RESOURCE_TYPE_TEXTURE2D as _,
                format: self.format as u32 as _,
                __bindgen_anon_1: fsr_sys::FfxApiResourceDescription__bindgen_ty_1 {
                    width: self.dimensions.width,
                },
                __bindgen_anon_2: fsr_sys::FfxApiResourceDescription__bindgen_ty_2 {
                    height: self.dimensions.height,
                },
                __bindgen_anon_3: fsr_sys::FfxApiResourceDescription__bindgen_ty_3 {
                    depth: self.depth,
                },
                mipCount: self.mip_count,
                flags: fsr_sys::FFX_API_RESOURCE_FLAGS_NONE as _,
                usage: (fsr_sys::FFX_API_RESOURCE_USAGE_RENDERTARGET as u32
                    | fsr_sys::FFX_API_RESOURCE_USAGE_UAV as u32) as _,
            },
            state: self.state.bits() as _,
        }
    }
}
