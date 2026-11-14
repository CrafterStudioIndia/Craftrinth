#pragma once

#include <string>
#include <vector>
#include <filesystem>

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

	// Checks and fetches the latest primeauthlib jars for both Windows 7 and modern editions
	bool sync_agents(const std::filesystem::path& destination_dir);

	// Returns the correct agent path for the requested Minecraft version
	[[nodiscard]] std::filesystem::path get_agent_for_version(
		const std::filesystem::path& base_dir,
		const std::string& mc_version
	);
};

} // namespace craftrinth::auth
