#![allow(clippy::unnecessary_cast)]

use std::fmt;

/// FSR error codes mapped to standard Rust Result errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FsrError {
    /// Generic failure or invalid return code.
    Failed,
    /// Invalid pointer or descriptor header structure.
    InvalidPointer,
    /// Invalid argument or configuration flag combination.
    InvalidParameter,
    /// Direct3D 12 or Vulkan runtime returned an internal error.
    DriverError,
    /// Required native library (e.g. `amd_fidelityfx_dx12.dll` or loader) not found.
    LibraryNotFound,
    /// Unknown return code from FFX API.
    Unknown(u32),
}

impl std::error::Error for FsrError {}

impl fmt::Display for FsrError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Failed => write!(f, "FSR operation failed"),
            Self::InvalidPointer => write!(f, "Invalid pointer or descriptor structure passed to FSR"),
            Self::InvalidParameter => write!(f, "Invalid parameter or configuration in FSR"),
            Self::DriverError => write!(f, "GPU driver error occurred during FSR operation"),
            Self::LibraryNotFound => write!(f, "Required FidelityFX dynamic library was not found"),
            Self::Unknown(code) => write!(f, "Unknown FidelityFX return code: {code}"),
        }
    }
}

impl FsrError {
    /// Convert raw C-ABI `ffxReturnCode_t` into a Rust `Result<(), FsrError>`.
    pub fn from_return_code(code: fsr_sys::ffxReturnCode_t) -> Result<(), Self> {
        let code_i32 = code as i32;
        if code_i32 == fsr_sys::FFX_API_RETURN_OK as i32 {
            Ok(())
        } else if code_i32 == fsr_sys::FFX_API_RETURN_ERROR_PARAMETER as i32 {
            Err(Self::InvalidParameter)
        } else if code_i32 == fsr_sys::FFX_API_RETURN_ERROR_RUNTIME_ERROR as i32 {
            Err(Self::DriverError)
        } else if code_i32 == fsr_sys::FFX_API_RETURN_NO_PROVIDER as i32 {
            Err(Self::LibraryNotFound)
        } else if code_i32 == fsr_sys::FFX_API_RETURN_ERROR as i32 {
            Err(Self::Failed)
        } else {
            Err(Self::Unknown(code as u32))
        }
    }
}

/// Standard quality presets for AMD FidelityFX Super Resolution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum QualityMode {
    /// Native AA (1.0x scale): No upscaling; applies temporal antialiasing at native resolution.
    NativeAA,
    /// Quality (1.5x scale): High visual fidelity; recommended default mode.
    #[default]
    Quality,
    /// Balanced (1.7x scale): Balance between performance and image sharpness.
    Balanced,
    /// Performance (2.0x scale): High frame rate mode with 2.0x linear scaling factor.
    Performance,
    /// Ultra Performance (3.0x scale): Maximum performance mode for 4K / 8K rendering targets.
    UltraPerformance,
}

impl fmt::Display for QualityMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label())
    }
}

impl QualityMode {
    /// An array containing all available QualityMode presets.
    pub const ALL: [QualityMode; 5] = [
        QualityMode::NativeAA,
        QualityMode::Quality,
        QualityMode::Balanced,
        QualityMode::Performance,
        QualityMode::UltraPerformance,
    ];

    /// Returns a human-readable English label with scaling ratio.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::NativeAA => "Native AA (1.0x)",
            Self::Quality => "Quality (1.5x - Recommended)",
            Self::Balanced => "Balanced (1.7x)",
            Self::Performance => "Performance (2.0x)",
            Self::UltraPerformance => "Ultra Performance (3.0x)",
        }
    }

    /// Linear downscale ratio for internal rendering.
    #[must_use]
    pub const fn upscale_ratio(self) -> f32 {
        match self {
            Self::NativeAA => 1.0,
            Self::Quality => 1.5,
            Self::Balanced => 1.7,
            Self::Performance => 2.0,
            Self::UltraPerformance => 3.0,
        }
    }

    /// Calculate internal rendering resolution `(render_width, render_height)` based on display resolution.
    #[must_use]
    pub fn render_resolution(self, display_width: u32, display_height: u32) -> (u32, u32) {
        let ratio = self.upscale_ratio();
        let render_w = ((display_width as f32) / ratio).round() as u32;
        let render_h = ((display_height as f32) / ratio).round() as u32;
        (render_w.max(1), render_h.max(1))
    }
}

bitflags::bitflags! {
    /// Configuration flags for FSR Upscaler context creation.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct InitializationFlags: u32 {
        /// Input color data is using high dynamic range (HDR).
        const HIGH_DYNAMIC_RANGE = fsr_sys::FFX_UPSCALE_ENABLE_HIGH_DYNAMIC_RANGE as u32;
        /// Motion vectors are rendered at display resolution instead of render resolution.
        const DISPLAY_RESOLUTION_MOTION_VECTORS = fsr_sys::FFX_UPSCALE_ENABLE_DISPLAY_RESOLUTION_MOTION_VECTORS as u32;
        /// Motion vectors already contain jitter cancellation.
        const MOTION_VECTORS_JITTER_CANCELLATION = fsr_sys::FFX_UPSCALE_ENABLE_MOTION_VECTORS_JITTER_CANCELLATION as u32;
        /// Input depth buffer is inverted [1.0 .. 0.0].
        const DEPTH_INVERTED = fsr_sys::FFX_UPSCALE_ENABLE_DEPTH_INVERTED as u32;
        /// Camera uses an infinite far plane.
        const DEPTH_INFINITE = fsr_sys::FFX_UPSCALE_ENABLE_DEPTH_INFINITE as u32;
        /// Apply automatic exposure calculation to input colors.
        const AUTO_EXPOSURE = fsr_sys::FFX_UPSCALE_ENABLE_AUTO_EXPOSURE as u32;
        /// Application uses dynamic resolution scaling.
        const DYNAMIC_RESOLUTION = fsr_sys::FFX_UPSCALE_ENABLE_DYNAMIC_RESOLUTION as u32;
        /// Runtime should validate API values and report debug warnings.
        const DEBUG_CHECKING = fsr_sys::FFX_UPSCALE_ENABLE_DEBUG_CHECKING as u32;
        /// Color resource contains perceptual (gamma corrected) colors.
        const NON_LINEAR_COLORSPACE = fsr_sys::FFX_UPSCALE_ENABLE_NON_LINEAR_COLORSPACE as u32;
        /// Enable debug visualization views.
        const DEBUG_VISUALIZATION = fsr_sys::FFX_UPSCALE_ENABLE_DEBUG_VISUALIZATION as u32;
    }
}

bitflags::bitflags! {
    /// Flags for per-frame FSR upscaler dispatch.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct DispatchFlags: u32 {
        /// Draw debug visualization information into output resource.
        const DRAW_DEBUG_VIEW = fsr_sys::FFX_UPSCALE_FLAG_DRAW_DEBUG_VIEW as u32;
        /// Input color resource contains perceptual sRGB colors.
        const NON_LINEAR_COLOR_SRGB = fsr_sys::FFX_UPSCALE_FLAG_NON_LINEAR_COLOR_SRGB as u32;
        /// Input color resource contains perceptual PQ colors.
        const NON_LINEAR_COLOR_PQ = fsr_sys::FFX_UPSCALE_FLAG_NON_LINEAR_COLOR_PQ as u32;
    }
}

bitflags::bitflags! {
    /// Configuration flags for FSR Frame Generation context creation.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct FrameGenerationInitializationFlags: u32 {
        /// Support async compute workloads on async compute queues.
        const ASYNC_WORKLOAD_SUPPORT = fsr_sys::FFX_FRAMEGENERATION_ENABLE_ASYNC_WORKLOAD_SUPPORT as u32;
        /// Motion vectors are rendered at display resolution instead of render resolution.
        const DISPLAY_RESOLUTION_MOTION_VECTORS = fsr_sys::FFX_FRAMEGENERATION_ENABLE_DISPLAY_RESOLUTION_MOTION_VECTORS as u32;
        /// Motion vectors already contain jitter pattern.
        const MOTION_VECTORS_JITTER_CANCELLATION = fsr_sys::FFX_FRAMEGENERATION_ENABLE_MOTION_VECTORS_JITTER_CANCELLATION as u32;
        /// Input depth buffer is inverted [1.0 .. 0.0].
        const DEPTH_INVERTED = fsr_sys::FFX_FRAMEGENERATION_ENABLE_DEPTH_INVERTED as u32;
        /// Camera uses an infinite far plane.
        const DEPTH_INFINITE = fsr_sys::FFX_FRAMEGENERATION_ENABLE_DEPTH_INFINITE as u32;
        /// Input color data is using high dynamic range (HDR).
        const HIGH_DYNAMIC_RANGE = fsr_sys::FFX_FRAMEGENERATION_ENABLE_HIGH_DYNAMIC_RANGE as u32;
        /// Validate API values and report debug issues.
        const DEBUG_CHECKING = fsr_sys::FFX_FRAMEGENERATION_ENABLE_DEBUG_CHECKING as u32;
    }
}

bitflags::bitflags! {
    /// Flags for per-frame FSR frame generation dispatch and preparation.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct FrameGenerationDispatchFlags: u32 {
        /// Draw debug tear lines to generated output.
        const DRAW_DEBUG_TEAR_LINES = fsr_sys::FFX_FRAMEGENERATION_FLAG_DRAW_DEBUG_TEAR_LINES as u32;
        /// Draw debug reset indicators to generated output.
        const DRAW_DEBUG_RESET_INDICATORS = fsr_sys::FFX_FRAMEGENERATION_FLAG_DRAW_DEBUG_RESET_INDICATORS as u32;
        /// Draw debug views with relevant information.
        const DRAW_DEBUG_VIEW = fsr_sys::FFX_FRAMEGENERATION_FLAG_DRAW_DEBUG_VIEW as u32;
        /// Only run frame interpolation without modifying the swapchain.
        const NO_SWAPCHAIN_CONTEXT_NOTIFY = fsr_sys::FFX_FRAMEGENERATION_FLAG_NO_SWAPCHAIN_CONTEXT_NOTIFY as u32;
        /// Draw debug pacing lines to generated output.
        const DRAW_DEBUG_PACING_LINES = fsr_sys::FFX_FRAMEGENERATION_FLAG_DRAW_DEBUG_PACING_LINES as u32;
    }
}

bitflags::bitflags! {
    /// Flags for UI composition in frame generation.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct UiCompositionFlags: u32 {
        /// Use premultiplied alpha for UI composition.
        const USE_PREMUL_ALPHA = fsr_sys::FFX_FRAMEGENERATION_UI_COMPOSITION_FLAG_USE_PREMUL_ALPHA as u32;
        /// Double buffer UI resource in swapchain.
        const ENABLE_INTERNAL_UI_DOUBLE_BUFFERING = fsr_sys::FFX_FRAMEGENERATION_UI_COMPOSITION_FLAG_ENABLE_INTERNAL_UI_DOUBLE_BUFFERING as u32;
    }
}

bitflags::bitflags! {
    /// Explicit state of a GPU resource when passed to FSR.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct ResourceState: u32 {
        const COMMON = fsr_sys::FFX_API_RESOURCE_STATE_COMMON as u32;
        const UNORDERED_ACCESS = fsr_sys::FFX_API_RESOURCE_STATE_UNORDERED_ACCESS as u32;
        const COMPUTE_READ = fsr_sys::FFX_API_RESOURCE_STATE_COMPUTE_READ as u32;
        const PIXEL_READ = fsr_sys::FFX_API_RESOURCE_STATE_PIXEL_READ as u32;
        const PIXEL_COMPUTE_READ = fsr_sys::FFX_API_RESOURCE_STATE_PIXEL_COMPUTE_READ as u32;
        const COPY_SRC = fsr_sys::FFX_API_RESOURCE_STATE_COPY_SRC as u32;
        const COPY_DEST = fsr_sys::FFX_API_RESOURCE_STATE_COPY_DEST as u32;
        const GENERIC_READ = fsr_sys::FFX_API_RESOURCE_STATE_GENERIC_READ as u32;
        const INDIRECT_ARGUMENT = fsr_sys::FFX_API_RESOURCE_STATE_INDIRECT_ARGUMENT as u32;
        const PRESENT = fsr_sys::FFX_API_RESOURCE_STATE_PRESENT as u32;
        const RENDER_TARGET = fsr_sys::FFX_API_RESOURCE_STATE_RENDER_TARGET as u32;
        const DEPTH_ATTACHMENT = fsr_sys::FFX_API_RESOURCE_STATE_DEPTH_ATTACHMENT as u32;
    }
}

/// Surface format of a GPU resource.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(u32)]
pub enum ResourceFormat {
    #[default]
    Unknown = fsr_sys::FFX_API_SURFACE_FORMAT_UNKNOWN as u32,
    R32G32B32A32Float = fsr_sys::FFX_API_SURFACE_FORMAT_R32G32B32A32_FLOAT as u32,
    R32G32B32A32Uint = fsr_sys::FFX_API_SURFACE_FORMAT_R32G32B32A32_UINT as u32,
    R16G16B16A16Float = fsr_sys::FFX_API_SURFACE_FORMAT_R16G16B16A16_FLOAT as u32,
    R32G32B32Float = fsr_sys::FFX_API_SURFACE_FORMAT_R32G32B32_FLOAT as u32,
    R32G32Float = fsr_sys::FFX_API_SURFACE_FORMAT_R32G32_FLOAT as u32,
    R8Uint = fsr_sys::FFX_API_SURFACE_FORMAT_R8_UINT as u32,
    R32Uint = fsr_sys::FFX_API_SURFACE_FORMAT_R32_UINT as u32,
    R8G8B8A8Unorm = fsr_sys::FFX_API_SURFACE_FORMAT_R8G8B8A8_UNORM as u32,
    R8G8B8A8Snorm = fsr_sys::FFX_API_SURFACE_FORMAT_R8G8B8A8_SNORM as u32,
    R8G8B8A8Srgb = fsr_sys::FFX_API_SURFACE_FORMAT_R8G8B8A8_SRGB as u32,
    B8G8R8A8Unorm = fsr_sys::FFX_API_SURFACE_FORMAT_B8G8R8A8_UNORM as u32,
    B8G8R8A8Srgb = fsr_sys::FFX_API_SURFACE_FORMAT_B8G8R8A8_SRGB as u32,
    R11G11B10Float = fsr_sys::FFX_API_SURFACE_FORMAT_R11G11B10_FLOAT as u32,
    R10G10B10A2Unorm = fsr_sys::FFX_API_SURFACE_FORMAT_R10G10B10A2_UNORM as u32,
    R16G16Float = fsr_sys::FFX_API_SURFACE_FORMAT_R16G16_FLOAT as u32,
    R16G16Uint = fsr_sys::FFX_API_SURFACE_FORMAT_R16G16_UINT as u32,
    R16G16Sint = fsr_sys::FFX_API_SURFACE_FORMAT_R16G16_SINT as u32,
    R16Float = fsr_sys::FFX_API_SURFACE_FORMAT_R16_FLOAT as u32,
    R16Uint = fsr_sys::FFX_API_SURFACE_FORMAT_R16_UINT as u32,
    R16Unorm = fsr_sys::FFX_API_SURFACE_FORMAT_R16_UNORM as u32,
    R16Snorm = fsr_sys::FFX_API_SURFACE_FORMAT_R16_SNORM as u32,
    R8Unorm = fsr_sys::FFX_API_SURFACE_FORMAT_R8_UNORM as u32,
    R8G8Unorm = fsr_sys::FFX_API_SURFACE_FORMAT_R8G8_UNORM as u32,
    R8G8Uint = fsr_sys::FFX_API_SURFACE_FORMAT_R8G8_UINT as u32,
    R32Float = fsr_sys::FFX_API_SURFACE_FORMAT_R32_FLOAT as u32,
    R9G9B9E5SharedExp = fsr_sys::FFX_API_SURFACE_FORMAT_R9G9B9E5_SHAREDEXP as u32,
}

/// 2D dimensions representation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Dimensions2D {
    pub width: u32,
    pub height: u32,
}

impl Dimensions2D {
    #[inline]
    #[must_use]
    pub const fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}

impl From<(u32, u32)> for Dimensions2D {
    #[inline]
    fn from((width, height): (u32, u32)) -> Self {
        Self { width, height }
    }
}

impl From<Dimensions2D> for (u32, u32) {
    #[inline]
    fn from(dims: Dimensions2D) -> Self {
        (dims.width, dims.height)
    }
}

impl From<Dimensions2D> for fsr_sys::FfxApiDimensions2D {
    #[inline]
    fn from(dims: Dimensions2D) -> Self {
        Self {
            width: dims.width,
            height: dims.height,
        }
    }
}

/// 2D floating-point coordinate representation (e.g. subpixel jitter offset or motion vector scale).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct FloatCoords2D {
    pub x: f32,
    pub y: f32,
}

impl FloatCoords2D {
    #[inline]
    #[must_use]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

impl From<(f32, f32)> for FloatCoords2D {
    #[inline]
    fn from((x, y): (f32, f32)) -> Self {
        Self { x, y }
    }
}

impl From<FloatCoords2D> for (f32, f32) {
    #[inline]
    fn from(coords: FloatCoords2D) -> Self {
        (coords.x, coords.y)
    }
}

impl From<FloatCoords2D> for fsr_sys::FfxApiFloatCoords2D {
    #[inline]
    fn from(coords: FloatCoords2D) -> Self {
        Self {
            x: coords.x,
            y: coords.y,
        }
    }
}
