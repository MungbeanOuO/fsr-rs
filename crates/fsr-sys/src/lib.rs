#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(clippy::all)]
#![allow(clippy::pedantic)]
#![allow(clippy::nursery)]
#![allow(rustdoc::broken_intra_doc_links)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

pub mod shaders;

pub const FFX_API_CREATE_CONTEXT_DESC_TYPE_UPSCALE: u64 =
    FFX_API_CREATE_CONTEXT_DESC_TYPE_UPSCALE_CONST;
pub const FFX_API_DISPATCH_DESC_TYPE_UPSCALE: u64 = FFX_API_DISPATCH_DESC_TYPE_UPSCALE_CONST;
pub const FFX_API_CREATE_CONTEXT_DESC_TYPE_FRAMEGENERATION: u64 =
    FFX_API_CREATE_CONTEXT_DESC_TYPE_FRAMEGENERATION_CONST;
pub const FFX_API_DISPATCH_DESC_TYPE_FRAMEGENERATION: u64 =
    FFX_API_DISPATCH_DESC_TYPE_FRAMEGENERATION_CONST;
pub const FFX_API_DISPATCH_DESC_TYPE_FRAMEGENERATION_PREPARE_V2: u64 =
    FFX_API_DISPATCH_DESC_TYPE_FRAMEGENERATION_PREPARE_V2_CONST;

// Convenient aliases with idiomatic PascalCase
pub type FfxApiHeader = ffxApiHeader;
pub type FfxCreateContextDescUpscale = ffxCreateContextDescUpscale;
pub type FfxDispatchDescUpscale = ffxDispatchDescUpscale;
pub type FfxCreateBackendVKDesc = ffxCreateBackendVKDesc;
pub type FfxCreateBackendVKAllocationCallbacksDesc = ffxCreateBackendVKAllocationCallbacksDesc;
pub type FfxCreateBackendDX12Desc = ffxCreateBackendDX12Desc;
pub type FfxOverrideVersion = ffxOverrideVersion;
pub type FfxQueryDescGetVersions = ffxQueryDescGetVersions;
pub type FfxQueryGetProviderVersion = ffxQueryGetProviderVersion;

pub type FfxCreateContextDescFrameGeneration = ffxCreateContextDescFrameGeneration;
pub type FfxDispatchDescFrameGeneration = ffxDispatchDescFrameGeneration;
pub type FfxDispatchDescFrameGenerationPrepareV2 = ffxDispatchDescFrameGenerationPrepareV2;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_core_struct_sizes() {
        assert_eq!(std::mem::size_of::<ffxApiHeader>(), 16);
        assert_eq!(std::mem::align_of::<ffxApiHeader>(), 8);

        // Verify FfxApiResource layout
        assert!(std::mem::size_of::<FfxApiResource>() > 0);
        assert_eq!(std::mem::align_of::<FfxApiResource>(), 8);

        // Verify Context and Dispatch struct sizes
        assert!(std::mem::size_of::<ffxCreateContextDescUpscale>() > 0);
        assert!(std::mem::size_of::<ffxDispatchDescUpscale>() > 0);

        // Verify DX12 Backend Desc size and alignment
        assert_eq!(std::mem::size_of::<ffxCreateBackendDX12Desc>(), 24);
        assert_eq!(std::mem::align_of::<ffxCreateBackendDX12Desc>(), 8);

        // Verify Override Version struct size
        assert_eq!(std::mem::size_of::<ffxOverrideVersion>(), 24);
        assert_eq!(std::mem::align_of::<ffxOverrideVersion>(), 8);

        // Verify Frame Generation struct sizes
        assert!(std::mem::size_of::<ffxCreateContextDescFrameGeneration>() > 0);
        assert!(std::mem::size_of::<ffxDispatchDescFrameGeneration>() > 0);
        assert!(std::mem::size_of::<ffxDispatchDescFrameGenerationPrepareV2>() > 0);
    }

    #[test]
    fn test_effect_ids() {
        assert_eq!(FFX_API_EFFECT_ID_UPSCALE, 0x00010000);
        assert_eq!(FFX_API_EFFECT_ID_FRAMEGENERATION, 0x00020000);
        assert_eq!(FFX_API_RETURN_OK, 0);
        assert_eq!(FFX_API_CREATE_CONTEXT_DESC_TYPE_UPSCALE, 0x00010000);
        assert_eq!(FFX_API_DISPATCH_DESC_TYPE_UPSCALE, 0x00010001);
        assert_eq!(FFX_API_CREATE_CONTEXT_DESC_TYPE_FRAMEGENERATION, 0x00020001);
        assert_eq!(FFX_API_DISPATCH_DESC_TYPE_FRAMEGENERATION, 0x00020003);
        assert_eq!(
            FFX_API_DISPATCH_DESC_TYPE_FRAMEGENERATION_PREPARE_V2,
            0x0002000c
        );
        assert_eq!(FFX_API_CREATE_CONTEXT_DESC_TYPE_BACKEND_DX12, 2);
        assert_eq!(FFX_API_DESC_TYPE_OVERRIDE_VERSION, 5);
        assert_eq!(FFX_API_QUERY_DESC_TYPE_GET_VERSIONS, 4);
    }
}
