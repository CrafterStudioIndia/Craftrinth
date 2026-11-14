#include "hardware_scanner.hpp"
#include "primeauth_sync.hpp"
#include <windows.h>
#include <iostream>
#include <filesystem>

namespace fs = std::filesystem;

int WINAPI WinMain(HINSTANCE hInstance, HINSTANCE hPrevInstance, LPSTR lpCmdLine, int nCmdShow) {
	(void)hInstance;
	(void)hPrevInstance;
	(void)lpCmdLine;
	(void)nCmdShow;

	// 1. Hardware Inspection: OpenGL & Legacy GPU scanner
	craftrinth::hardware::HardwareScanner scanner;
	auto gpu = scanner.inspect_graphics();

	wchar_t appdata[MAX_PATH];
	GetEnvironmentVariableW(L"APPDATA", appdata, MAX_PATH);
	fs::path craftrinth_dir = fs::path(appdata) / ".craftrinth";

	// 2. Automated Mesa3D deployment for legacy GPUs lacking OpenGL >= 3.2
	if (gpu.requires_mesa3d) {
		fs::path mesa_dir = craftrinth_dir / "launcher" / "mesa3d";
		scanner.ensure_mesa3d_if_needed(gpu, mesa_dir.wstring());
	}

	// 3. Automated PrimeAuthLib sync for Windows 7 & modern targets
	craftrinth::auth::PrimeAuthSync auth_sync;
	fs::path agents_dir = craftrinth_dir / "launcher" / "agents";
	auth_sync.sync_agents(agents_dir);

	// Display initialization status
	std::wstring msg = L"Craftrinth Windows 7 Engine Initialized.\n\nGPU: " + gpu.renderer +
		L"\nOpenGL Driver Version: " + gpu.version +
		L"\nModern OpenGL (>= 3.2): " + (gpu.supports_modern_opengl ? L"Yes" : L"No (Mesa3D Fallback Active)") +
		L"\nPrimeAuthLib: Synchronized & Ready";

	MessageBoxW(nullptr, msg.c_str(), L"Craftrinth - Windows 7 Edition", MB_OK | MB_ICONINFORMATION);

	return 0;
}
