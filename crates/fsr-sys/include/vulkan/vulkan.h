#ifndef VULKAN_H_
#define VULKAN_H_

#include <stdint.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

#define VK_DEFINE_HANDLE(object) typedef struct object##_T* object;
#define VK_DEFINE_NON_DISPATCHABLE_HANDLE(object) typedef uint64_t object;

VK_DEFINE_HANDLE(VkInstance)
VK_DEFINE_HANDLE(VkPhysicalDevice)
VK_DEFINE_HANDLE(VkDevice)
VK_DEFINE_HANDLE(VkQueue)
VK_DEFINE_HANDLE(VkCommandBuffer)
VK_DEFINE_NON_DISPATCHABLE_HANDLE(VkImage)
VK_DEFINE_NON_DISPATCHABLE_HANDLE(VkBuffer)
VK_DEFINE_NON_DISPATCHABLE_HANDLE(VkFence)
VK_DEFINE_NON_DISPATCHABLE_HANDLE(VkSwapchainKHR)

typedef void (*PFN_vkVoidFunction)(void);
typedef PFN_vkVoidFunction (*PFN_vkGetDeviceProcAddr)(VkDevice device, const char* pName);

typedef int32_t VkResult;
#define VK_SUCCESS 0

typedef struct VkSubmitInfo VkSubmitInfo;
typedef struct VkAllocationCallbacks VkAllocationCallbacks;
typedef struct VkSwapchainCreateInfoKHR VkSwapchainCreateInfoKHR;
typedef struct VkHdrMetadataEXT VkHdrMetadataEXT;

typedef uint32_t VkFormat;

typedef VkResult (*PFN_vkQueuePresentKHR)(VkQueue queue, const void* pPresentInfo);
typedef VkResult (*PFN_vkAcquireNextImageKHR)(VkDevice device, VkSwapchainKHR swapchain, uint64_t timeout, uint64_t semaphore, VkFence fence, uint32_t* pImageIndex);
typedef VkResult (*PFN_vkGetSwapchainImagesKHR)(VkDevice device, VkSwapchainKHR swapchain, uint32_t* pSwapchainImageCount, VkImage* pSwapchainImages);
typedef void (*PFN_vkSetHdrMetadataEXT)(VkDevice device, uint32_t swapchainCount, const VkSwapchainKHR* pSwapchains, const VkHdrMetadataEXT* pMetadata);

#ifdef __cplusplus
}
#endif

#endif // VULKAN_H_
