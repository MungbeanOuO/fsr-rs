use crate::types::{FsrError, ResourceFormat};
use std::ffi::c_void;

/// Description of a GPU resource requested by FSR internal passes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FsrResourceRequest {
    pub effect_id: u32,
    pub width: u32,
    pub height: u32,
    pub depth: u32,
    pub mip_count: u32,
    pub format: ResourceFormat,
    pub usage: u32,
}

/// Allocated GPU resource handles returned by custom allocator.
#[derive(Debug, Clone, Copy)]
pub struct FsrAllocatedResource {
    /// Vulkan Image or Buffer handle (`VkImage` / `VkBuffer` as `*mut c_void`).
    pub resource: *mut c_void,
    /// Vulkan Device Memory handle (`VkDeviceMemory` as `*mut c_void`).
    pub device_memory: *mut c_void,
}

/// Trait allowing host engines to intercept FSR scratch memory allocations
/// and integrate them into host memory pools or RenderGraph memory aliasing.
pub trait FsrAllocator: Send + Sync {
    /// Allocate a GPU resource (Image or Buffer).
    fn allocate_resource(
        &mut self,
        request: &FsrResourceRequest,
    ) -> Result<FsrAllocatedResource, FsrError>;

    /// Deallocate a previously allocated GPU resource.
    fn deallocate_resource(
        &mut self,
        effect_id: u32,
        resource: *mut c_void,
        device_memory: *mut c_void,
    ) -> Result<(), FsrError>;
}
