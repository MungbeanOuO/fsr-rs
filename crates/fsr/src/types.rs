use bitflags::bitflags;
use std::fmt;

/// Errors returned by FSR operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FsrError {
    /// An unspecified error occurred.
    Error,
    /// Unknown descriptor structure type passed to the API.
    UnknownDescriptorType,
    /// Underlying graphics runtime or driver returned an error.
    RuntimeError,
    /// No provider found for the requested structure type or version.
    NoProvider,
    /// Memory allocation failed.
    OutOfMemory,
    /// Invalid parameter (null pointer, invalid bounds, etc.).
    InvalidParameter,
    /// Structure type is newer and not supported by current provider.
    ProviderNotSupported,
    /// Other return code.
    Other(u32),
}

impl FsrError {
    pub(crate) fn from_return_code(code: u32) -> Result<(), Self> {
        match code {
            fsr_sys::FFX_API_RETURN_OK => Ok(()),
            fsr_sys::FFX_API_RETURN_ERROR => Err(FsrError::Error),
            fsr_sys::FFX_API_RETURN_ERROR_UNKNOWN_DESCTYPE => Err(FsrError::UnknownDescriptorType),
            fsr_sys::FFX_API_RETURN_ERROR_RUNTIME_ERROR => Err(FsrError::RuntimeError),
            fsr_sys::FFX_API_RETURN_NO_PROVIDER => Err(FsrError::NoProvider),
            fsr_sys::FFX_API_RETURN_ERROR_MEMORY => Err(FsrError::OutOfMemory),
            fsr_sys::FFX_API_RETURN_ERROR_PARAMETER => Err(FsrError::InvalidParameter),
            fsr_sys::FFX_API_RETURN_PROVIDER_NO_SUPPORT_NEW_DESCTYPE => {
                Err(FsrError::ProviderNotSupported)
            }
            other => Err(FsrError::Other(other)),
        }
    }
}

impl fmt::Display for FsrError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Error => write!(f, "FSR general error"),
            Self::UnknownDescriptorType => write!(f, "FSR unknown descriptor structure type"),
            Self::RuntimeError => write!(f, "FSR underlying runtime error"),
            Self::NoProvider => write!(f, "FSR no provider found"),
            Self::OutOfMemory => write!(f, "FSR out of memory"),
            Self::InvalidParameter => write!(f, "FSR invalid parameter"),
            Self::ProviderNotSupported => write!(f, "FSR provider does not support structure type"),
            Self::Other(code) => write!(f, "FSR error (code: {code})"),
        }
    }
}

impl std::error::Error for FsrError {}

/// FSR Upscaling Quality Modes and preset scaling factors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum QualityMode {
    /// Native AA: 1.0x scaling per dimension (anti-aliasing only).
    NativeAA = fsr_sys::FFX_UPSCALE_QUALITY_MODE_NATIVEAA,
    /// Quality: 1.5x scaling per dimension.
    Quality = fsr_sys::FFX_UPSCALE_QUALITY_MODE_QUALITY,
    /// Balanced: 1.7x scaling per dimension.
    Balanced = fsr_sys::FFX_UPSCALE_QUALITY_MODE_BALANCED,
    /// Performance: 2.0x scaling per dimension.
    Performance = fsr_sys::FFX_UPSCALE_QUALITY_MODE_PERFORMANCE,
    /// Ultra Performance: 3.0x scaling per dimension.
    UltraPerformance = fsr_sys::FFX_UPSCALE_QUALITY_MODE_ULTRA_PERFORMANCE,
}

impl QualityMode {
    /// Array containing all standard quality modes for UI selection and iteration.
    pub const ALL: [QualityMode; 5] = [
        QualityMode::NativeAA,
        QualityMode::Quality,
        QualityMode::Balanced,
        QualityMode::Performance,
        QualityMode::UltraPerformance,
    ];

    /// Returns a human-readable display label and upscale factor for the quality mode.
    #[inline]
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

    /// Get the per-dimension upscaling ratio for this quality mode.
    #[inline]
    pub const fn upscale_ratio(self) -> f32 {
        match self {
            Self::NativeAA => 1.0,
            Self::Quality => 1.5,
            Self::Balanced => 1.7,
            Self::Performance => 2.0,
            Self::UltraPerformance => 3.0,
        }
    }

    /// Calculate the recommended render resolution for a target display resolution.
    #[inline]
    pub fn render_resolution(self, target_width: u32, target_height: u32) -> (u32, u32) {
        let ratio = self.upscale_ratio();
        let render_w = ((target_width as f32 / ratio).round() as u32).max(1);
        let render_h = ((target_height as f32 / ratio).round() as u32).max(1);
        (render_w, render_h)
    }
}

impl fmt::Display for QualityMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label())
    }
}

bitflags! {
    /// Configuration flags for FSR Upscaler context creation.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct InitializationFlags: u32 {
        /// Input color data is using high dynamic range (HDR).
        const HIGH_DYNAMIC_RANGE = fsr_sys::FFX_UPSCALE_ENABLE_HIGH_DYNAMIC_RANGE;
        /// Motion vectors are rendered at display resolution instead of render resolution.
        const DISPLAY_RESOLUTION_MOTION_VECTORS = fsr_sys::FFX_UPSCALE_ENABLE_DISPLAY_RESOLUTION_MOTION_VECTORS;
        /// Motion vectors already contain jitter cancellation.
        const MOTION_VECTORS_JITTER_CANCELLATION = fsr_sys::FFX_UPSCALE_ENABLE_MOTION_VECTORS_JITTER_CANCELLATION;
        /// Input depth buffer is inverted [1.0 .. 0.0].
        const DEPTH_INVERTED = fsr_sys::FFX_UPSCALE_ENABLE_DEPTH_INVERTED;
        /// Camera uses an infinite far plane.
        const DEPTH_INFINITE = fsr_sys::FFX_UPSCALE_ENABLE_DEPTH_INFINITE;
        /// Apply automatic exposure calculation to input colors.
        const AUTO_EXPOSURE = fsr_sys::FFX_UPSCALE_ENABLE_AUTO_EXPOSURE;
        /// Application uses dynamic resolution scaling.
        const DYNAMIC_RESOLUTION = fsr_sys::FFX_UPSCALE_ENABLE_DYNAMIC_RESOLUTION;
        /// Runtime should validate API values and report debug warnings.
        const DEBUG_CHECKING = fsr_sys::FFX_UPSCALE_ENABLE_DEBUG_CHECKING;
        /// Color resource contains perceptual (gamma corrected) colors.
        const NON_LINEAR_COLORSPACE = fsr_sys::FFX_UPSCALE_ENABLE_NON_LINEAR_COLORSPACE;
        /// Enable debug visualization views.
        const DEBUG_VISUALIZATION = fsr_sys::FFX_UPSCALE_ENABLE_DEBUG_VISUALIZATION;
    }
}

bitflags! {
    /// Flags for per-frame FSR upscaler dispatch.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct DispatchFlags: u32 {
        /// Draw debug visualization information into output resource.
        const DRAW_DEBUG_VIEW = fsr_sys::FFX_UPSCALE_FLAG_DRAW_DEBUG_VIEW;
        /// Input color resource contains perceptual sRGB colors.
        const NON_LINEAR_COLOR_SRGB = fsr_sys::FFX_UPSCALE_FLAG_NON_LINEAR_COLOR_SRGB;
        /// Input color resource contains perceptual PQ colors.
        const NON_LINEAR_COLOR_PQ = fsr_sys::FFX_UPSCALE_FLAG_NON_LINEAR_COLOR_PQ;
    }
}

bitflags! {
    /// Configuration flags for FSR Frame Generation context creation.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct FrameGenerationInitializationFlags: u32 {
        /// Support async compute workloads on async compute queues.
        const ASYNC_WORKLOAD_SUPPORT = fsr_sys::FFX_FRAMEGENERATION_ENABLE_ASYNC_WORKLOAD_SUPPORT;
        /// Motion vectors are rendered at display resolution instead of render resolution.
        const DISPLAY_RESOLUTION_MOTION_VECTORS = fsr_sys::FFX_FRAMEGENERATION_ENABLE_DISPLAY_RESOLUTION_MOTION_VECTORS;
        /// Motion vectors already contain jitter pattern.
        const MOTION_VECTORS_JITTER_CANCELLATION = fsr_sys::FFX_FRAMEGENERATION_ENABLE_MOTION_VECTORS_JITTER_CANCELLATION;
        /// Input depth buffer is inverted [1.0 .. 0.0].
        const DEPTH_INVERTED = fsr_sys::FFX_FRAMEGENERATION_ENABLE_DEPTH_INVERTED;
        /// Camera uses an infinite far plane.
        const DEPTH_INFINITE = fsr_sys::FFX_FRAMEGENERATION_ENABLE_DEPTH_INFINITE;
        /// Input color data is using high dynamic range (HDR).
        const HIGH_DYNAMIC_RANGE = fsr_sys::FFX_FRAMEGENERATION_ENABLE_HIGH_DYNAMIC_RANGE;
        /// Validate API values and report debug issues.
        const DEBUG_CHECKING = fsr_sys::FFX_FRAMEGENERATION_ENABLE_DEBUG_CHECKING;
    }
}

bitflags! {
    /// Flags for per-frame FSR frame generation dispatch and preparation.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct FrameGenerationDispatchFlags: u32 {
        /// Draw debug tear lines to generated output.
        const DRAW_DEBUG_TEAR_LINES = fsr_sys::FFX_FRAMEGENERATION_FLAG_DRAW_DEBUG_TEAR_LINES;
        /// Draw debug reset indicators to generated output.
        const DRAW_DEBUG_RESET_INDICATORS = fsr_sys::FFX_FRAMEGENERATION_FLAG_DRAW_DEBUG_RESET_INDICATORS;
        /// Draw debug views with relevant information.
        const DRAW_DEBUG_VIEW = fsr_sys::FFX_FRAMEGENERATION_FLAG_DRAW_DEBUG_VIEW;
        /// Only run frame interpolation without modifying the swapchain.
        const NO_SWAPCHAIN_CONTEXT_NOTIFY = fsr_sys::FFX_FRAMEGENERATION_FLAG_NO_SWAPCHAIN_CONTEXT_NOTIFY;
        /// Draw debug pacing lines to generated output.
        const DRAW_DEBUG_PACING_LINES = fsr_sys::FFX_FRAMEGENERATION_FLAG_DRAW_DEBUG_PACING_LINES;
    }
}

bitflags! {
    /// Flags for UI composition in frame generation.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct UiCompositionFlags: u32 {
        /// Use premultiplied alpha for UI composition.
        const USE_PREMUL_ALPHA = fsr_sys::FFX_FRAMEGENERATION_UI_COMPOSITION_FLAG_USE_PREMUL_ALPHA;
        /// Double buffer UI resource in swapchain.
        const ENABLE_INTERNAL_UI_DOUBLE_BUFFERING = fsr_sys::FFX_FRAMEGENERATION_UI_COMPOSITION_FLAG_ENABLE_INTERNAL_UI_DOUBLE_BUFFERING;
    }
}

bitflags! {
    /// Explicit state of a GPU resource when passed to FSR.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct ResourceState: u32 {
        const COMMON = fsr_sys::FFX_API_RESOURCE_STATE_COMMON;
        const UNORDERED_ACCESS = fsr_sys::FFX_API_RESOURCE_STATE_UNORDERED_ACCESS;
        const COMPUTE_READ = fsr_sys::FFX_API_RESOURCE_STATE_COMPUTE_READ;
        const PIXEL_READ = fsr_sys::FFX_API_RESOURCE_STATE_PIXEL_READ;
        const PIXEL_COMPUTE_READ = fsr_sys::FFX_API_RESOURCE_STATE_PIXEL_COMPUTE_READ;
        const COPY_SRC = fsr_sys::FFX_API_RESOURCE_STATE_COPY_SRC;
        const COPY_DEST = fsr_sys::FFX_API_RESOURCE_STATE_COPY_DEST;
        const GENERIC_READ = fsr_sys::FFX_API_RESOURCE_STATE_GENERIC_READ;
        const INDIRECT_ARGUMENT = fsr_sys::FFX_API_RESOURCE_STATE_INDIRECT_ARGUMENT;
        const PRESENT = fsr_sys::FFX_API_RESOURCE_STATE_PRESENT;
        const RENDER_TARGET = fsr_sys::FFX_API_RESOURCE_STATE_RENDER_TARGET;
        const DEPTH_ATTACHMENT = fsr_sys::FFX_API_RESOURCE_STATE_DEPTH_ATTACHMENT;
    }
}

/// Surface format of a GPU resource.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(u32)]
pub enum ResourceFormat {
    #[default]
    Unknown = fsr_sys::FFX_API_SURFACE_FORMAT_UNKNOWN,
    R32G32B32A32Float = fsr_sys::FFX_API_SURFACE_FORMAT_R32G32B32A32_FLOAT,
    R32G32B32A32Uint = fsr_sys::FFX_API_SURFACE_FORMAT_R32G32B32A32_UINT,
    R16G16B16A16Float = fsr_sys::FFX_API_SURFACE_FORMAT_R16G16B16A16_FLOAT,
    R32G32B32Float = fsr_sys::FFX_API_SURFACE_FORMAT_R32G32B32_FLOAT,
    R32G32Float = fsr_sys::FFX_API_SURFACE_FORMAT_R32G32_FLOAT,
    R8Uint = fsr_sys::FFX_API_SURFACE_FORMAT_R8_UINT,
    R32Uint = fsr_sys::FFX_API_SURFACE_FORMAT_R32_UINT,
    R8G8B8A8Unorm = fsr_sys::FFX_API_SURFACE_FORMAT_R8G8B8A8_UNORM,
    R8G8B8A8Snorm = fsr_sys::FFX_API_SURFACE_FORMAT_R8G8B8A8_SNORM,
    R8G8B8A8Srgb = fsr_sys::FFX_API_SURFACE_FORMAT_R8G8B8A8_SRGB,
    B8G8R8A8Unorm = fsr_sys::FFX_API_SURFACE_FORMAT_B8G8R8A8_UNORM,
    B8G8R8A8Srgb = fsr_sys::FFX_API_SURFACE_FORMAT_B8G8R8A8_SRGB,
    R11G11B10Float = fsr_sys::FFX_API_SURFACE_FORMAT_R11G11B10_FLOAT,
    R10G10B10A2Unorm = fsr_sys::FFX_API_SURFACE_FORMAT_R10G10B10A2_UNORM,
    R16G16Float = fsr_sys::FFX_API_SURFACE_FORMAT_R16G16_FLOAT,
    R16G16Uint = fsr_sys::FFX_API_SURFACE_FORMAT_R16G16_UINT,
    R16G16Sint = fsr_sys::FFX_API_SURFACE_FORMAT_R16G16_SINT,
    R16Float = fsr_sys::FFX_API_SURFACE_FORMAT_R16_FLOAT,
    R16Uint = fsr_sys::FFX_API_SURFACE_FORMAT_R16_UINT,
    R16Unorm = fsr_sys::FFX_API_SURFACE_FORMAT_R16_UNORM,
    R16Snorm = fsr_sys::FFX_API_SURFACE_FORMAT_R16_SNORM,
    R8Unorm = fsr_sys::FFX_API_SURFACE_FORMAT_R8_UNORM,
    R8G8Unorm = fsr_sys::FFX_API_SURFACE_FORMAT_R8G8_UNORM,
    R8G8Uint = fsr_sys::FFX_API_SURFACE_FORMAT_R8G8_UINT,
    R32Float = fsr_sys::FFX_API_SURFACE_FORMAT_R32_FLOAT,
    R9G9B9E5SharedExp = fsr_sys::FFX_API_SURFACE_FORMAT_R9G9B9E5_SHAREDEXP,
}

/// 2D dimensions representation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Dimensions2D {
    pub width: u32,
    pub height: u32,
}

impl Dimensions2D {
    #[inline]
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

/// 2D floating point coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct FloatCoords2D {
    pub x: f32,
    pub y: f32,
}

impl FloatCoords2D {
    #[inline]
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
