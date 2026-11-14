#pragma once

#include <string>
#include <memory>
#include <optional>
#include <vector>

namespace craftrinth::hardware {

struct GpuInfo {
	std::wstring vendor;
	std::wstring renderer;
	std::wstring version;
	int major_version = 0;
	int minor_version = 0;
	bool supports_modern_opengl = false; // >= 3.2 required for Minecraft 1.17+
	bool requires_mesa3d = false;
};

class HardwareScanner {
public:
	HardwareScanner() = default;
	~HardwareScanner() = default;

	// Scans system GPU and reports OpenGL capability
	[[nodiscard]] GpuInfo inspect_graphics();

	// Ensures Mesa3D software renderer is deployed if hardware lacks OpenGL >= 3.2
	bool ensure_mesa3d_if_needed(const GpuInfo& info, const std::wstring& target_dir);
};

} // namespace craftrinth::hardware
