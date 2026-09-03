//! Precompiled Vulkan SPIR-V shader bytecode for AMD FSR 3.1.5 Upscaler compute passes.

pub static FSR3_PREPARE_INPUTS_SPV: &[u8] =
    include_bytes!("../shaders/spv/ffx_fsr3upscaler_prepare_inputs_pass.spv");
pub static FSR3_PREPARE_REACTIVITY_SPV: &[u8] =
    include_bytes!("../shaders/spv/ffx_fsr3upscaler_prepare_reactivity_pass.spv");
pub static FSR3_LUMA_PYRAMID_SPV: &[u8] =
    include_bytes!("../shaders/spv/ffx_fsr3upscaler_luma_pyramid_pass.spv");
pub static FSR3_SHADING_CHANGE_SPV: &[u8] =
    include_bytes!("../shaders/spv/ffx_fsr3upscaler_shading_change_pass.spv");
pub static FSR3_SHADING_CHANGE_PYRAMID_SPV: &[u8] =
    include_bytes!("../shaders/spv/ffx_fsr3upscaler_shading_change_pyramid_pass.spv");
pub static FSR3_LUMA_INSTABILITY_SPV: &[u8] =
    include_bytes!("../shaders/spv/ffx_fsr3upscaler_luma_instability_pass.spv");
pub static FSR3_ACCUMULATE_SPV: &[u8] =
    include_bytes!("../shaders/spv/ffx_fsr3upscaler_accumulate_pass.spv");
pub static FSR3_RCAS_SPV: &[u8] = include_bytes!("../shaders/spv/ffx_fsr3upscaler_rcas_pass.spv");
pub static FSR3_AUTOGEN_REACTIVE_SPV: &[u8] =
    include_bytes!("../shaders/spv/ffx_fsr3upscaler_autogen_reactive_pass.spv");
pub static FSR3_DEBUG_VIEW_SPV: &[u8] =
    include_bytes!("../shaders/spv/ffx_fsr3upscaler_debug_view_pass.spv");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedded_spirv_validity() {
        let all_shaders = [
            FSR3_PREPARE_INPUTS_SPV,
            FSR3_PREPARE_REACTIVITY_SPV,
            FSR3_LUMA_PYRAMID_SPV,
            FSR3_SHADING_CHANGE_SPV,
            FSR3_SHADING_CHANGE_PYRAMID_SPV,
            FSR3_LUMA_INSTABILITY_SPV,
            FSR3_ACCUMULATE_SPV,
            FSR3_RCAS_SPV,
            FSR3_AUTOGEN_REACTIVE_SPV,
            FSR3_DEBUG_VIEW_SPV,
        ];

        for spv in all_shaders {
            assert!(
                spv.len() % 4 == 0,
                "SPIR-V binary must be 4-byte aligned in length"
            );
            // Check SPIR-V Magic Number: 0x07230203 (little-endian: 0x03, 0x02, 0x23, 0x07)
            let magic = u32::from_le_bytes(spv[0..4].try_into().unwrap());
            assert_eq!(magic, 0x07230203, "SPIR-V header magic number mismatch");
        }
    }
}
