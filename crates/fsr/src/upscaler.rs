use crate::resource::FsrResource;
use crate::types::{Dimensions2D, DispatchFlags, FloatCoords2D, FsrError, InitializationFlags};
use std::ffi::c_void;

/// Vulkan backend configuration for FSR.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VulkanBackendConfig {
    /// Raw `VkDevice` pointer or handle (`*mut c_void`).
    pub device: *mut c_void,
    /// Raw `VkPhysicalDevice` pointer or handle (`*mut c_void`).
    pub physical_device: *mut c_void,
    /// Raw `VkInstance` pointer or handle (`*mut c_void`).
    pub instance: *mut c_void,
    /// Function pointer address of `vkGetDeviceProcAddr`.
    pub get_device_proc_addr: usize,
}

impl VulkanBackendConfig {
    /// Construct Vulkan backend config from raw pointers.
    #[inline]
    pub const fn new(
        device: *mut c_void,
        physical_device: *mut c_void,
        instance: *mut c_void,
        get_device_proc_addr: usize,
    ) -> Self {
        Self {
            device,
            physical_device,
            instance,
            get_device_proc_addr,
        }
    }

    /// Construct Vulkan backend config from standard `u64` raw handles (e.g. from `ash` or `vulkano`).
    #[inline]
    pub fn from_raw_u64(
        device: u64,
        physical_device: u64,
        instance: u64,
        get_device_proc_addr: usize,
    ) -> Self {
        Self::new(
            device as usize as *mut c_void,
            physical_device as usize as *mut c_void,
            instance as usize as *mut c_void,
            get_device_proc_addr,
        )
    }
}

/// DirectX 12 backend configuration for FSR.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Dx12BackendConfig {
    /// Raw pointer to `ID3D12Device` (`*mut c_void` or `u64`).
    pub device: *mut c_void,
}

impl Dx12BackendConfig {
    /// Construct DirectX 12 backend config from raw pointer to `ID3D12Device`.
    #[inline]
    pub const fn new(device: *mut c_void) -> Self {
        Self { device }
    }

    /// Construct DirectX 12 backend config from standard `u64` raw handle.
    #[inline]
    pub fn from_raw_u64(device: u64) -> Self {
        Self::new(device as usize as *mut c_void)
    }
}

/// Graphics API backend configuration enum supporting both Vulkan and DirectX 12.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FsrBackendConfig {
    /// Vulkan backend configuration.
    Vulkan(VulkanBackendConfig),
    /// DirectX 12 backend configuration.
    Dx12(Dx12BackendConfig),
}

impl From<VulkanBackendConfig> for FsrBackendConfig {
    #[inline]
    fn from(cfg: VulkanBackendConfig) -> Self {
        Self::Vulkan(cfg)
    }
}

impl From<Dx12BackendConfig> for FsrBackendConfig {
    #[inline]
    fn from(cfg: Dx12BackendConfig) -> Self {
        Self::Dx12(cfg)
    }
}

/// Configuration parameters for creating an `FsrUpscaler` context.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FsrUpscalerConfig {
    /// Maximum internal rendering resolution.
    pub max_render_size: Dimensions2D,
    /// Maximum presentation (target) resolution.
    pub max_upscale_size: Dimensions2D,
    /// Initialization flags.
    pub flags: InitializationFlags,
    /// Backend graphics API configuration (Vulkan or DirectX 12).
    pub backend: FsrBackendConfig,
    /// Optional version override ID (e.g. selected from `FsrUpscaler::query_available_versions` to invoke FSR 4 ML).
    pub version_override: Option<u64>,
}

impl FsrUpscalerConfig {
    /// Helper to create a standard configuration for Vulkan backend.
    pub fn vulkan(
        max_render_size: impl Into<Dimensions2D>,
        max_upscale_size: impl Into<Dimensions2D>,
        flags: InitializationFlags,
        backend: VulkanBackendConfig,
    ) -> Self {
        Self {
            max_render_size: max_render_size.into(),
            max_upscale_size: max_upscale_size.into(),
            flags,
            backend: FsrBackendConfig::Vulkan(backend),
            version_override: None,
        }
    }

    /// Helper to create a standard configuration for DirectX 12 backend.
    pub fn dx12(
        max_render_size: impl Into<Dimensions2D>,
        max_upscale_size: impl Into<Dimensions2D>,
        flags: InitializationFlags,
        backend: Dx12BackendConfig,
    ) -> Self {
        Self {
            max_render_size: max_render_size.into(),
            max_upscale_size: max_upscale_size.into(),
            flags,
            backend: FsrBackendConfig::Dx12(backend),
            version_override: None,
        }
    }
}

/// Per-frame parameter descriptor for FSR temporal upscaling dispatch.
#[derive(Debug, Clone, PartialEq)]
pub struct FsrUpscaleDispatchDesc {
    /// Raw command buffer handle (`VkCommandBuffer` or `ID3D12GraphicsCommandList*` as `*mut c_void`).
    pub command_buffer: *mut c_void,
    /// Color buffer for the current frame at render resolution.
    pub color: FsrResource,
    /// 32-bit depth buffer at render resolution.
    pub depth: FsrResource,
    /// 2D motion vector buffer (screen space or display space).
    pub motion_vectors: FsrResource,
    /// Target presentation buffer for the upscaled output color.
    pub output: FsrResource,
    /// Optional 1x1 luminance/exposure texture.
    pub exposure: Option<FsrResource>,
    /// Optional reactivity mask for reactive objects (particles, UI, hair).
    pub reactive_mask: Option<FsrResource>,
    /// Optional transparency & composition mask.
    pub transparency_and_composition_mask: Option<FsrResource>,
    /// Subpixel camera jitter offset applied to this frame's projection.
    pub jitter_offset: FloatCoords2D,
    /// Motion vector scale factor (usually `(1.0, 1.0)` or `(render_w, render_h)`).
    pub motion_vector_scale: FloatCoords2D,
    /// Input render resolution for the current frame.
    pub render_size: Dimensions2D,
    /// Target presentation resolution for the current frame.
    pub upscale_size: Dimensions2D,
    /// Enable additional sharpening pass (RCAS).
    pub enable_sharpening: bool,
    /// Sharpness value between 0.0 (no sharpening) and 1.0 (maximum sharpness).
    pub sharpness: f32,
    /// Flag indicating if temporal accumulation history should be reset (camera cut / scene switch).
    pub reset_accumulation: bool,
    /// Camera near plane distance in world units.
    pub camera_near: f32,
    /// Camera far plane distance in world units (or 0.0 if using infinite far plane).
    pub camera_far: f32,
    /// Camera vertical field of view angle in radians.
    pub camera_fov_y: f32,
    /// Factor converting view space units to meters (default: 1.0).
    pub view_space_to_meters_factor: f32,
    /// Frame delta time in seconds (converted to milliseconds internally).
    pub delta_time_seconds: f32,
    /// Pre-exposure value applied to input color before tonemapping (default: 1.0).
    pub pre_exposure: f32,
    /// Per-dispatch flags (e.g. debug view, non-linear colors).
    pub flags: DispatchFlags,
}

impl FsrUpscaleDispatchDesc {
    /// Build C-ABI dispatch descriptor struct.
    pub fn to_sys(&self) -> fsr_sys::FfxDispatchDescUpscale {
        fsr_sys::FfxDispatchDescUpscale {
            header: fsr_sys::FfxApiHeader {
                type_: fsr_sys::FFX_API_DISPATCH_DESC_TYPE_UPSCALE,
                pNext: std::ptr::null_mut(),
            },
            commandList: self.command_buffer,
            color: self.color.to_sys(),
            depth: self.depth.to_sys(),
            motionVectors: self.motion_vectors.to_sys(),
            exposure: self.exposure.unwrap_or_else(FsrResource::null).to_sys(),
            reactive: self
                .reactive_mask
                .unwrap_or_else(FsrResource::null)
                .to_sys(),
            transparencyAndComposition: self
                .transparency_and_composition_mask
                .unwrap_or_else(FsrResource::null)
                .to_sys(),
            output: self.output.to_sys(),
            jitterOffset: self.jitter_offset.into(),
            motionVectorScale: self.motion_vector_scale.into(),
            renderSize: self.render_size.into(),
            upscaleSize: self.upscale_size.into(),
            enableSharpening: self.enable_sharpening,
            sharpness: self.sharpness.clamp(0.0, 1.0),
            frameTimeDelta: self.delta_time_seconds * 1000.0,
            preExposure: self.pre_exposure,
            reset: self.reset_accumulation,
            cameraNear: self.camera_near,
            cameraFar: self.camera_far,
            cameraFovAngleVertical: self.camera_fov_y,
            viewSpaceToMetersFactor: self.view_space_to_meters_factor,
            flags: self.flags.bits(),
        }
    }
}

/// RAII wrapper managing an AMD FSR Upscaler Context.
pub struct FsrUpscaler {
    context: fsr_sys::ffxContext,
    is_created: bool,
}

unsafe impl Send for FsrUpscaler {}
unsafe impl Sync for FsrUpscaler {}

impl FsrUpscaler {
    /// Query all available upscaler version IDs and names supported by the underlying driver or DLL.
    ///
    /// Useful for runtime version enumeration and enabling FSR 4 ML Neural Network models dynamically.
    pub fn query_available_versions(
        backend: &FsrBackendConfig,
    ) -> Result<Vec<(u64, String)>, FsrError> {
        let mut vk_context = match backend {
            FsrBackendConfig::Vulkan(vk) => Some(fsr_sys::VkDeviceContext {
                vkDevice: vk.device as _,
                vkPhysicalDevice: vk.physical_device as _,
                vkDeviceProcAddr: if vk.get_device_proc_addr != 0 {
                    unsafe {
                        std::mem::transmute::<usize, fsr_sys::PFN_vkGetDeviceProcAddr>(
                            vk.get_device_proc_addr,
                        )
                    }
                } else {
                    None
                },
            }),
            FsrBackendConfig::Dx12(_) => None,
        };

        let device_ptr = match backend {
            FsrBackendConfig::Vulkan(_) => {
                vk_context.as_mut().unwrap() as *mut fsr_sys::VkDeviceContext as *mut c_void
            }
            FsrBackendConfig::Dx12(dx) => dx.device,
        };

        let mut count: u64 = 0;
        let mut query_desc = fsr_sys::FfxQueryDescGetVersions {
            header: fsr_sys::ffxQueryDescHeader {
                type_: fsr_sys::FFX_API_QUERY_DESC_TYPE_GET_VERSIONS as u64,
                pNext: std::ptr::null_mut(),
            },
            createDescType: fsr_sys::FFX_API_CREATE_CONTEXT_DESC_TYPE_UPSCALE,
            device: device_ptr,
            outputCount: &mut count,
            versionIds: std::ptr::null_mut(),
            versionNames: std::ptr::null_mut(),
        };

        unsafe {
            // First call: query the number of available versions
            let ret = fsr_sys::ffxQuery(std::ptr::null_mut(), &mut query_desc.header);
            FsrError::from_return_code(ret)?;

            if count == 0 {
                return Ok(Vec::new());
            }

            let mut version_ids: Vec<u64> = vec![0; count as usize];
            let mut version_names: Vec<*const std::os::raw::c_char> =
                vec![std::ptr::null(); count as usize];

            query_desc.versionIds = version_ids.as_mut_ptr();
            query_desc.versionNames = version_names.as_mut_ptr();

            // Second call: populate IDs and display names
            let ret = fsr_sys::ffxQuery(std::ptr::null_mut(), &mut query_desc.header);
            FsrError::from_return_code(ret)?;

            let mut results = Vec::with_capacity(count as usize);
            for i in 0..count as usize {
                let id = version_ids[i];
                let name_ptr = version_names[i];
                let name = if !name_ptr.is_null() {
                    std::ffi::CStr::from_ptr(name_ptr)
                        .to_string_lossy()
                        .into_owned()
                } else {
                    format!("Version {id}")
                };
                results.push((id, name));
            }

            Ok(results)
        }
    }

    /// Create a new FSR Upscaler RAII context with the provided configuration.
    pub fn new(config: &FsrUpscalerConfig) -> Result<Self, FsrError> {
        let mut backend_vk_desc = fsr_sys::FfxCreateBackendVKDesc {
            header: fsr_sys::FfxApiHeader {
                type_: fsr_sys::FFX_API_CREATE_CONTEXT_DESC_TYPE_BACKEND_VK,
                pNext: std::ptr::null_mut(),
            },
            deviceContext: fsr_sys::VkDeviceContext {
                vkDevice: std::ptr::null_mut(),
                vkPhysicalDevice: std::ptr::null_mut(),
                vkDeviceProcAddr: None,
            },
        };

        let mut backend_dx12_desc = fsr_sys::FfxCreateBackendDX12Desc {
            header: fsr_sys::FfxApiHeader {
                type_: fsr_sys::FFX_API_CREATE_CONTEXT_DESC_TYPE_BACKEND_DX12 as u64,
                pNext: std::ptr::null_mut(),
            },
            device: std::ptr::null_mut(),
        };

        let backend_header_ptr: *mut fsr_sys::FfxApiHeader = match config.backend {
            FsrBackendConfig::Vulkan(vk) => {
                backend_vk_desc.deviceContext.vkDevice = vk.device as _;
                backend_vk_desc.deviceContext.vkPhysicalDevice = vk.physical_device as _;
                backend_vk_desc.deviceContext.vkDeviceProcAddr = if vk.get_device_proc_addr != 0 {
                    unsafe {
                        std::mem::transmute::<usize, fsr_sys::PFN_vkGetDeviceProcAddr>(
                            vk.get_device_proc_addr,
                        )
                    }
                } else {
                    None
                };
                &mut backend_vk_desc.header as *mut fsr_sys::FfxApiHeader
            }
            FsrBackendConfig::Dx12(dx) => {
                backend_dx12_desc.device = dx.device;
                &mut backend_dx12_desc.header as *mut fsr_sys::FfxApiHeader
            }
        };

        let mut override_version_desc = fsr_sys::FfxOverrideVersion {
            header: fsr_sys::FfxApiHeader {
                type_: fsr_sys::FFX_API_DESC_TYPE_OVERRIDE_VERSION as u64,
                pNext: backend_header_ptr,
            },
            versionId: 0,
        };

        let root_pnext: *mut fsr_sys::FfxApiHeader = if let Some(ver_id) = config.version_override {
            override_version_desc.versionId = ver_id;
            &mut override_version_desc.header as *mut fsr_sys::FfxApiHeader
        } else {
            backend_header_ptr
        };

        let mut create_desc = fsr_sys::FfxCreateContextDescUpscale {
            header: fsr_sys::FfxApiHeader {
                type_: fsr_sys::FFX_API_CREATE_CONTEXT_DESC_TYPE_UPSCALE,
                pNext: root_pnext,
            },
            flags: config.flags.bits(),
            maxRenderSize: config.max_render_size.into(),
            maxUpscaleSize: config.max_upscale_size.into(),
            fpMessage: None,
        };

        let mut context: fsr_sys::ffxContext = std::ptr::null_mut();

        unsafe {
            let ret = fsr_sys::ffxCreateContext(
                &mut context,
                &mut create_desc.header as *mut fsr_sys::FfxApiHeader,
                std::ptr::null(),
            );
            FsrError::from_return_code(ret)?;
        }

        Ok(Self {
            context,
            is_created: true,
        })
    }

    /// Record temporal upscaling compute commands into the specified command buffer.
    pub fn dispatch(&mut self, desc: &FsrUpscaleDispatchDesc) -> Result<(), FsrError> {
        if !self.is_created || self.context.is_null() {
            return Err(FsrError::InvalidParameter);
        }

        let mut sys_desc = desc.to_sys();

        unsafe {
            let ret = fsr_sys::ffxDispatch(
                &mut self.context,
                &mut sys_desc.header as *mut fsr_sys::FfxApiHeader,
            );
            FsrError::from_return_code(ret)
        }
    }

    /// Destroy the internal context and release all GPU scratch memory.
    pub fn destroy(&mut self) {
        if self.is_created && !self.context.is_null() {
            unsafe {
                let _ = fsr_sys::ffxDestroyContext(&mut self.context, std::ptr::null());
            }
            self.context = std::ptr::null_mut();
            self.is_created = false;
        }
    }
}

impl Drop for FsrUpscaler {
    fn drop(&mut self) {
        self.destroy();
    }
}
