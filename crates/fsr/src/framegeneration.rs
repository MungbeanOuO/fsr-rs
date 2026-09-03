use crate::resource::FsrResource;
use crate::types::{
    Dimensions2D, FloatCoords2D, FrameGenerationDispatchFlags, FrameGenerationInitializationFlags,
    FsrError, ResourceFormat,
};
use crate::upscaler::{Dx12BackendConfig, FsrBackendConfig, VulkanBackendConfig};
use std::ffi::c_void;

/// Configuration parameters for creating an `FsrFrameGeneration` context.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FsrFrameGenerationConfig {
    /// Target presentation resolution.
    pub display_size: Dimensions2D,
    /// Maximum rendering resolution.
    pub max_render_size: Dimensions2D,
    /// Surface format for the swapchain backbuffer.
    pub back_buffer_format: ResourceFormat,
    /// Initialization flags.
    pub flags: FrameGenerationInitializationFlags,
    /// Backend graphics API configuration (Vulkan or DirectX 12).
    pub backend: FsrBackendConfig,
    /// Optional version override ID.
    pub version_override: Option<u64>,
}

impl FsrFrameGenerationConfig {
    /// Helper constructor for Vulkan backend.
    pub fn vulkan(
        display_size: impl Into<Dimensions2D>,
        max_render_size: impl Into<Dimensions2D>,
        back_buffer_format: ResourceFormat,
        flags: FrameGenerationInitializationFlags,
        backend: VulkanBackendConfig,
    ) -> Self {
        Self {
            display_size: display_size.into(),
            max_render_size: max_render_size.into(),
            back_buffer_format,
            flags,
            backend: FsrBackendConfig::Vulkan(backend),
            version_override: None,
        }
    }

    /// Helper constructor for DirectX 12 backend.
    pub fn dx12(
        display_size: impl Into<Dimensions2D>,
        max_render_size: impl Into<Dimensions2D>,
        back_buffer_format: ResourceFormat,
        flags: FrameGenerationInitializationFlags,
        backend: Dx12BackendConfig,
    ) -> Self {
        Self {
            display_size: display_size.into(),
            max_render_size: max_render_size.into(),
            back_buffer_format,
            flags,
            backend: FsrBackendConfig::Dx12(backend),
            version_override: None,
        }
    }
}

/// Parameters for preparing frame generation inputs (camera motion vectors, depth, and view vectors).
#[derive(Debug, Clone, PartialEq)]
pub struct FsrFrameGenerationPrepareDesc {
    /// Raw command buffer handle (`VkCommandBuffer` or `ID3D12GraphicsCommandList*` as `*mut c_void`).
    pub command_buffer: *mut c_void,
    /// Monotonically incrementing frame identifier (must increment by exactly 1 per frame).
    pub frame_id: u64,
    /// Dispatch flags for frame generation preparation.
    pub flags: FrameGenerationDispatchFlags,
    /// Dimensions used to render the game content.
    pub render_size: Dimensions2D,
    /// Subpixel camera jitter offset applied to this frame.
    pub jitter_offset: FloatCoords2D,
    /// Motion vector scale factor (usually `(1.0, 1.0)` or `(render_w, render_h)`).
    pub motion_vector_scale: FloatCoords2D,
    /// Depth buffer resource for current frame.
    pub depth: FsrResource,
    /// 2D motion vectors for current frame.
    pub motion_vectors: FsrResource,
    /// Frame delta time in seconds (converted to milliseconds internally).
    pub delta_time_seconds: f32,
    /// Flag indicating if the camera has moved discontinuously and internal state should reset.
    pub reset: bool,
    /// Camera near plane distance in world units.
    pub camera_near: f32,
    /// Camera far plane distance in world units (or 0.0 if using infinite far plane).
    pub camera_far: f32,
    /// Camera vertical field of view angle in radians.
    pub camera_fov_y: f32,
    /// Factor converting view space units to meters (default: 1.0).
    pub view_space_to_meters_factor: f32,
    /// Camera position in world space.
    pub camera_position: [f32; 3],
    /// Camera up normalized vector in world space.
    pub camera_up: [f32; 3],
    /// Camera right normalized vector in world space.
    pub camera_right: [f32; 3],
    /// Camera forward normalized vector in world space.
    pub camera_forward: [f32; 3],
}

impl Default for FsrFrameGenerationPrepareDesc {
    fn default() -> Self {
        Self {
            command_buffer: std::ptr::null_mut(),
            frame_id: 0,
            flags: FrameGenerationDispatchFlags::empty(),
            render_size: Dimensions2D::default(),
            jitter_offset: FloatCoords2D::default(),
            motion_vector_scale: FloatCoords2D::new(1.0, 1.0),
            depth: FsrResource::null(),
            motion_vectors: FsrResource::null(),
            delta_time_seconds: 0.0166,
            reset: false,
            camera_near: 0.1,
            camera_far: 1000.0,
            camera_fov_y: 1.047,
            view_space_to_meters_factor: 1.0,
            camera_position: [0.0; 3],
            camera_up: [0.0, 1.0, 0.0],
            camera_right: [1.0, 0.0, 0.0],
            camera_forward: [0.0, 0.0, 1.0],
        }
    }
}

impl FsrFrameGenerationPrepareDesc {
    /// Convert to the C-ABI `ffxDispatchDescFrameGenerationPrepareV2` structure.
    pub fn to_sys(&self) -> fsr_sys::FfxDispatchDescFrameGenerationPrepareV2 {
        fsr_sys::FfxDispatchDescFrameGenerationPrepareV2 {
            header: fsr_sys::ffxDispatchDescHeader {
                type_: fsr_sys::FFX_API_DISPATCH_DESC_TYPE_FRAMEGENERATION_PREPARE_V2,
                pNext: std::ptr::null_mut(),
            },
            frameID: self.frame_id,
            flags: self.flags.bits(),
            commandList: self.command_buffer,
            renderSize: self.render_size.into(),
            jitterOffset: self.jitter_offset.into(),
            motionVectorScale: self.motion_vector_scale.into(),
            frameTimeDelta: self.delta_time_seconds * 1000.0,
            reset: self.reset,
            cameraNear: self.camera_near,
            cameraFar: self.camera_far,
            cameraFovAngleVertical: self.camera_fov_y,
            viewSpaceToMetersFactor: self.view_space_to_meters_factor,
            depth: self.depth.to_sys(),
            motionVectors: self.motion_vectors.to_sys(),
            cameraPosition: self.camera_position,
            cameraUp: self.camera_up,
            cameraRight: self.camera_right,
            cameraForward: self.camera_forward,
        }
    }
}

/// Parameters for dispatching frame generation work and generating interpolated frames.
#[derive(Debug, Clone, PartialEq)]
pub struct FsrFrameGenerationDispatchDesc {
    /// Raw command buffer handle (`VkCommandBuffer` or `ID3D12GraphicsCommandList*` as `*mut c_void`).
    pub command_buffer: *mut c_void,
    /// Presentation color buffer for current frame.
    pub present_color: FsrResource,
    /// Output destination targets for generated frames (1 for each in `num_generated_frames`).
    pub outputs: [FsrResource; 4],
    /// Number of frames to generate (typically 1).
    pub num_generated_frames: u32,
    /// Reset flag for camera cuts / scene switches.
    pub reset: bool,
    /// Transfer function to convert source color to linear RGB (default: 0).
    pub backbuffer_transfer_function: u32,
    /// Min and max luminance values in nits `[min, max]`.
    pub min_max_luminance: [f32; 2],
    /// Monotonically incrementing frame identifier.
    pub frame_id: u64,
}

impl Default for FsrFrameGenerationDispatchDesc {
    fn default() -> Self {
        Self {
            command_buffer: std::ptr::null_mut(),
            present_color: FsrResource::null(),
            outputs: [FsrResource::null(); 4],
            num_generated_frames: 1,
            reset: false,
            backbuffer_transfer_function: 0,
            min_max_luminance: [0.0, 1000.0],
            frame_id: 0,
        }
    }
}

impl FsrFrameGenerationDispatchDesc {
    /// Convert to the C-ABI `ffxDispatchDescFrameGeneration` structure.
    pub fn to_sys(&self) -> fsr_sys::FfxDispatchDescFrameGeneration {
        fsr_sys::FfxDispatchDescFrameGeneration {
            header: fsr_sys::ffxDispatchDescHeader {
                type_: fsr_sys::FFX_API_DISPATCH_DESC_TYPE_FRAMEGENERATION,
                pNext: std::ptr::null_mut(),
            },
            commandList: self.command_buffer,
            presentColor: self.present_color.to_sys(),
            outputs: [
                self.outputs[0].to_sys(),
                self.outputs[1].to_sys(),
                self.outputs[2].to_sys(),
                self.outputs[3].to_sys(),
            ],
            numGeneratedFrames: self.num_generated_frames,
            reset: self.reset,
            backbufferTransferFunction: self.backbuffer_transfer_function,
            minMaxLuminance: self.min_max_luminance,
            generationRect: fsr_sys::FfxApiRect2D {
                left: 0,
                top: 0,
                width: 0,
                height: 0,
            },
            frameID: self.frame_id,
        }
    }
}

/// RAII wrapper managing an AMD FSR Frame Generation Context.
pub struct FsrFrameGeneration {
    context: fsr_sys::ffxContext,
    is_created: bool,
}

unsafe impl Send for FsrFrameGeneration {}
unsafe impl Sync for FsrFrameGeneration {}

impl FsrFrameGeneration {
    /// Create a new FSR Frame Generation RAII context with the provided configuration.
    pub fn new(config: &FsrFrameGenerationConfig) -> Result<Self, FsrError> {
        let mut backend_vk_desc = fsr_sys::FfxCreateBackendVKDesc {
            header: fsr_sys::FfxApiHeader {
                type_: fsr_sys::FFX_API_CREATE_CONTEXT_DESC_TYPE_BACKEND_VK as u64,
                pNext: std::ptr::null_mut(),
            },
            vkDevice: std::ptr::null_mut(),
            vkPhysicalDevice: std::ptr::null_mut(),
            vkInstance: std::ptr::null_mut(),
            vkGetDeviceProcAddr: 0,
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
                backend_vk_desc.vkDevice = vk.device;
                backend_vk_desc.vkPhysicalDevice = vk.physical_device;
                backend_vk_desc.vkInstance = vk.instance;
                backend_vk_desc.vkGetDeviceProcAddr = vk.get_device_proc_addr;
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

        let mut create_desc = fsr_sys::FfxCreateContextDescFrameGeneration {
            header: fsr_sys::FfxApiHeader {
                type_: fsr_sys::FFX_API_CREATE_CONTEXT_DESC_TYPE_FRAMEGENERATION,
                pNext: root_pnext,
            },
            flags: config.flags.bits(),
            displaySize: config.display_size.into(),
            maxRenderSize: config.max_render_size.into(),
            backBufferFormat: config.back_buffer_format as u32,
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

    /// Prepare frame generation inputs (camera motion vectors, depth, and optical flow metadata).
    pub fn prepare(&mut self, desc: &FsrFrameGenerationPrepareDesc) -> Result<(), FsrError> {
        if !self.is_created || self.context.is_null() {
            return Err(FsrError::InvalidParameter);
        }

        let mut sys_desc = desc.to_sys();

        unsafe {
            let ret = fsr_sys::ffxDispatch(
                &mut self.context,
                &mut sys_desc.header as *mut fsr_sys::ffxDispatchDescHeader,
            );
            FsrError::from_return_code(ret)
        }
    }

    /// Dispatch frame generation compute work and generate interpolated frames.
    pub fn dispatch(&mut self, desc: &FsrFrameGenerationDispatchDesc) -> Result<(), FsrError> {
        if !self.is_created || self.context.is_null() {
            return Err(FsrError::InvalidParameter);
        }

        let mut sys_desc = desc.to_sys();

        unsafe {
            let ret = fsr_sys::ffxDispatch(
                &mut self.context,
                &mut sys_desc.header as *mut fsr_sys::ffxDispatchDescHeader,
            );
            FsrError::from_return_code(ret)
        }
    }

    /// Destroy the frame generation context and release all scratch resources.
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

impl Drop for FsrFrameGeneration {
    fn drop(&mut self) {
        self.destroy();
    }
}
