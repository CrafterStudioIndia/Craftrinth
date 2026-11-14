#include "hardware_scanner.hpp"
#include <windows.h>
#include <GL/gl.h>
#include <iostream>
#include <sstream>
#include <wininet.h>
#include <filesystem>
#include <fstream>

namespace fs = std::filesystem;

namespace craftrinth::hardware {

namespace {

// RAII Wrapper for Windows Device Context and OpenGL Dummy Context
class ScopedGlContext {
public:
	ScopedGlContext() {
		// Register a temporary window class for context probing
		WNDCLASSW wc = {};
		wc.lpfnWndProc = DefWindowProcW;
		wc.hInstance = GetModuleHandleW(nullptr);
		wc.lpszClassName = L"CraftrinthGlProbe";
		RegisterClassW(&wc);

		hwnd_ = CreateWindowW(
			wc.lpszClassName, L"",
			WS_OVERLAPPEDWINDOW, 0, 0, 10, 10,
			nullptr, nullptr, wc.hInstance, nullptr
		);

		if (!hwnd_) return;

		hdc_ = GetDC(hwnd_);
		PIXELFORMATDESCRIPTOR pfd = {};
		pfd.nSize = sizeof(pfd);
		pfd.nVersion = 1;
		pfd.dwFlags = PFD_DRAW_TO_WINDOW | PFD_SUPPORT_OPENGL | PFD_DOUBLEBUFFER;
		pfd.iPixelType = PFD_TYPE_RGBA;
		pfd.cColorBits = 24;

		int format = ChoosePixelFormat(hdc_, &pfd);
		if (format) {
			SetPixelFormat(hdc_, format, &pfd);
			hglrc_ = wglCreateContext(hdc_);
			if (hglrc_) {
				wglMakeCurrent(hdc_, hglrc_);
				valid_ = true;
			}
		}
	}

	~ScopedGlContext() {
		if (hglrc_) {
			wglMakeCurrent(nullptr, nullptr);
			wglDeleteContext(hglrc_);
		}
		if (hdc_ && hwnd_) {
			ReleaseDC(hwnd_, hdc_);
		}
		if (hwnd_) {
			DestroyWindow(hwnd_);
		}
		UnregisterClassW(L"CraftrinthGlProbe", GetModuleHandleW(nullptr));
	}

	[[nodiscard]] bool is_valid() const noexcept { return valid_; }

private:
	HWND hwnd_ = nullptr;
	HDC hdc_ = nullptr;
	HGLRC hglrc_ = nullptr;
	bool valid_ = false;
};

// Safe string conversion helper
std::wstring to_wstring(const char* str) {
	if (!str) return L"";
	int len = MultiByteToWideChar(CP_UTF8, 0, str, -1, nullptr, 0);
	if (len <= 0) return L"";
	std::wstring result(len - 1, 0);
	MultiByteToWideChar(CP_UTF8, 0, str, -1, result.data(), len);
	return result;
}

} // namespace

GpuInfo HardwareScanner::inspect_graphics() {
	GpuInfo info;
	ScopedGlContext gl_probe;

	if (!gl_probe.is_valid()) {
		info.vendor = L"Unknown / No Hardware Acceleration";
		info.renderer = L"Generic GDI Driver";
		info.version = L"1.1.0";
		info.major_version = 1;
		info.minor_version = 1;
		info.supports_modern_opengl = false;
		info.requires_mesa3d = true;
		return info;
	}

	const char* vendor_str = reinterpret_cast<const char*>(glGetString(GL_VENDOR));
	const char* renderer_str = reinterpret_cast<const char*>(glGetString(GL_RENDERER));
	const char* version_str = reinterpret_cast<const char*>(glGetString(GL_VERSION));

	info.vendor = to_wstring(vendor_str);
	info.renderer = to_wstring(renderer_str);
	info.version = to_wstring(version_str);

	if (version_str) {
		std::istringstream ss(version_str);
		char dot = 0;
		if (ss >> info.major_version >> dot >> info.minor_version) {
			if (info.major_version > 3 || (info.major_version == 3 && info.minor_version >= 2)) {
				info.supports_modern_opengl = true;
			}
		}
	}

	// Minecraft 1.17+ requires OpenGL 3.2+ Core Profile
	// Intel HD 2000/3000 only support up to OpenGL 3.1 on Windows 7
	if (!info.supports_modern_opengl) {
		info.requires_mesa3d = true;
	}

	return info;
}

bool HardwareScanner::ensure_mesa3d_if_needed(const GpuInfo& info, const std::wstring& target_dir) {
	if (!info.requires_mesa3d) {
		return true;
	}

	fs::path out_path = fs::path(target_dir) / "opengl32.dll";
	if (fs::exists(out_path) && fs::file_size(out_path) > 1024 * 1024) {
		return true; // Already provisioned
	}

	fs::create_directories(target_dir);

	// Download Mesa3D software rasterizer build for Windows 7
	HINTERNET hInternet = InternetOpenW(L"Craftrinth-Win7-HardwareSync", INTERNET_OPEN_TYPE_PRECONFIG, nullptr, nullptr, 0);
	if (!hInternet) return false;

	// Pre-built Mesa3D Windows opengl32.dll software renderer URL
	const wchar_t* mesa_url = L"https://github.com/pal1000/mesa-dist-win/releases/download/23.1.9/mesa3d-23.1.9-release-msvc.7z";

	HINTERNET hUrl = InternetOpenUrlW(hInternet, mesa_url, nullptr, 0, INTERNET_FLAG_RELOAD | INTERNET_FLAG_SECURE, 0);
	if (!hUrl) {
		InternetCloseHandle(hInternet);
		return false;
	}

	fs::path temp_download = fs::path(target_dir) / "opengl32.dll.part";
	std::ofstream out_file(temp_download, std::ios::binary);

	char buffer[8192];
	DWORD bytes_read = 0;
	bool success = true;

	while (InternetReadFile(hUrl, buffer, sizeof(buffer), &bytes_read) && bytes_read > 0) {
		out_file.write(buffer, bytes_read);
	}

	out_file.close();
	InternetCloseHandle(hUrl);
	InternetCloseHandle(hInternet);

	if (fs::exists(temp_download) && fs::file_size(temp_download) > 0) {
		fs::rename(temp_download, out_path);
		return true;
	}

	return false;
}

} // namespace craftrinth::hardware
