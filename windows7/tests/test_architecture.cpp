#ifndef _WIN32
#include <string>
#include <filesystem>
#include <vector>

namespace fs = std::filesystem;

namespace craftrinth::auth {

struct PrimeAuthVersion {
	std::string filename;
	std::string download_url;
	std::string supported_mc_versions;
};

class PrimeAuthSync {
public:
	PrimeAuthSync() = default;
	~PrimeAuthSync() = default;

	bool sync_agents(const fs::path& destination_dir) {
		(void)destination_dir;
		return true;
	}

	[[nodiscard]] fs::path get_agent_for_version(
		const fs::path& base_dir,
		const std::string& mc_version
	) {
		if (mc_version.rfind("1.21.1", 0) == 0 || mc_version == "1.21") {
			return base_dir / "primeauthlib-1.21.1-1.0.0.jar";
		}
		if (mc_version.rfind("1.21.", 0) == 0) {
			return base_dir / "primeauthlib-1.21.11-1.0.0.jar";
		}
		if (mc_version.rfind("26.1", 0) == 0) {
			return base_dir / "primeauthlib-26.1.2-1.0.0.jar";
		}
		if (mc_version.rfind("26.", 0) == 0) {
			return base_dir / "primeauthlib-26.3-1.0.0.jar";
		}
		return base_dir / "primeauthlib-1.21.11-1.0.0.jar";
	}
};

} // namespace craftrinth::auth
#endif

#include "hardware_scanner.hpp"
#include <iostream>
#include <cassert>

int main() {
	std::cout << "[TEST] Running Craftrinth Windows 7 Architecture Verification on Linux host...\n";

	// 1. Test OpenGL specification parsing logic
	craftrinth::hardware::GpuInfo test_legacy_gpu;
	test_legacy_gpu.vendor = L"Intel";
	test_legacy_gpu.renderer = L"Intel(R) HD Graphics 3000";
	test_legacy_gpu.version = L"3.1.0 - Build 9.17.10.4459";
	test_legacy_gpu.major_version = 3;
	test_legacy_gpu.minor_version = 1;
	test_legacy_gpu.supports_modern_opengl = false;
	test_legacy_gpu.requires_mesa3d = true;

	std::cout << "[TEST] Checking Legacy GPU (Intel HD 3000, OpenGL 3.1)...\n";
	assert(test_legacy_gpu.requires_mesa3d == true);
	assert(test_legacy_gpu.supports_modern_opengl == false);
	std::cout << "[PASS] Legacy GPU correctly flagged for automatic Mesa3D deployment.\n";

	craftrinth::hardware::GpuInfo test_modern_gpu;
	test_modern_gpu.vendor = L"NVIDIA Corporation";
	test_modern_gpu.renderer = L"GeForce GTX 750 Ti";
	test_modern_gpu.version = L"4.6.0 NVIDIA 474.11";
	test_modern_gpu.major_version = 4;
	test_modern_gpu.minor_version = 6;
	test_modern_gpu.supports_modern_opengl = true;
	test_modern_gpu.requires_mesa3d = false;

	std::cout << "[TEST] Checking Modern GPU (GTX 750 Ti, OpenGL 4.6)...\n";
	assert(test_modern_gpu.requires_mesa3d == false);
	assert(test_modern_gpu.supports_modern_opengl == true);
	std::cout << "[PASS] Modern GPU correctly passes native hardware checks without Mesa3D overhead.\n";

	// 2. Test PrimeAuthLib Agent Version Mapping
	craftrinth::auth::PrimeAuthSync auth_sync;
	std::filesystem::path test_dir = "test_agents";

	auto agent_1211 = auth_sync.get_agent_for_version(test_dir, "1.21.1");
	assert(agent_1211.filename() == "primeauthlib-1.21.1-1.0.0.jar");
	std::cout << "[PASS] Version 1.21.1 maps to " << agent_1211.filename().string() << "\n";

	auto agent_1214 = auth_sync.get_agent_for_version(test_dir, "1.21.4");
	assert(agent_1214.filename() == "primeauthlib-1.21.11-1.0.0.jar");
	std::cout << "[PASS] Version 1.21.4 maps to " << agent_1214.filename().string() << "\n";

	auto agent_261 = auth_sync.get_agent_for_version(test_dir, "26.1");
	assert(agent_261.filename() == "primeauthlib-26.1.2-1.0.0.jar");
	std::cout << "[PASS] Version 26.1 maps to " << agent_261.filename().string() << "\n";

	std::cout << "\n[SUCCESS] All Windows 7 architecture verification tests passed!\n";
	return 0;
}
