# fsr-rs

Modern, high-performance, and **non-opinionated** Rust bindings for **AMD FidelityFX Super Resolution (FSR 2.x / 3.x / 4.x & Modern `ffx_api`)**.

---

## 🌟 Key Features

- **Modern `ffx_api` Standard**:
  - Aligned with AMD's modern C-ABI architecture (`ffxCreateContext`, `ffxDestroyContext`, `ffxDispatch`, `ffxQuery`, `ffxConfigure`).
  - Supports extensible descriptor chaining (`ffxApiHeader.pNext`).
- **Zero Ecosystem Lock-in**:
  - Pure, transparent C-ABI primitive handles (`u64`, `*mut c_void`, `u32`, `f32`).
  - Seamlessly integrates with any graphics library (`ash`, `wgpu-hal`, `vulkano`, `windows`, or custom raw FFI).
- **Dual-Backend Support (Vulkan & DirectX 12)**:
  - **Vulkan (Cross-Platform)**: Supports FSR 3.1.5 (Upscaler) with 10 precompiled SPIR-V passes and FSR 3.1.6 (Frame Generation).
  - **DirectX 12 (Windows)**: Supports dynamic version overrides to invoke official signed providers, including **FSR 4.1.1 (ML Neural Network Upscaling)**.
- **Decoupled Architecture**:
  - Independent **`FsrUpscaler`** and **`FsrFrameGeneration`** contexts.
- **RenderGraph & Memory Aliasing**:
  - Implements the `FsrAllocator` trait to redirect scratch resource allocations into host memory pools.
- **Pure Math Helpers**:
  - Built-in Halton sequence camera jitter, Mipmap LOD bias, and jitter phase count calculators.

---

## 📦 Workspace Structure

- **`crates/fsr-sys`**: Unsafe 1:1 C-ABI FFI bindings generated via `bindgen`, including precompiled Vulkan SPIR-V compute shaders.
- **`crates/fsr`**: Safe, ergonomic Rust wrapper with RAII contexts, bitflags, builders, and math utilities.

---

## 🚀 Quick Start

Add `fsr` to your `Cargo.toml`:

```toml
[dependencies]
fsr = { git = "https://github.com/mungbeanouo/fsr-rs.git", features = ["vulkan"] }
```

### 1. Temporal Upscaler (`FsrUpscaler`)

```rust,no_run
use fsr::{
    Dimensions2D, FloatCoords2D, FsrBackendConfig, FsrResource, FsrUpscaleDispatchDesc,
    FsrUpscaler, FsrUpscalerConfig, InitializationFlags, QualityMode, ResourceFormat,
    ResourceState, VulkanBackendConfig,
};
use std::ffi::c_void;

fn run_upscaler(
    device: *mut c_void,
    physical_device: *mut c_void,
    instance: *mut c_void,
    proc_addr: usize,
    cmd_buffer: *mut c_void,
    color_tex: *mut c_void,
    depth_tex: *mut c_void,
    motion_vectors_tex: *mut c_void,
    output_tex: *mut c_void,
) -> Result<(), fsr::FsrError> {
    // 1. Calculate recommended render resolution from target resolution
    let (target_w, target_h) = (3840, 2160);
    let (render_w, render_h) = QualityMode::Quality.render_resolution(target_w, target_h);

    // 2. Configure and create upscaler context
    let config = FsrUpscalerConfig::vulkan(
        (render_w, render_h),
        (target_w, target_h),
        InitializationFlags::HIGH_DYNAMIC_RANGE | InitializationFlags::AUTO_EXPOSURE,
        VulkanBackendConfig::new(device, physical_device, instance, proc_addr),
    );
    let mut upscaler = FsrUpscaler::new(&config)?;

    // 3. Prepare frame resource wrappers
    let color = FsrResource::texture2d(color_tex, render_w, render_h, ResourceFormat::R16G16B16A16Float, ResourceState::COMPUTE_READ);
    let depth = FsrResource::texture2d(depth_tex, render_w, render_h, ResourceFormat::R32Float, ResourceState::COMPUTE_READ);
    let motion_vectors = FsrResource::texture2d(motion_vectors_tex, render_w, render_h, ResourceFormat::R16G16Float, ResourceState::COMPUTE_READ);
    let output = FsrResource::texture2d(output_tex, target_w, target_h, ResourceFormat::R16G16B16A16Float, ResourceState::UNORDERED_ACCESS);

    // 4. Record upscaling dispatch commands
    let dispatch_desc = FsrUpscaleDispatchDesc {
        command_buffer: cmd_buffer,
        color,
        depth,
        motion_vectors,
        output,
        exposure: None,
        reactive_mask: None,
        transparency_and_composition_mask: None,
        jitter_offset: FloatCoords2D::new(0.125, -0.25),
        motion_vector_scale: FloatCoords2D::new(render_w as f32, render_h as f32),
        render_size: Dimensions2D::new(render_w, render_h),
        upscale_size: Dimensions2D::new(target_w, target_h),
        enable_sharpening: true,
        sharpness: 0.8,
        reset_accumulation: false,
        camera_near: 0.1,
        camera_far: 1000.0,
        camera_fov_y: 1.047,
        view_space_to_meters_factor: 1.0,
        delta_time_seconds: 0.0166,
        pre_exposure: 1.0,
        flags: Default::default(),
    };

    upscaler.dispatch(&dispatch_desc)?;
    Ok(())
}
```

---

### 2. DirectX 12 & FSR 4 (ML) Version Override

On Windows with DirectX 12, query available provider versions and override the version ID on context creation to enable FSR 4:

```rust,no_run
use fsr::{
    Dx12BackendConfig, FsrBackendConfig, FsrUpscaler, FsrUpscalerConfig,
    InitializationFlags,
};
use std::ffi::c_void;

fn run_fsr4_dx12(d3d12_device: *mut c_void) -> Result<(), fsr::FsrError> {
    let backend = FsrBackendConfig::Dx12(Dx12BackendConfig::new(d3d12_device));

    // Enumerate supported versions (e.g. FSR 2.3.4, FSR 3.1.5, FSR 4.1.1 ML)
    let available_versions = FsrUpscaler::query_available_versions(&backend)?;
    let fsr4_id = available_versions.iter().find(|(_, name)| name.contains("4.")).map(|(id, _)| *id);

    let mut config = FsrUpscalerConfig::dx12(
        (1920, 1080),
        (3840, 2160),
        InitializationFlags::HIGH_DYNAMIC_RANGE,
        Dx12BackendConfig::new(d3d12_device),
    );
    config.version_override = fsr4_id; // Activate FSR 4 ML Neural Network pipeline

    let mut upscaler = FsrUpscaler::new(&config)?;
    Ok(())
}
```

---

### 3. Frame Generation (`FsrFrameGeneration`)

```rust,no_run
use fsr::{
    Dimensions2D, FloatCoords2D, FrameGenerationDispatchFlags,
    FrameGenerationInitializationFlags, FsrFrameGeneration, FsrFrameGenerationConfig,
    FsrFrameGenerationDispatchDesc, FsrFrameGenerationPrepareDesc, FsrResource,
    ResourceFormat, ResourceState, VulkanBackendConfig,
};
use std::ffi::c_void;

fn run_frame_generation(
    device: *mut c_void,
    physical_device: *mut c_void,
    instance: *mut c_void,
    proc_addr: usize,
    cmd_buffer: *mut c_void,
    present_color_tex: *mut c_void,
    depth_tex: *mut c_void,
    motion_vectors_tex: *mut c_void,
    interpolated_output_tex: *mut c_void,
) -> Result<(), fsr::FsrError> {
    let config = FsrFrameGenerationConfig::vulkan(
        (3840, 2160),
        (1920, 1080),
        ResourceFormat::R16G16B16A16Float,
        FrameGenerationInitializationFlags::ASYNC_WORKLOAD_SUPPORT,
        VulkanBackendConfig::new(device, physical_device, instance, proc_addr),
    );
    let mut frame_gen = FsrFrameGeneration::new(&config)?;

    // 1. Prepare inputs (camera motion vectors, depth, and view vectors)
    let depth = FsrResource::texture2d(depth_tex, 1920, 1080, ResourceFormat::R32Float, ResourceState::COMPUTE_READ);
    let motion_vectors = FsrResource::texture2d(motion_vectors_tex, 1920, 1080, ResourceFormat::R16G16Float, ResourceState::COMPUTE_READ);

    let prepare_desc = FsrFrameGenerationPrepareDesc {
        command_buffer: cmd_buffer,
        frame_id: 100,
        flags: FrameGenerationDispatchFlags::empty(),
        render_size: Dimensions2D::new(1920, 1080),
        jitter_offset: FloatCoords2D::new(0.0, 0.0),
        motion_vector_scale: FloatCoords2D::new(1.0, 1.0),
        depth,
        motion_vectors,
        delta_time_seconds: 0.0166,
        reset: false,
        camera_near: 0.1,
        camera_far: 1000.0,
        camera_fov_y: 1.047,
        view_space_to_meters_factor: 1.0,
        camera_position: [0.0, 1.0, -5.0],
        camera_up: [0.0, 1.0, 0.0],
        camera_right: [1.0, 0.0, 0.0],
        camera_forward: [0.0, 0.0, 1.0],
    };
    frame_gen.prepare(&prepare_desc)?;

    // 2. Dispatch frame interpolation
    let present_color = FsrResource::texture2d(present_color_tex, 3840, 2160, ResourceFormat::R16G16B16A16Float, ResourceState::COMPUTE_READ);
    let output = FsrResource::texture2d(interpolated_output_tex, 3840, 2160, ResourceFormat::R16G16B16A16Float, ResourceState::UNORDERED_ACCESS);

    let dispatch_desc = FsrFrameGenerationDispatchDesc {
        command_buffer: cmd_buffer,
        present_color,
        outputs: [output, FsrResource::null(), FsrResource::null(), FsrResource::null()],
        num_generated_frames: 1,
        reset: false,
        backbuffer_transfer_function: 0,
        min_max_luminance: [0.0, 1000.0],
        frame_id: 100,
    };
    frame_gen.dispatch(&dispatch_desc)?;

    Ok(())
}
```

---

### 4. Camera Jitter & Mipmap LOD Bias

```rust
use fsr::math::{calculate_mip_lod_bias, halton, jitter_phase_count};

let render_w = 1920;
let target_w = 3840;

// Calculate Halton jitter offset for current frame index
let phase_count = jitter_phase_count(render_w, target_w);
let frame_index = 42 % phase_count;
let jitter_x = halton(frame_index + 1, 2) - 0.5;
let jitter_y = halton(frame_index + 1, 3) - 0.5;

// Texture sampler Mipmap LOD Bias
let mip_lod_bias = calculate_mip_lod_bias(render_w, target_w);
assert_eq!(mip_lod_bias, -2.0); // 2x upscale -> log2(1920 / 3840) - 1.0 = -2.0
```

---

## 🤝 Acknowledgements

- Developed with pair-programming assistance from **Antigravity**.
- AMD FidelityFX SDK is a trademark of Advanced Micro Devices, Inc.

---

## 📄 License

- Rust code and SPIR-V pipeline in this repository are licensed under either of [Apache License 2.0](LICENSE-APACHE) or [MIT License](LICENSE-MIT) at your option.
- AMD FidelityFX SDK shaders and headers are copyrighted by Advanced Micro Devices, Inc. and licensed under the [MIT License](https://opensource.org/licenses/MIT).
