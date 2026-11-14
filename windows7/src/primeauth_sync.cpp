#include "primeauth_sync.hpp"
#include <windows.h>
#include <wininet.h>
#include <fstream>
#include <iostream>

namespace fs = std::filesystem;

namespace craftrinth::auth {

namespace {

bool download_file(const std::wstring& url, const fs::path& dest_path) {
	HINTERNET hInternet = InternetOpenW(L"Craftrinth-PrimeAuthSync", INTERNET_OPEN_TYPE_PRECONFIG, nullptr, nullptr, 0);
	if (!hInternet) return false;

	HINTERNET hUrl = InternetOpenUrlW(hInternet, url.c_str(), nullptr, 0, INTERNET_FLAG_RELOAD | INTERNET_FLAG_SECURE, 0);
	if (!hUrl) {
		InternetCloseHandle(hInternet);
		return false;
	}

	fs::path temp_path = dest_path;
	temp_path += ".tmp";

	std::ofstream out_file(temp_path, std::ios::binary);
	if (!out_file.is_open()) {
		InternetCloseHandle(hUrl);
		InternetCloseHandle(hInternet);
		return false;
	}

	char buffer[8192];
	DWORD bytes_read = 0;
	while (InternetReadFile(hUrl, buffer, sizeof(buffer), &bytes_read) && bytes_read > 0) {
		out_file.write(buffer, bytes_read);
	}

	out_file.close();
	InternetCloseHandle(hUrl);
	InternetCloseHandle(hInternet);

	if (fs::exists(temp_path) && fs::file_size(temp_path) > 1024) {
		std::error_code ec;
		fs::rename(temp_path, dest_path, ec);
		return !ec;
	}

	fs::remove(temp_path);
	return false;
}

std::wstring to_wstring(const std::string& str) {
	if (str.empty()) return L"";
	int len = MultiByteToWideChar(CP_UTF8, 0, str.c_str(), -1, nullptr, 0);
	if (len <= 0) return L"";
	std::wstring res(len - 1, 0);
	MultiByteToWideChar(CP_UTF8, 0, str.c_str(), -1, res.data(), len);
	return res;
}

} // namespace

bool PrimeAuthSync::sync_agents(const fs::path& destination_dir) {
	fs::create_directories(destination_dir);

	// Standard PrimeAuthLib targets across modern and legacy Minecraft versions
	const std::vector<PrimeAuthVersion> agents = {
		{
			"primeauthlib-1.21.1-1.0.0.jar",
			"https://raw.githubusercontent.com/CrafterStudioIndia/Craftrinth/main/packages/app-lib/resources/primeauthlib/primeauthlib-1.21.1-1.0.0.jar",
			"1.21, 1.21.1"
		},
		{
			"primeauthlib-1.21.11-1.0.0.jar",
			"https://raw.githubusercontent.com/CrafterStudioIndia/Craftrinth/main/packages/app-lib/resources/primeauthlib/primeauthlib-1.21.11-1.0.0.jar",
			"1.21.2, 1.21.3, 1.21.4, 1.21.5, 1.21.11"
		},
		{
			"primeauthlib-26.1.2-1.0.0.jar",
			"https://raw.githubusercontent.com/CrafterStudioIndia/Craftrinth/main/packages/app-lib/resources/primeauthlib/primeauthlib-26.1.2-1.0.0.jar",
			"26.1, 26.1.1, 26.1.2"
		},
		{
			"primeauthlib-26.3-1.0.0.jar",
			"https://raw.githubusercontent.com/CrafterStudioIndia/Craftrinth/main/packages/app-lib/resources/primeauthlib/primeauthlib-26.3-1.0.0.jar",
			"26.2, 26.3"
		}
	};

	bool all_ok = true;
	for (const auto& agent : agents) {
		fs::path target_file = destination_dir / agent.filename;
		if (!fs::exists(target_file) || fs::file_size(target_file) < 1024) {
			std::wcout << L"[PrimeAuthSync] Downloading " << to_wstring(agent.filename) << std::endl;
			if (!download_file(to_wstring(agent.download_url), target_file)) {
				all_ok = false;
			}
		}
	}

	return all_ok;
}

fs::path PrimeAuthSync::get_agent_for_version(
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

} // namespace craftrinth::auth
