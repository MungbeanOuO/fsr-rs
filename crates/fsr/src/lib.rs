//! Modern, safe, zero-dependency Rust bindings for AMD FidelityFX Super Resolution (FSR).
//!
//! Aligned with AMD FSR SDK 2.4.0 and the modern `ffx_api` architecture standard.

#![allow(clippy::unnecessary_cast)]

pub use fsr_sys as sys;

pub mod allocator;
pub mod framegeneration;
pub mod resource;
pub mod types;
pub mod upscaler;

pub use allocator::{FsrAllocatedResource, FsrAllocator, FsrResourceRequest};
pub use framegeneration::{
    FsrFrameGeneration, FsrFrameGenerationConfig, FsrFrameGenerationDispatchDesc,
    FsrFrameGenerationPrepareDesc,
};
pub use resource::FsrResource;
pub use types::{
    Dimensions2D, DispatchFlags, FloatCoords2D, FrameGenerationDispatchFlags,
    FrameGenerationInitializationFlags, FsrError, InitializationFlags, QualityMode, ResourceFormat,
    ResourceState, UiCompositionFlags,
};
pub use upscaler::{
    Dx12BackendConfig, FsrBackendConfig, FsrUpscaleDispatchDesc, FsrUpscaler, FsrUpscalerConfig,
    VulkanBackendConfig,
};

/// Pure math helpers for camera jitter offset and texture MIP LOD bias calculations.
pub mod math {
    /// Calculate the Halton sequence value for camera jittering.
    #[inline]
    pub fn halton(index: u32, base: u32) -> f32 {
        let mut result = 0.0f32;
        let mut f = 1.0f32 / base as f32;
        let mut i = index;
        while i > 0 {
            result += f * (i % base) as f32;
            i /= base;
            f /= base as f32;
        }
        result
    }

    /// Calculate the recommended jitter phase count for a given render resolution and target resolution.
    #[inline]
    pub fn jitter_phase_count(render_width: u32, target_width: u32) -> u32 {
        let ratio = target_width as f32 / render_width.max(1) as f32;
        (8.0 * ratio * ratio).round() as u32
    }

    /// Calculate Mipmap LOD Bias for texture sampling when upscaling.
    ///
    /// Formula: `log2(render_width / target_width) - 1.0`
    #[inline]
    pub fn calculate_mip_lod_bias(render_width: u32, target_width: u32) -> f32 {
        let scale = render_width as f32 / target_width.max(1) as f32;
        scale.log2() - 1.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quality_modes_and_resolutions() {
        assert_eq!(QualityMode::NativeAA.upscale_ratio(), 1.0);
        assert_eq!(QualityMode::Quality.upscale_ratio(), 1.5);
        assert_eq!(QualityMode::Balanced.upscale_ratio(), 1.7);
        assert_eq!(QualityMode::Performance.upscale_ratio(), 2.0);
        assert_eq!(QualityMode::UltraPerformance.upscale_ratio(), 3.0);

        // 4K target: 3840x2160
        let (q_w, q_h) = QualityMode::Quality.render_resolution(3840, 2160);
        assert_eq!(q_w, 2560); // 3840 / 1.5 = 2560
        assert_eq!(q_h, 1440); // 2160 / 1.5 = 1440

        let (p_w, p_h) = QualityMode::Performance.render_resolution(3840, 2160);
        assert_eq!(p_w, 1920); // 3840 / 2.0 = 1920
        assert_eq!(p_h, 1080); // 2160 / 2.0 = 1080
    }

    #[test]
    fn test_initialization_flags_bitflags() {
        let flags = InitializationFlags::HIGH_DYNAMIC_RANGE
            | InitializationFlags::AUTO_EXPOSURE
            | InitializationFlags::DEPTH_INVERTED;

        assert!(flags.contains(InitializationFlags::HIGH_DYNAMIC_RANGE));
        assert!(flags.contains(InitializationFlags::AUTO_EXPOSURE));
        assert!(flags.contains(InitializationFlags::DEPTH_INVERTED));
        assert!(!flags.contains(InitializationFlags::DEPTH_INFINITE));
    }

    #[test]
    fn test_frame_generation_flags_bitflags() {
        let flags = FrameGenerationInitializationFlags::ASYNC_WORKLOAD_SUPPORT
            | FrameGenerationInitializationFlags::HIGH_DYNAMIC_RANGE
            | FrameGenerationInitializationFlags::DEPTH_INVERTED;

        assert!(flags.contains(FrameGenerationInitializationFlags::ASYNC_WORKLOAD_SUPPORT));
        assert!(flags.contains(FrameGenerationInitializationFlags::HIGH_DYNAMIC_RANGE));
        assert!(flags.contains(FrameGenerationInitializationFlags::DEPTH_INVERTED));
        assert!(!flags.contains(FrameGenerationInitializationFlags::DEPTH_INFINITE));

        let dispatch_flags = FrameGenerationDispatchFlags::DRAW_DEBUG_VIEW
            | FrameGenerationDispatchFlags::DRAW_DEBUG_TEAR_LINES;
        assert!(dispatch_flags.contains(FrameGenerationDispatchFlags::DRAW_DEBUG_VIEW));
        assert!(dispatch_flags.contains(FrameGenerationDispatchFlags::DRAW_DEBUG_TEAR_LINES));
    }

    #[test]
    fn test_resource_to_sys_conversion() {
        let dummy_handle = 0x12345678 as *mut std::ffi::c_void;
        let res = FsrResource::texture2d(
            dummy_handle,
            1920,
            1080,
            ResourceFormat::R16G16B16A16Float,
            ResourceState::COMPUTE_READ,
        );

        let sys_res = res.to_sys();
        assert_eq!(sys_res.resource, dummy_handle);
        assert_eq!(
            sys_res.state,
            fsr_sys::FFX_API_RESOURCE_STATE_COMPUTE_READ as _
        );
        unsafe {
            assert_eq!(sys_res.description.__bindgen_anon_1.width, 1920);
            assert_eq!(sys_res.description.__bindgen_anon_2.height, 1080);
        }
    }

    #[test]
    fn test_dispatch_desc_builder() {
        let dummy_cmd = 0xABCDEF as *mut std::ffi::c_void;
        let dummy_res = FsrResource::texture2d(
            0x1000 as *mut std::ffi::c_void,
            1920,
            1080,
            ResourceFormat::R16G16B16A16Float,
            ResourceState::COMPUTE_READ,
        );

        let dispatch = FsrUpscaleDispatchDesc {
            command_buffer: dummy_cmd,
            color: dummy_res,
            depth: dummy_res,
            motion_vectors: dummy_res,
            output: dummy_res,
            exposure: None,
            reactive_mask: None,
            transparency_and_composition_mask: None,
            jitter_offset: FloatCoords2D::new(0.125, -0.25),
            motion_vector_scale: FloatCoords2D::new(1920.0, 1080.0),
            render_size: Dimensions2D::new(1920, 1080),
            upscale_size: Dimensions2D::new(3840, 2160),
            enable_sharpening: true,
            sharpness: 0.8,
            reset_accumulation: true,
            camera_near: 0.1,
            camera_far: 1000.0,
            camera_fov_y: 1.047, // 60 deg
            view_space_to_meters_factor: 1.0,
            delta_time_seconds: 0.0166,
            pre_exposure: 1.0,
            flags: DispatchFlags::DRAW_DEBUG_VIEW,
        };

        let sys_desc = dispatch.to_sys();
        assert_eq!(sys_desc.commandList, dummy_cmd);
        assert_eq!(sys_desc.jitterOffset.x, 0.125);
        assert_eq!(sys_desc.jitterOffset.y, -0.25);
        assert_eq!(sys_desc.renderSize.width, 1920);
        assert_eq!(sys_desc.upscaleSize.width, 3840);
        assert!(sys_desc.enableSharpening);
        assert_eq!(sys_desc.sharpness, 0.8);
        assert!((sys_desc.frameTimeDelta - 16.6).abs() < 1e-3);
        assert!(sys_desc.reset);
    }

    #[test]
    fn test_frame_generation_descriptors() {
        let dummy_cmd = 0xABCDEF as *mut std::ffi::c_void;
        let dummy_res = FsrResource::texture2d(
            0x1000 as *mut std::ffi::c_void,
            1920,
            1080,
            ResourceFormat::R16G16B16A16Float,
            ResourceState::COMPUTE_READ,
        );

        // Prepare descriptor
        let prepare_desc = FsrFrameGenerationPrepareDesc {
            command_buffer: dummy_cmd,
            frame_id: 42,
            flags: FrameGenerationDispatchFlags::DRAW_DEBUG_VIEW,
            render_size: Dimensions2D::new(1920, 1080),
            jitter_offset: FloatCoords2D::new(0.5, -0.5),
            motion_vector_scale: FloatCoords2D::new(1.0, 1.0),
            depth: dummy_res,
            motion_vectors: dummy_res,
            delta_time_seconds: 0.0166,
            reset: false,
            camera_near: 0.1,
            camera_far: 1000.0,
            camera_fov_y: 1.047,
            view_space_to_meters_factor: 1.0,
            camera_position: [1.0, 2.0, 3.0],
            camera_up: [0.0, 1.0, 0.0],
            camera_right: [1.0, 0.0, 0.0],
            camera_forward: [0.0, 0.0, 1.0],
        };

        let sys_prepare = prepare_desc.to_sys();
        assert_eq!(sys_prepare.frameID, 42);
        assert_eq!(sys_prepare.commandList, dummy_cmd);
        assert_eq!(sys_prepare.cameraPosition, [1.0, 2.0, 3.0]);
        assert_eq!(sys_prepare.cameraForward, [0.0, 0.0, 1.0]);

        // Dispatch descriptor
        let dispatch_desc = FsrFrameGenerationDispatchDesc {
            command_buffer: dummy_cmd,
            present_color: dummy_res,
            outputs: [
                dummy_res,
                FsrResource::null(),
                FsrResource::null(),
                FsrResource::null(),
            ],
            num_generated_frames: 1,
            reset: false,
            backbuffer_transfer_function: 0,
            min_max_luminance: [0.0, 1000.0],
            frame_id: 42,
        };

        let sys_dispatch = dispatch_desc.to_sys();
        assert_eq!(sys_dispatch.frameID, 42);
        assert_eq!(sys_dispatch.numGeneratedFrames, 1);
        assert_eq!(sys_dispatch.presentColor.resource, dummy_res.resource);
    }

    #[test]
    fn test_quality_mode_all_and_labels() {
        assert_eq!(QualityMode::ALL.len(), 5);
        assert_eq!(QualityMode::NativeAA.label(), "Native AA (1.0x)");
        assert_eq!(QualityMode::Quality.label(), "Quality (1.5x - Recommended)");
        assert_eq!(QualityMode::Balanced.label(), "Balanced (1.7x)");
        assert_eq!(QualityMode::Performance.label(), "Performance (2.0x)");
        assert_eq!(
            QualityMode::UltraPerformance.label(),
            "Ultra Performance (3.0x)"
        );

        // Test Display implementation
        assert_eq!(
            format!("{}", QualityMode::Quality),
            "Quality (1.5x - Recommended)"
        );
    }

    #[test]
    fn test_backend_configurations_and_version_override() {
        let dummy_device = 0x5555 as *mut std::ffi::c_void;

        // Test DX12 backend config
        let dx_cfg = Dx12BackendConfig::new(dummy_device);
        let backend_dx: FsrBackendConfig = dx_cfg.into();
        assert_eq!(backend_dx, FsrBackendConfig::Dx12(dx_cfg));

        // Test Vulkan backend config
        let vk_cfg = VulkanBackendConfig::new(dummy_device, dummy_device, dummy_device, 0x1234);
        let backend_vk: FsrBackendConfig = vk_cfg.into();
        assert_eq!(backend_vk, FsrBackendConfig::Vulkan(vk_cfg));

        // Test FsrUpscalerConfig builder helpers
        let config_dx = FsrUpscalerConfig::dx12(
            (1920, 1080),
            (3840, 2160),
            InitializationFlags::HIGH_DYNAMIC_RANGE,
            dx_cfg,
        );
        assert_eq!(config_dx.backend, FsrBackendConfig::Dx12(dx_cfg));
        assert_eq!(config_dx.max_render_size.width, 1920);
        assert_eq!(config_dx.max_upscale_size.width, 3840);
        assert!(config_dx.version_override.is_none());

        let mut config_with_fsr4 = config_dx;
        config_with_fsr4.version_override = Some(40102); // FSR 4.1.2 version override
        assert_eq!(config_with_fsr4.version_override, Some(40102));

        // Test FsrFrameGenerationConfig builder helpers
        let fg_config_vk = FsrFrameGenerationConfig::vulkan(
            (3840, 2160),
            (1920, 1080),
            ResourceFormat::R16G16B16A16Float,
            FrameGenerationInitializationFlags::ASYNC_WORKLOAD_SUPPORT,
            vk_cfg,
        );
        assert_eq!(fg_config_vk.backend, FsrBackendConfig::Vulkan(vk_cfg));
        assert_eq!(fg_config_vk.display_size.width, 3840);
    }

    #[test]
    fn test_math_functions() {
        // Halton sequence tests
        let h2_1 = math::halton(1, 2);
        assert!((h2_1 - 0.5).abs() < 1e-6);
        let h2_2 = math::halton(2, 2);
        assert!((h2_2 - 0.25).abs() < 1e-6);

        // Mip LOD bias
        let bias_2x = math::calculate_mip_lod_bias(1920, 3840);
        assert!((bias_2x - (-2.0)).abs() < 1e-5);

        // Jitter phase count for 2x upscale: 8 * 2.0^2 = 32
        let phase = math::jitter_phase_count(1920, 3840);
        assert_eq!(phase, 32);
    }

    #[test]
    fn test_custom_allocator_trait() {
        struct MockAllocator {
            allocated_count: usize,
        }

        impl FsrAllocator for MockAllocator {
            fn allocate_resource(
                &mut self,
                _request: &FsrResourceRequest,
            ) -> Result<FsrAllocatedResource, FsrError> {
                self.allocated_count += 1;
                Ok(FsrAllocatedResource {
                    resource: 0x1111 as *mut std::ffi::c_void,
                    device_memory: 0x2222 as *mut std::ffi::c_void,
                })
            }

            fn deallocate_resource(
                &mut self,
                _effect_id: u32,
                _resource: *mut std::ffi::c_void,
                _device_memory: *mut std::ffi::c_void,
            ) -> Result<(), FsrError> {
                self.allocated_count -= 1;
                Ok(())
            }
        }

        let mut alloc = MockAllocator { allocated_count: 0 };
        let req = FsrResourceRequest {
            effect_id: 1,
            width: 1920,
            height: 1080,
            depth: 1,
            mip_count: 1,
            format: ResourceFormat::R16G16B16A16Float,
            usage: 0,
        };

        let allocated = alloc.allocate_resource(&req).unwrap();
        assert_eq!(alloc.allocated_count, 1);
        assert_eq!(allocated.resource, 0x1111 as *mut std::ffi::c_void);

        alloc
            .deallocate_resource(1, allocated.resource, allocated.device_memory)
            .unwrap();
        assert_eq!(alloc.allocated_count, 0);
    }
}
