#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
VENDOR_DIR="${ROOT_DIR}/vendor"
OUT_DIR="${ROOT_DIR}/shaders/spv"

mkdir -p "${OUT_DIR}"

INCLUDE_ARGS=(
  -I "${VENDOR_DIR}/api/internal/gpu"
  -I "${VENDOR_DIR}/upscalers/fsr3/include/gpu"
)

BASE_DEFS=(
  -DFFX_HLSL=1
  -DFFX_GPU=1
  -DFFX_FSR3UPSCALER_EMBED_ROOTSIG=0
  -DFFX_FSR3UPSCALER_OPTION_UPSAMPLE_SAMPLERS_USE_DATA_HALF=0
  -DFFX_FSR3UPSCALER_OPTION_ACCUMULATE_SAMPLERS_USE_DATA_HALF=0
  -DFFX_FSR3UPSCALER_OPTION_REPROJECT_SAMPLERS_USE_DATA_HALF=1
  -DFFX_FSR3UPSCALER_OPTION_POSTPROCESSLOCKSTATUS_SAMPLERS_USE_DATA_HALF=0
  -DFFX_FSR3UPSCALER_OPTION_UPSAMPLE_USE_LANCZOS_TYPE=2
)

SHADERS=(
  "ffx_fsr3upscaler_prepare_inputs_pass"
  "ffx_fsr3upscaler_prepare_reactivity_pass"
  "ffx_fsr3upscaler_luma_pyramid_pass"
  "ffx_fsr3upscaler_shading_change_pass"
  "ffx_fsr3upscaler_shading_change_pyramid_pass"
  "ffx_fsr3upscaler_luma_instability_pass"
  "ffx_fsr3upscaler_accumulate_pass"
  "ffx_fsr3upscaler_rcas_pass"
  "ffx_fsr3upscaler_autogen_reactive_pass"
  "ffx_fsr3upscaler_debug_view_pass"
)

echo "=== Compiling FSR 3.1.5 HLSL Compute Shaders to Vulkan SPIR-V ==="

for shader in "${SHADERS[@]}"; do
  src_file="${VENDOR_DIR}/upscalers/fsr3/internal/shaders/${shader}.hlsl"
  spv_out="${OUT_DIR}/${shader}.spv"

  echo "Compiling ${shader}.hlsl -> ${shader}.spv..."
  dxc -spirv -fspv-target-env=vulkan1.2 -T cs_6_2 -E CS \
    "${INCLUDE_ARGS[@]}" \
    "${BASE_DEFS[@]}" \
    -DFFX_HALF=0 \
    -DFFX_FSR3UPSCALER_OPTION_REPROJECT_USE_LANCZOS_TYPE=1 \
    -DFFX_FSR3UPSCALER_OPTION_HDR_COLOR_INPUT=1 \
    -DFFX_FSR3UPSCALER_OPTION_LOW_RESOLUTION_MOTION_VECTORS=0 \
    -DFFX_FSR3UPSCALER_OPTION_JITTERED_MOTION_VECTORS=0 \
    -DFFX_FSR3UPSCALER_OPTION_INVERTED_DEPTH=1 \
    -DFFX_FSR3UPSCALER_OPTION_APPLY_SHARPENING=1 \
    "${src_file}" \
    -Fo "${spv_out}"

  spirv-val "${spv_out}"
done

echo "=== All 10 FSR 3.1.5 Passes compiled and validated successfully ==="
