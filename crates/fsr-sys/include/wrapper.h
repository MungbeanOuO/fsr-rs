#ifndef FSR_SYS_WRAPPER_H
#define FSR_SYS_WRAPPER_H

#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>

#ifndef _WIN32
#ifndef __declspec
#define __declspec(x) __attribute__((visibility("default")))
#endif
#endif

// Minimal Vulkan handle typedefs to keep fsr-sys build zero-dependency
typedef struct VkDevice_T* VkDevice;
typedef struct VkPhysicalDevice_T* VkPhysicalDevice;
typedef struct VkInstance_T* VkInstance;
typedef struct VkQueue_T* VkQueue;
typedef struct VkCommandBuffer_T* VkCommandBuffer;
typedef uint64_t VkImage;
typedef uint64_t VkBuffer;
typedef uint64_t VkDeviceMemory;
typedef uint64_t VkFence;

typedef void (*PFN_vkVoidFunction)(void);
typedef PFN_vkVoidFunction (*PFN_vkGetDeviceProcAddr)(VkDevice device, const char* pName);

typedef int32_t VkResult;
#define VK_SUCCESS 0

typedef struct VkSubmitInfo VkSubmitInfo;
typedef uint32_t VkFormat;

#include "../vendor/api/include/ffx_api.h"
#include "../vendor/api/include/ffx_api_types.h"
#include "../vendor/api/include/ffx_api_loader.h"
#include "../vendor/api/include/vk/ffx_api_vk.h"
#include "../vendor/upscalers/include/ffx_upscale.h"
#include "../vendor/framegeneration/include/ffx_framegeneration.h"
#include "../vendor/framegeneration/include/ffx_framegeneration_api_types.h"

// Explicit descriptor types evaluated as typed constants
static const uint64_t FFX_API_CREATE_CONTEXT_DESC_TYPE_UPSCALE_CONST = (0x00010000u | 0x00u);
static const uint64_t FFX_API_DISPATCH_DESC_TYPE_UPSCALE_CONST = (0x00010000u | 0x01u);
static const uint64_t FFX_API_CREATE_CONTEXT_DESC_TYPE_FRAMEGENERATION_CONST = (0x00020000u | 0x01u);
static const uint64_t FFX_API_DISPATCH_DESC_TYPE_FRAMEGENERATION_CONST = (0x00020000u | 0x03u);
static const uint64_t FFX_API_DISPATCH_DESC_TYPE_FRAMEGENERATION_PREPARE_V2_CONST = (0x00020000u | 0x0cu);

static const uint64_t FFX_API_CREATE_CONTEXT_DESC_TYPE_BACKEND_DX12_CONST = 2u;
static const uint64_t FFX_API_CREATE_CONTEXT_DESC_TYPE_BACKEND_VK_CONST = (0x01000000u | 0x03u);
static const uint64_t FFX_API_DESC_TYPE_OVERRIDE_VERSION_CONST = 5u;
static const uint64_t FFX_API_QUERY_DESC_TYPE_GET_VERSIONS_CONST = 4u;
static const uint64_t FFX_API_QUERY_DESC_TYPE_GET_PROVIDER_VERSION_CONST = 6u;

// DirectX 12 Backend descriptor extensions for ffx_api
#define FFX_API_BACKEND_ID_DX12 0x00000000u
#define FFX_API_CREATE_CONTEXT_DESC_TYPE_BACKEND_DX12 (FFX_API_BACKEND_ID_DX12 | 0x02)
#define FFX_API_CREATE_CONTEXT_DESC_TYPE_BACKEND_DX12_ALLOCATION_CALLBACKS (FFX_API_BACKEND_ID_DX12 | 0x03)

typedef struct ffxCreateBackendDX12Desc {
    ffxCreateContextDescHeader header;
    void* device; // Pointer to ID3D12Device
} ffxCreateBackendDX12Desc;

#endif // FSR_SYS_WRAPPER_H
