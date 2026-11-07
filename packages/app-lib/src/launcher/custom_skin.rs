//! CustomSkinLoader and authlib-injector dual-layer skin pipeline
use std::path::{Path, PathBuf};
use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrackedSkinItem {
	pub username: String,
	pub filename: String,
	pub path: String,
	pub modified: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrackedStatus {
	pub authlib_present: bool,
	pub authlib_path: String,
	pub auth_endpoint: String,
	#[serde(alias = "createlauncher_dir")]
	pub craftrinth_dir: String,
	pub skins: Vec<CrackedSkinItem>,
	#[serde(default)]
	pub primeauthlib_present: bool,
}

pub struct PrimeAuthLibJar {
	pub filename: &'static str,
	pub bytes: &'static [u8],
	pub supported_versions: &'static [&'static str],
}

pub static BUNDLED_PRIMEAUTHLIB_JARS: &[PrimeAuthLibJar] = &[
	PrimeAuthLibJar {
		filename: "primeauthlib-1.21.1-1.0.0.jar",
		bytes: include_bytes!("../../resources/primeauthlib/primeauthlib-1.21.1-1.0.0.jar"),
		supported_versions: &["1.21", "1.21.0", "1.21.1"],
	},
	PrimeAuthLibJar {
		filename: "primeauthlib-1.21.11-1.0.0.jar",
		bytes: include_bytes!("../../resources/primeauthlib/primeauthlib-1.21.11-1.0.0.jar"),
		supported_versions: &["1.21.2", "1.21.3", "1.21.4", "1.21.5", "1.21.11"],
	},
	PrimeAuthLibJar {
		filename: "primeauthlib-26.1.2-1.0.0.jar",
		bytes: include_bytes!("../../resources/primeauthlib/primeauthlib-26.1.2-1.0.0.jar"),
		supported_versions: &["26.1", "26.1.1", "26.1.2"],
	},
	PrimeAuthLibJar {
		filename: "primeauthlib-26.3-1.0.0.jar",
		bytes: include_bytes!("../../resources/primeauthlib/primeauthlib-26.3-1.0.0.jar"),
		supported_versions: &["26.2", "26.3"],
	},
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomSkinLoaderMetadata {
	pub jar_file: String,
	pub version_number: String,
	pub game_versions: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct ModrinthVersionFile {
	url: String,
	filename: String,
	#[serde(default)]
	primary: bool,
}

#[derive(Debug, Deserialize)]
struct ModrinthVersionResponse {
	#[allow(dead_code)]
	id: String,
	version_number: String,
	game_versions: Vec<String>,
	#[allow(dead_code)]
	loaders: Vec<String>,
	files: Vec<ModrinthVersionFile>,
}

/// Validate and sanitize a Minecraft username
pub fn sanitize_username(username: &str) -> crate::Result<String> {
	let trimmed = username.trim();
	if trimmed.is_empty() || trimmed.len() > 32 {
		return Err(crate::ErrorKind::InputError(
			"Username must be between 1 and 32 characters".to_string(),
		)
		.into());
	}
	if !trimmed
		.chars()
		.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
	{
		return Err(crate::ErrorKind::InputError(
			"Username may only contain letters, numbers, underscores, and hyphens".to_string(),
		)
		.into());
	}
	if trimmed.contains("..")
		|| trimmed.contains('/')
		|| trimmed.contains('\\')
		|| trimmed.contains('\0')
	{
		return Err(crate::ErrorKind::InputError(
			"Security: path traversal sequence detected in username".to_string(),
		)
		.into());
	}
	Ok(trimmed.to_string())
}

/// Validate skin image bytes (size and PNG signature)
pub fn validate_skin_bytes(raw_bytes: &[u8]) -> crate::Result<()> {
	const MAX_SKIN_BYTES: usize = 2 * 1024 * 1024;
	if raw_bytes.is_empty() {
		return Err(crate::ErrorKind::InputError("Skin image data cannot be empty".to_string()).into());
	}
	if raw_bytes.len() > MAX_SKIN_BYTES {
		return Err(crate::ErrorKind::InputError("Skin image size exceeds 2 MB limit".to_string()).into());
	}
	const PNG_HEADER: [u8; 8] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
	if raw_bytes.len() < 8 || raw_bytes[0..8] != PNG_HEADER {
		return Err(crate::ErrorKind::InputError("File is not a valid PNG image".to_string()).into());
	}
	Ok(())
}

/// Validate and sanitize authlib-injector auth endpoint URL
pub fn validate_auth_endpoint(endpoint: &str) -> crate::Result<String> {
	let trimmed = endpoint.trim();
	if trimmed.is_empty() {
		return Ok(String::new());
	}
	if !trimmed.starts_with("http://") && !trimmed.starts_with("https://") {
		return Err(crate::ErrorKind::InputError(
			"Auth endpoint must start with http:// or https://".to_string(),
		)
		.into());
	}
	if trimmed.chars().any(|c| {
		c.is_whitespace()
			|| c.is_control()
			|| c == '"'
			|| c == '\''
			|| c == ';'
			|| c == '&'
			|| c == '|'
			|| c == '$'
			|| c == '`'
			|| c == '\\'
	}) {
		return Err(crate::ErrorKind::InputError(
			"Security: auth endpoint contains invalid or unsafe characters".to_string(),
		)
		.into());
	}
	url::Url::parse(trimmed).map_err(|e| {
		crate::ErrorKind::InputError(format!("Invalid auth endpoint URL: {e}"))
	})?;
	Ok(trimmed.to_string())
}

/// Compute standard offline deterministic UUID v3: MD5 of "OfflinePlayer:" + username
pub fn compute_offline_uuid(username: &str) -> uuid::Uuid {
	let digest = md5::compute(format!("OfflinePlayer:{}", username));
	let mut bytes = digest.0;
	bytes[6] = (bytes[6] & 0x0f) | 0x30; // Version 3 (MD5 based)
	bytes[8] = (bytes[8] & 0x3f) | 0x80; // Variant 2 (IETF RFC 4122)
	uuid::Uuid::from_bytes(bytes)
}

/// Resolves `.craftrinth` directory
pub fn get_craftrinth_dir() -> PathBuf {
	if let Some(home) = dirs::home_dir() {
		let path = home.join(".craftrinth");
		if path.exists() {
			return path;
		}
	}
	let local = PathBuf::from(".craftrinth");
	if local.exists() {
		return local;
	}
	dirs::home_dir()
		.map(|h| h.join(".craftrinth"))
		.unwrap_or_else(|| PathBuf::from(".craftrinth"))
}

/// Check for authlib-injector.jar in .craftrinth/launcher/agents/
/// If present, append `-javaagent:"<path_to_agent>"=<configured_auth_endpoint>` to the JVM arguments.
/// If missing, log a warning and proceed with standard local mod rendering.
pub async fn detect_authlib_injector() -> Option<String> {
	let craftrinth = get_craftrinth_dir();
	let agent_path = craftrinth.join("launcher/agents/authlib-injector.jar");
	let fallback_local = PathBuf::from(".craftrinth/launcher/agents/authlib-injector.jar");

	let final_agent_path = if agent_path.is_file() {
		Some(agent_path)
	} else if fallback_local.is_file() {
		Some(fallback_local)
	} else {
		None
	};

	let Some(agent) = final_agent_path else {
		return None;
	};

	let configured_endpoint = std::env::var("AUTHLIB_INJECTOR_ENDPOINT")
		.or_else(|_| std::env::var("CRAFTINTH_AUTH_ENDPOINT"))
		.ok()
		.or_else(|| {
			let endpoint_file = agent.parent().map(|p| p.join("auth_endpoint.txt"));
			if let Some(ef) = endpoint_file
				&& ef.is_file()
			{
				let content = std::fs::read_to_string(&ef).unwrap_or_default().trim().to_string();
				if !content.is_empty() {
					Some(content)
				} else {
					None
				}
			} else {
				None
			}
		});

	// Only inject if an external auth server endpoint is explicitly configured by the user
	if let Some(endpoint) = configured_endpoint {
		if let Ok(safe_endpoint) = validate_auth_endpoint(&endpoint) {
			if !safe_endpoint.is_empty() && safe_endpoint != "https://auth.craftrinth.com" {
				let canonical_agent = dunce::canonicalize(&agent).unwrap_or(agent);
				let arg = format!("-javaagent:{}={}", canonical_agent.display(), safe_endpoint);
				tracing::info!("Appended authlib-injector javaagent with endpoint: {arg}");
				return Some(arg);
			}
		}
	}

	tracing::info!("Authlib-injector not configured with an external server; operating strictly client-side");
	None
}

/// Detects CustomSkinLoader mod in `<instance_path>/mods/` or downloads matching version from Modrinth
pub async fn ensure_custom_skin_loader(
	instance_path: &Path,
	mc_version: &str,
	loader: &str,
) -> crate::Result<()> {
	if loader == "vanilla" {
		tracing::debug!("CustomSkinLoader skipped: instance is Vanilla");
		return Ok(());
	}

	let mods_dir = instance_path.join("mods");
	tokio::fs::create_dir_all(&mods_dir).await.ok();

	let regex = Regex::new(r"(?i).*customskinloader.*\.jar$").map_err(|e| {
		crate::ErrorKind::LauncherError(format!("regex compile error: {e}"))
	})?;

	let meta_file = mods_dir.join(".customskinloader_meta.json");

	let mut existing_jar: Option<PathBuf> = None;
	if let Ok(mut read_dir) = tokio::fs::read_dir(&mods_dir).await {
		while let Ok(Some(entry)) = read_dir.next_entry().await {
			let path = entry.path();
			if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
				if regex.is_match(name) {
					existing_jar = Some(path);
					break;
				}
			}
		}
	}

	let mut needs_download = existing_jar.is_none();
	if existing_jar.is_some() && meta_file.is_file() {
		if let Ok(content) = tokio::fs::read_to_string(&meta_file).await {
			if let Ok(meta) = serde_json::from_str::<CustomSkinLoaderMetadata>(&content) {
				if meta.game_versions.iter().any(|v| v == mc_version) {
					tracing::debug!(
						"Existing CustomSkinLoader ({}) is compatible with Minecraft {}",
						meta.version_number,
						mc_version
					);
					needs_download = false;
				} else {
					tracing::info!(
						"CustomSkinLoader version mismatch (current MC: {}, mod MCs: {:?}); upgrading via Modrinth",
						mc_version,
						meta.game_versions
					);
					needs_download = true;
				}
			}
		}
	}

	if needs_download {
		let client = reqwest::Client::builder()
			.timeout(std::time::Duration::from_secs(10))
			.user_agent(crate::launcher_user_agent())
			.build()
			.unwrap_or_default();

		let loader_param = if loader.is_empty() { "fabric" } else { loader };
		let url = format!(
			"https://api.modrinth.com/v2/project/customskinloader/version?game_versions=%5B%22{}%22%5D&loaders=%5B%22{}%22%5D",
			mc_version, loader_param
		);

		match client.get(&url).send().await {
			Ok(resp) if resp.status().is_success() => {
				match resp.json::<Vec<ModrinthVersionResponse>>().await {
					Ok(versions) if !versions.is_empty() => {
						let selected = &versions[0];
						if let Some(file) = selected
							.files
							.iter()
							.find(|f| f.primary)
							.or_else(|| selected.files.first())
						{
							tracing::info!(
								"Downloading CustomSkinLoader {} ({}) from Modrinth: {}",
								selected.version_number,
								file.filename,
								file.url
							);
							match client.get(&file.url).send().await {
								Ok(file_resp) if file_resp.status().is_success() => {
									if let Ok(bytes) = file_resp.bytes().await {
										if let Some(old_jar) = existing_jar {
											if old_jar.file_name().and_then(|n| n.to_str())
												!= Some(&file.filename)
											{
												tokio::fs::remove_file(old_jar).await.ok();
											}
										}

										let target_path = mods_dir.join(&file.filename);
										tokio::fs::write(&target_path, bytes).await.ok();

										let meta = CustomSkinLoaderMetadata {
											jar_file: file.filename.clone(),
											version_number: selected.version_number.clone(),
											game_versions: selected.game_versions.clone(),
										};
										if let Ok(json) = serde_json::to_string_pretty(&meta) {
											tokio::fs::write(&meta_file, json).await.ok();
										}
										tracing::info!("CustomSkinLoader successfully downloaded and placed in mods/");
									}
								}
								Err(err) => {
									tracing::warn!("Failed to download CustomSkinLoader jar: {err}");
								}
								_ => {}
							}
						}
					}
					_ => {
						tracing::warn!(
							"No compatible CustomSkinLoader version found on Modrinth for MC {mc_version} with loader {loader_param}"
						);
					}
				}
			}
			Err(err) => {
				if let Some(ref jar) = existing_jar {
					tracing::warn!(
						"Network dropout when querying Modrinth for CustomSkinLoader ({err}). Using existing cached jar: {}",
						jar.display()
					);
				} else {
					tracing::warn!(
						"Failed to query Modrinth for CustomSkinLoader ({err}) and no cached jar found."
					);
				}
			}
			_ => {}
		}
	}

	Ok(())
}

/// Local skin sync and CustomSkinLoader.json provisioning
pub async fn provision_custom_skin_loader_config(
	instance_path: &Path,
	username: &str,
) -> crate::Result<()> {
	let clean_username = sanitize_username(username)?;
	let csl_dir = instance_path.join("CustomSkinLoader");
	let csl_skins = csl_dir.join("skins");
	let csl_local_skins = csl_dir.join("LocalSkin").join("skins");
	let direct_local_skins = instance_path.join("LocalSkin").join("skins");
	let direct_skins = instance_path.join("skins");

	tokio::fs::create_dir_all(&csl_skins).await.ok();
	tokio::fs::create_dir_all(&csl_local_skins).await.ok();
	tokio::fs::create_dir_all(&direct_local_skins).await.ok();
	tokio::fs::create_dir_all(&direct_skins).await.ok();

	let craftrinth = get_craftrinth_dir();
	let lower_name = clean_username.to_lowercase();
	let candidates = [
		craftrinth.join("launcher/user/skin").join(format!("{clean_username}.png")),
		craftrinth.join("launcher/user/skin").join(format!("{lower_name}.png")),
		PathBuf::from(".craftrinth/launcher/user/skin").join(format!("{clean_username}.png")),
		PathBuf::from(".craftrinth/launcher/user/skin").join(format!("{lower_name}.png")),
	];

	let actual_skin_source = candidates.into_iter().find(|p| p.is_file());

	if let Some(src) = actual_skin_source {
		if let Ok(bytes) = tokio::fs::read(&src).await {
			let targets = [
				csl_skins.join(format!("{clean_username}.png")),
				csl_skins.join(format!("{lower_name}.png")),
				csl_local_skins.join(format!("{clean_username}.png")),
				csl_local_skins.join(format!("{lower_name}.png")),
				direct_local_skins.join(format!("{clean_username}.png")),
				direct_local_skins.join(format!("{lower_name}.png")),
				direct_skins.join(format!("{clean_username}.png")),
				direct_skins.join(format!("{lower_name}.png")),
			];
			for dest in targets {
				tokio::fs::write(&dest, &bytes).await.ok();
			}
			tracing::info!("Synchronized local custom skin for offline user '{clean_username}' across all CSL destinations");
		}
	} else {
		tracing::debug!(
			"No custom skin assigned for offline user '{clean_username}'; falling back to default model"
		);
	}

	let config_path = csl_dir.join("CustomSkinLoader.json");
	let config_content = r#"{
  "version": "14.12",
  "enable": true,
  "loadlist": [
    {
      "name": "LocalSkin",
      "type": "Legacy",
      "checkPNG": false,
      "skin": "CustomSkinLoader/LocalSkin/skins/{USERNAME}.png",
      "cape": "CustomSkinLoader/LocalSkin/capes/{USERNAME}.png",
      "model": "auto"
    },
    {
      "name": "LocalSkinRoot",
      "type": "Legacy",
      "checkPNG": false,
      "skin": "CustomSkinLoader/skins/{USERNAME}.png",
      "cape": "CustomSkinLoader/capes/{USERNAME}.png",
      "model": "auto"
    },
    {
      "name": "LocalSkinRelative",
      "type": "Legacy",
      "checkPNG": false,
      "skin": "LocalSkin/skins/{USERNAME}.png",
      "cape": "LocalSkin/capes/{USERNAME}.png",
      "model": "auto"
    },
    {
      "name": "LocalSkinDirect",
      "type": "Legacy",
      "checkPNG": false,
      "skin": "skins/{USERNAME}.png",
      "cape": "capes/{USERNAME}.png",
      "model": "auto"
    },
    {
      "name": "MojangAPI",
      "type": "MojangAPI"
    }
  ],
  "enableDynamicSkull": true,
  "enableDynamicTextures": true,
  "cacheExpiry": 0
}"#;

	let needs_write = if config_path.is_file() {
		match tokio::fs::read_to_string(&config_path).await {
			Ok(existing) => !existing.contains("\"LocalSkin\"") || existing.contains("\"LocalOnly\"") || existing.contains("\"type\": \"Local\""),
			Err(_) => true,
		}
	} else {
		true
	};

	if needs_write {
		if let Err(e) = tokio::fs::write(&config_path, config_content).await {
			tracing::warn!("Failed to write CustomSkinLoader.json: {e}");
		} else {
			tracing::info!("Provisioned CustomSkinLoader.json for local resolution");
		}
	}

	Ok(())
}

/// Ensures PrimeAuthLib is installed in <instance_path>/mods for Fabric instances
pub async fn ensure_primeauthlib(
	instance_path: &Path,
	mc_version: &str,
) -> crate::Result<()> {
	let mods_dir = instance_path.join("mods");
	tokio::fs::create_dir_all(&mods_dir).await.ok();

	let pal_regex = Regex::new(r"(?i).*primeauthlib.*\.jar$").map_err(|e| {
		crate::ErrorKind::LauncherError(format!("regex compile error: {e}"))
	})?;

	let mut existing_jar: Option<PathBuf> = None;
	if let Ok(mut read_dir) = tokio::fs::read_dir(&mods_dir).await {
		while let Ok(Some(entry)) = read_dir.next_entry().await {
			let path = entry.path();
			if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
				if pal_regex.is_match(name) {
					existing_jar = Some(path);
					break;
				}
			}
		}
	}

	// Remove any CustomSkinLoader jar in Fabric instance to prevent conflicts
	if let Ok(csl_regex) = Regex::new(r"(?i).*customskinloader.*\.jar$") {
		if let Ok(mut read_dir) = tokio::fs::read_dir(&mods_dir).await {
			while let Ok(Some(entry)) = read_dir.next_entry().await {
				let path = entry.path();
				if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
					if csl_regex.is_match(name) {
						tracing::info!("Removing CustomSkinLoader in favor of PrimeAuthLib: {}", path.display());
						tokio::fs::remove_file(&path).await.ok();
					}
				}
			}
		}
	}

	let best_bundled = BUNDLED_PRIMEAUTHLIB_JARS
		.iter()
		.find(|j| j.supported_versions.contains(&mc_version))
		.or_else(|| {
			if mc_version.starts_with("1.21.1") || mc_version == "1.21" || mc_version == "1.21.0" {
				Some(&BUNDLED_PRIMEAUTHLIB_JARS[0])
			} else if mc_version.starts_with("1.21") {
				Some(&BUNDLED_PRIMEAUTHLIB_JARS[1])
			} else if mc_version.starts_with("26.1") {
				Some(&BUNDLED_PRIMEAUTHLIB_JARS[2])
			} else if mc_version.starts_with("26.") {
				Some(&BUNDLED_PRIMEAUTHLIB_JARS[3])
			} else {
				BUNDLED_PRIMEAUTHLIB_JARS.first()
			}
		});

	if let Some(bundled) = best_bundled {
		let target_file = mods_dir.join(bundled.filename);
		if !target_file.is_file() {
			if let Some(old) = existing_jar {
				if old != target_file {
					tokio::fs::remove_file(old).await.ok();
				}
			}
			if let Err(e) = tokio::fs::write(&target_file, bundled.bytes).await {
				tracing::warn!("Failed to install bundled PrimeAuthLib: {e}");
			} else {
				tracing::info!("Installed PrimeAuthLib ({}) for Minecraft {mc_version}", bundled.filename);
			}
		} else {
			tracing::debug!("PrimeAuthLib ({}) is present in mods/", bundled.filename);
		}
	}

	// Background check for online release updates from CrafterStudioIndia/primeauthlib
	tokio::spawn({
		let mods_dir = mods_dir.clone();
		let mc_version = mc_version.to_string();
		async move {
			let client = reqwest::Client::builder()
				.timeout(std::time::Duration::from_secs(4))
				.user_agent("Craftrinth-Launcher")
				.build()
				.unwrap_or_default();
			let url = "https://api.github.com/repos/CrafterStudioIndia/primeauthlib/releases";
			if let Ok(resp) = client.get(url).send().await {
				if resp.status().is_success() {
					if let Ok(releases) = resp.json::<serde_json::Value>().await {
						if let Some(arr) = releases.as_array() {
							for rel in arr {
								if let Some(assets) = rel.get("assets").and_then(|a| a.as_array()) {
									for asset in assets {
										if let (Some(name), Some(durl)) = (
											asset.get("name").and_then(|n| n.as_str()),
											asset.get("browser_download_url").and_then(|u| u.as_str()),
										) {
											if name.contains(&mc_version) && name.ends_with(".jar") && !name.contains("sources") {
												let dest = mods_dir.join(name);
												if !dest.is_file() {
													if let Ok(dl_resp) = client.get(durl).send().await {
														if dl_resp.status().is_success() {
															if let Ok(bytes) = dl_resp.bytes().await {
																let _ = tokio::fs::write(&dest, bytes).await;
																tracing::info!("Updated PrimeAuthLib to latest release: {name}");
															}
														}
													}
												}
												return;
											}
										}
									}
								}
							}
						}
					}
				}
			}
		}
	});

	Ok(())
}

/// Provisions the auth-libskins/ folder for PrimeAuthLib
pub async fn provision_primeauthlib_skins(
	instance_path: &Path,
	username: &str,
) -> crate::Result<()> {
	let clean_username = sanitize_username(username)?;
	let skins_dir = instance_path.join("auth-libskins");
	tokio::fs::create_dir_all(&skins_dir).await.ok();

	let craftrinth = get_craftrinth_dir();
	let lower_name = clean_username.to_lowercase();
	let candidates = [
		craftrinth.join("launcher/user/skin").join(format!("{clean_username}.png")),
		craftrinth.join("launcher/user/skin").join(format!("{lower_name}.png")),
		PathBuf::from(".craftrinth/launcher/user/skin").join(format!("{clean_username}.png")),
		PathBuf::from(".craftrinth/launcher/user/skin").join(format!("{lower_name}.png")),
	];

	let actual_skin_source = candidates.into_iter().find(|p| p.is_file());

	if let Some(src) = actual_skin_source {
		if let Ok(bytes) = tokio::fs::read(&src).await {
			let offline_uuid = compute_offline_uuid(&clean_username);
			let compact_uuid = offline_uuid.simple().to_string();
			let hyphens_uuid = offline_uuid.hyphenated().to_string();

			let targets = [
				skins_dir.join("skin.png"),
				skins_dir.join(format!("{clean_username}.png")),
				skins_dir.join(format!("{lower_name}.png")),
				skins_dir.join(format!("{compact_uuid}.png")),
				skins_dir.join(format!("{hyphens_uuid}.png")),
			];
			for dest in targets {
				tokio::fs::write(&dest, &bytes).await.ok();
			}
			tracing::info!("Synchronized custom skin for '{clean_username}' to PrimeAuthLib auth-libskins/");
		}
	}

	let config_path = skins_dir.join("config.properties");
	if !config_path.is_file() {
		let default_config = format!(
			"# PrimeAuthLib Configuration\nmodel_mode=AUTO\nselected_skin=skin.png\n"
		);
		tokio::fs::write(&config_path, default_config).await.ok();
	}

	let readme_path = skins_dir.join("README.txt");
	if !readme_path.is_file() {
		let readme = format!(
			"=====================================================\n  PrimeAuthLib Custom Skins Folder\n=====================================================\n\nPrimeAuthLib loads skin textures directly from this directory.\nNo external auth servers, java agents, or complicated setup needed!\n\nHOW TO USE:\n1. Put any Minecraft skin (.png) in this folder.\n2. Name it one of the following:\n   - 'skin.png'     -> Applies to your player by default\n   - '<PlayerName>.png' -> Applies to that specific player\n   - '<UUID>.png'   -> Applies to that player's UUID\n   - 'default.png'  -> Fallback skin\n3. Use the in-game 'Auth-Lib Skin...' menu to switch skins or preview in 3D.\n4. Model arms (Slim 3px vs Wide 4px) are AUTO-DETECTED automatically!\n"
		);
		tokio::fs::write(&readme_path, readme).await.ok();
	}

	Ok(())
}

/// Orchestrates the entire dual-layer skin pipeline pre-launch
pub async fn orchestrate_skin_pipeline(
	instance_path: &Path,
	mc_version: &str,
	loader: &str,
	credentials: &crate::state::Credentials,
	extra_jvm_args: &mut Vec<String>,
) -> crate::Result<()> {
	if let Some(agent_arg) = detect_authlib_injector().await {
		extra_jvm_args.push(agent_arg);
	}

	let username = &credentials.offline_profile.name;

	if loader.eq_ignore_ascii_case("fabric") || loader.eq_ignore_ascii_case("quilt") {
		let _ = ensure_primeauthlib(instance_path, mc_version).await;
		let _ = provision_primeauthlib_skins(instance_path, username).await;
	} else if loader != "vanilla" {
		let _ = ensure_custom_skin_loader(instance_path, mc_version, loader).await;
	}

	let _ = provision_custom_skin_loader_config(instance_path, username).await;

	Ok(())
}

/// Retrieves cracked and offline engine status
pub async fn get_cracked_status() -> crate::Result<CrackedStatus> {
	let craftrinth = get_craftrinth_dir();
	let agent_path = craftrinth.join("launcher/agents/authlib-injector.jar");
	let authlib_present = agent_path.is_file();

	let endpoint = std::env::var("AUTHLIB_INJECTOR_ENDPOINT")
		.or_else(|_| std::env::var("CRAFTINTH_AUTH_ENDPOINT"))
		.unwrap_or_else(|_| {
			let endpoint_file = agent_path.parent().map(|p| p.join("auth_endpoint.txt"));
			if let Some(ef) = endpoint_file
				&& ef.is_file()
			{
				std::fs::read_to_string(&ef).unwrap_or_default().trim().to_string()
			} else {
				String::new()
			}
		});

	let skins_dir = craftrinth.join("launcher/user/skin");
	let mut skins = Vec::new();
	if skins_dir.is_dir() {
		if let Ok(mut read_dir) = tokio::fs::read_dir(&skins_dir).await {
			while let Ok(Some(entry)) = read_dir.next_entry().await {
				let path = entry.path();
				if path.extension().and_then(|e| e.to_str()) == Some("png") {
					let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or_default().to_string();
					let filename = path.file_name().and_then(|f| f.to_str()).unwrap_or_default().to_string();
					let modified = entry.metadata().await
						.and_then(|m| m.modified())
						.ok()
						.and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
						.map(|d| d.as_secs())
						.unwrap_or(0);
					skins.push(CrackedSkinItem {
						username: stem,
						filename,
						path: path.to_string_lossy().to_string(),
						modified,
					});
				}
			}
		}
	}

	Ok(CrackedStatus {
		authlib_present,
		authlib_path: agent_path.to_string_lossy().to_string(),
		auth_endpoint: endpoint,
		craftrinth_dir: craftrinth.to_string_lossy().to_string(),
		skins,
		primeauthlib_present: true,
	})
}

/// Crops the 8x8 head + hat layer and upscales to 64x64 PNG bytes
pub fn extract_head_png_bytes(skin_png_bytes: &[u8]) -> crate::Result<Vec<u8>> {
	let img = image::load_from_memory(skin_png_bytes)
		.map_err(|e| crate::ErrorKind::LauncherError(format!("Failed to parse skin image: {e}")))?
		.to_rgba8();

	if img.width() < 64 || img.height() < 32 {
		return Err(crate::ErrorKind::LauncherError("Skin image dimensions too small".to_string()).into());
	}

	let mut head = image::RgbaImage::new(8, 8);
	for y in 0..8 {
		for x in 0..8 {
			let pixel = img.get_pixel(8 + x, 8 + y);
			head.put_pixel(x, y, *pixel);
		}
	}

	for y in 0..8 {
		for x in 0..8 {
			let hat_pixel = img.get_pixel(40 + x, 8 + y);
			if hat_pixel[3] > 0 {
				head.put_pixel(x, y, *hat_pixel);
			}
		}
	}

	let upscaled = image::imageops::resize(&head, 64, 64, image::imageops::FilterType::Nearest);

	let mut out_bytes = Vec::new();
	let mut cursor = std::io::Cursor::new(&mut out_bytes);
	upscaled
		.write_to(&mut cursor, image::ImageFormat::Png)
		.map_err(|e| crate::ErrorKind::LauncherError(format!("Failed to encode head PNG: {e}")))?;

	Ok(out_bytes)
}

/// Saves skin PNG bytes for an offline user
pub async fn save_cracked_skin(username: &str, raw_bytes: &[u8]) -> crate::Result<()> {
	let clean_username = sanitize_username(username)?;
	validate_skin_bytes(raw_bytes)?;

	let craftrinth = get_craftrinth_dir();
	let skins_dir = craftrinth.join("launcher/user/skin");
	tokio::fs::create_dir_all(&skins_dir).await.ok();
	let skin_path = skins_dir.join(format!("{clean_username}.png"));
	tokio::fs::write(&skin_path, raw_bytes).await.map_err(|e| {
		crate::ErrorKind::LauncherError(format!("Failed to write skin file: {e}"))
	})?;
	let lower_path = skins_dir.join(format!("{}.png", clean_username.to_lowercase()));
	tokio::fs::write(&lower_path, raw_bytes).await.ok();

	if let Ok(head_bytes) = extract_head_png_bytes(raw_bytes) {
		let head_path = skins_dir.join(format!("{clean_username}_head.png"));
		let _ = tokio::fs::write(&head_path, &head_bytes).await;

		let username_clone = clean_username.clone();
		let skins_dir_clone = skins_dir.clone();
		tokio::spawn(async move {
			let form = reqwest::multipart::Form::new()
				.text("reqtype", "fileupload")
				.part(
					"fileToUpload",
					reqwest::multipart::Part::bytes(head_bytes)
						.file_name("head.png")
						.mime_str("image/png")
						.unwrap(),
				);
			if let Ok(resp) = crate::util::fetch::INSECURE_REQWEST_CLIENT
				.post("https://catbox.moe/user/api.php")
				.multipart(form)
				.send()
				.await
			{
				if resp.status().is_success() {
					if let Ok(url) = resp.text().await {
						let clean_url = url.trim();
						if clean_url.starts_with("https://") {
							let url_path = skins_dir_clone.join(format!("{username_clone}_head.url"));
							let _ = tokio::fs::write(&url_path, clean_url.as_bytes()).await;
						}
					}
				}
			}
		});
	}

	tracing::info!("Saved custom skin for offline user '{clean_username}' to {}", skin_path.display());
	let _ = sync_cracked_skins_to_all_instances().await;
	Ok(())
}

/// Saves skin from base64 data URL, raw base64, or remote texture URL
pub async fn save_cracked_skin_from_str(username: &str, data_or_url: &str) -> crate::Result<()> {
	let trimmed = data_or_url.trim();
	let bytes = if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
		let resp = crate::util::fetch::INSECURE_REQWEST_CLIENT
			.get(trimmed)
			.send()
			.await
			.map_err(|e| crate::ErrorKind::LauncherError(format!("Failed to fetch skin texture URL: {e}")))?;
		resp.bytes()
			.await
			.map_err(|e| crate::ErrorKind::LauncherError(format!("Failed to read skin bytes: {e}")))?
			.to_vec()
	} else {
		let clean_base64 = if let Some(idx) = trimmed.find("base64,") {
			&trimmed[idx + 7..]
		} else {
			trimmed
		};
		BASE64_STANDARD
			.decode(clean_base64)
			.map_err(|e| crate::ErrorKind::LauncherError(format!("Invalid base64 skin data: {e}")))?
	};

	save_cracked_skin(username, &bytes).await
}

/// Reads skin PNG bytes for an offline user and returns data URL
pub async fn get_cracked_skin(username: &str) -> crate::Result<Option<String>> {
	let clean_username = sanitize_username(username)?;
	let craftrinth = get_craftrinth_dir();
	let lower_name = clean_username.to_lowercase();
	let candidates = [
		craftrinth.join("launcher/user/skin").join(format!("{clean_username}.png")),
		craftrinth.join("launcher/user/skin").join(format!("{lower_name}.png")),
		PathBuf::from(".craftrinth/launcher/user/skin").join(format!("{clean_username}.png")),
		PathBuf::from(".craftrinth/launcher/user/skin").join(format!("{lower_name}.png")),
	];

	let actual = candidates.into_iter().find(|p| p.is_file());

	if let Some(path) = actual {
		let bytes = tokio::fs::read(&path).await.map_err(|e| {
			crate::ErrorKind::LauncherError(format!("Failed to read skin file: {e}"))
		})?;
		let b64 = BASE64_STANDARD.encode(&bytes);
		Ok(Some(format!("data:image/png;base64,{b64}")))
	} else {
		Ok(None)
	}
}

/// Deletes custom skin for an offline user
pub async fn delete_cracked_skin(username: &str) -> crate::Result<()> {
	let clean_username = sanitize_username(username)?;
	let craftrinth = get_craftrinth_dir();
	let skin_path = craftrinth.join("launcher/user/skin").join(format!("{clean_username}.png"));
	if skin_path.is_file() {
		tokio::fs::remove_file(&skin_path).await.ok();
	}
	let lower_path = craftrinth.join("launcher/user/skin").join(format!("{}.png", clean_username.to_lowercase()));
	if lower_path.is_file() {
		tokio::fs::remove_file(&lower_path).await.ok();
	}
	Ok(())
}

/// Sets the authlib-injector auth endpoint URL
pub async fn set_authlib_endpoint(endpoint: &str) -> crate::Result<()> {
	let clean_endpoint = validate_auth_endpoint(endpoint)?;
	let craftrinth = get_craftrinth_dir();
	let agents_dir = craftrinth.join("launcher/agents");
	tokio::fs::create_dir_all(&agents_dir).await.ok();
	let endpoint_file = agents_dir.join("auth_endpoint.txt");
	tokio::fs::write(&endpoint_file, clean_endpoint.trim()).await.map_err(|e| {
		crate::ErrorKind::LauncherError(format!("Failed to write auth endpoint file: {e}"))
	})?;
	tracing::info!("Set authlib-injector endpoint to: {clean_endpoint}");
	Ok(())
}

/// Downloads latest authlib-injector.jar
pub async fn download_authlib_injector() -> crate::Result<()> {
	let craftrinth = get_craftrinth_dir();
	let agents_dir = craftrinth.join("launcher/agents");
	tokio::fs::create_dir_all(&agents_dir).await.ok();
	let agent_path = agents_dir.join("authlib-injector.jar");

	let client = reqwest::Client::builder()
		.timeout(std::time::Duration::from_secs(30))
		.user_agent(crate::launcher_user_agent())
		.build()
		.unwrap_or_default();

	let url = "https://authlib-injector.yggdrasil.co/artifact/latest.jar";
	let resp = client.get(url).send().await.map_err(|e| {
		crate::ErrorKind::LauncherError(format!("Failed to request authlib-injector: {e}"))
	})?;

	if !resp.status().is_success() {
		return Err(crate::ErrorKind::LauncherError(format!(
			"Failed to download authlib-injector: HTTP {}",
			resp.status()
		)).into());
	}

	let bytes = resp.bytes().await.map_err(|e| {
		crate::ErrorKind::LauncherError(format!("Failed to read authlib-injector response: {e}"))
	})?;

	tokio::fs::write(&agent_path, bytes).await.map_err(|e| {
		crate::ErrorKind::LauncherError(format!("Failed to save authlib-injector.jar: {e}"))
	})?;

	tracing::info!("Successfully downloaded authlib-injector.jar to {}", agent_path.display());
	Ok(())
}

/// Syncs cracked skins to all instance directories
pub async fn sync_cracked_skins_to_all_instances() -> crate::Result<usize> {
	let state = crate::State::get().await?;
	let instances = state.directories.instances_dir();
	let mut count = 0;

	if let Ok(mut read_dir) = tokio::fs::read_dir(&instances).await {
		while let Ok(Some(entry)) = read_dir.next_entry().await {
			let path = entry.path();
			if path.is_dir() {
				let craftrinth = get_craftrinth_dir();
				let skins_dir = craftrinth.join("launcher/user/skin");
				if skins_dir.is_dir() {
					if let Ok(mut skin_read) = tokio::fs::read_dir(&skins_dir).await {
						while let Ok(Some(skin_entry)) = skin_read.next_entry().await {
							let skin_path = skin_entry.path();
							if let Some(username) = skin_path.file_stem().and_then(|s| s.to_str()) {
								let _ = provision_primeauthlib_skins(&path, username).await;
								let _ = provision_custom_skin_loader_config(&path, username).await;
							}
						}
					}
				}
				count += 1;
			}
		}
	}

	Ok(count)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchedSkinResult {
	pub username: String,
	pub base64_png: String,
	pub model: String,
}

pub async fn fetch_skin_by_username(username: &str) -> crate::Result<FetchedSkinResult> {
	let clean_username = sanitize_username(username)?;

	let profile_url = format!("https://api.mojang.com/users/profiles/minecraft/{}", clean_username);
	let profile_resp = crate::util::fetch::INSECURE_REQWEST_CLIENT
		.get(&profile_url)
		.send()
		.await
		.map_err(|e| crate::ErrorKind::LauncherError(format!("Failed to connect to Mojang API: {e}")))?;

	if !profile_resp.status().is_success() {
		return Err(crate::ErrorKind::LauncherError(format!(
			"Player '{}' not found on Mojang (status {})",
			clean_username,
			profile_resp.status()
		)).into());
	}

	#[derive(Deserialize)]
	struct MojangProfileResp {
		id: String,
		name: String,
	}

	let profile_data: MojangProfileResp = profile_resp
		.json()
		.await
		.map_err(|e| crate::ErrorKind::LauncherError(format!("Failed to parse Mojang profile: {e}")))?;

	let session_url = format!(
		"https://sessionserver.mojang.com/session/minecraft/profile/{}",
		profile_data.id
	);
	let session_resp = crate::util::fetch::INSECURE_REQWEST_CLIENT
		.get(&session_url)
		.send()
		.await
		.map_err(|e| crate::ErrorKind::LauncherError(format!("Failed to connect to Mojang session server: {e}")))?;

	if !session_resp.status().is_success() {
		return Err(crate::ErrorKind::LauncherError(format!(
			"Failed to retrieve profile textures for '{}' (status {})",
			clean_username,
			session_resp.status()
		)).into());
	}

	#[derive(Deserialize)]
	struct SessionProp {
		name: String,
		value: String,
	}

	#[derive(Deserialize)]
	struct SessionResp {
		properties: Vec<SessionProp>,
	}

	let session_data: SessionResp = session_resp
		.json()
		.await
		.map_err(|e| crate::ErrorKind::LauncherError(format!("Failed to parse session profile: {e}")))?;

	let textures_prop = session_data
		.properties
		.into_iter()
		.find(|p| p.name == "textures")
		.ok_or_else(|| crate::ErrorKind::LauncherError("Player has no textures associated with profile".to_string()))?;

	let decoded_bytes = BASE64_STANDARD
		.decode(textures_prop.value.as_bytes())
		.map_err(|e| crate::ErrorKind::LauncherError(format!("Failed to base64-decode textures payload: {e}")))?;

	let decoded_json_str = std::str::from_utf8(&decoded_bytes)
		.map_err(|e| crate::ErrorKind::LauncherError(format!("Invalid UTF-8 in textures payload: {e}")))?;

	#[derive(Deserialize)]
	struct SkinMeta {
		model: Option<String>,
	}

	#[derive(Deserialize)]
	struct SkinData {
		url: String,
		metadata: Option<SkinMeta>,
	}

	#[derive(Deserialize)]
	struct TexturesData {
		#[serde(rename = "SKIN")]
		skin: Option<SkinData>,
	}

	#[derive(Deserialize)]
	struct TexturesPayload {
		textures: TexturesData,
	}

	let textures_payload: TexturesPayload = serde_json::from_str(decoded_json_str)
		.map_err(|e| crate::ErrorKind::LauncherError(format!("Failed to parse textures payload: {e}")))?;

	let skin_data = textures_payload
		.textures
		.skin
		.ok_or_else(|| crate::ErrorKind::LauncherError(format!("Player '{}' does not have a custom skin", profile_data.name)))?;

	if !skin_data.url.starts_with("https://textures.minecraft.net/texture/")
		&& !skin_data.url.starts_with("http://textures.minecraft.net/texture/")
	{
		return Err(crate::ErrorKind::LauncherError("Suspicious texture URL: must point to textures.minecraft.net".to_string()).into());
	}

	let skin_png_resp = crate::util::fetch::INSECURE_REQWEST_CLIENT
		.get(&skin_data.url)
		.send()
		.await
		.map_err(|e| crate::ErrorKind::LauncherError(format!("Failed to download skin PNG: {e}")))?;

	let skin_png_bytes = skin_png_resp
		.bytes()
		.await
		.map_err(|e| crate::ErrorKind::LauncherError(format!("Failed to read skin PNG bytes: {e}")))?;

	validate_skin_bytes(&skin_png_bytes)?;

	let model = skin_data
		.metadata
		.and_then(|m| m.model)
		.unwrap_or_else(|| "classic".to_string());

	let base64_png = BASE64_STANDARD.encode(&skin_png_bytes);

	Ok(FetchedSkinResult {
		username: profile_data.name,
		base64_png,
		model,
	})
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_offline_uuid_steve() {
		let uuid = compute_offline_uuid("Steve");
		assert_eq!(uuid.to_string(), "5627dd98-e6be-3c21-b8a8-e92344183641");
	}

	#[test]
	fn test_offline_uuid_alex() {
		let uuid = compute_offline_uuid("Alex");
		assert_eq!(uuid.to_string(), "36532b5e-c442-3dbb-a24c-c7e55d0f979a");
	}

	#[test]
	fn test_sanitize_username_valid() {
		assert_eq!(sanitize_username("Crafter_123").unwrap(), "Crafter_123");
		assert_eq!(sanitize_username("Player-One").unwrap(), "Player-One");
	}

	#[test]
	fn test_sanitize_username_path_traversal() {
		assert!(sanitize_username("../../../etc/passwd").is_err());
		assert!(sanitize_username("foo/bar").is_err());
		assert!(sanitize_username("foo\\bar").is_err());
		assert!(sanitize_username("evil..name").is_err());
		assert!(sanitize_username("").is_err());
		assert!(sanitize_username("this_username_is_way_too_long_for_minecraft_limits").is_err());
	}

	#[test]
	fn test_validate_auth_endpoint_valid() {
		assert_eq!(
			validate_auth_endpoint("https://auth.example.com").unwrap(),
			"https://auth.example.com"
		);
		assert_eq!(validate_auth_endpoint("").unwrap(), "");
	}

	#[test]
	fn test_validate_auth_endpoint_injection() {
		assert!(validate_auth_endpoint("https://auth.example.com -Dbad=true").is_err());
		assert!(validate_auth_endpoint("https://auth.example.com; rm -rf /").is_err());
		assert!(validate_auth_endpoint("https://auth.example.com\" && bad").is_err());
		assert!(validate_auth_endpoint("ftp://invalid-scheme.com").is_err());
	}

	#[test]
	fn test_validate_skin_bytes() {
		let valid_png = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00];
		assert!(validate_skin_bytes(&valid_png).is_ok());
		let invalid_bytes = [0x00, 0x01, 0x02, 0x03];
		assert!(validate_skin_bytes(&invalid_bytes).is_err());
		assert!(validate_skin_bytes(&[]).is_err());
	}
}

