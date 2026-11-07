/**
 * All theseus API calls return serialized values (both return values and errors);
 * So, for example, addDefaultInstance creates a blank instance object, where the Rust struct is serialized,
 *  and deserialized into a usable JS object.
 */
import { invoke } from '@tauri-apps/api/core'

// Example function:
// User goes to auth_url to complete flow, and when completed, authenticate_await_completion() returns the credentials
// export async function authenticate() {
//   const auth_url = await authenticate_begin_flow()
//   console.log(auth_url)
//   await authenticate_await_completion()
// }

/**
 * Check if the authentication servers are reachable, throwing an exception if
 * not reachable.
 */
export async function check_reachable() {
	await invoke('plugin:auth|check_reachable')
}

/**
 * Authenticate a user with Hydra - part 1.
 * This begins the authentication flow quasi-synchronously.
 *
 * @returns {Promise<DeviceLoginSuccess>} A DeviceLoginSuccess object with two relevant fields:
 * @property {string} verification_uri - The URL to go to complete the flow.
 * @property {string} user_code - The code to enter on the verification_uri page.
 */
export async function login() {
	return await invoke('plugin:auth|login')
}

/**
 * Creates an offline user profile
 * @param {string} username
 * @returns {Promise<Credential>}
 */
export async function add_offline_user(username) {
	return await invoke('plugin:auth|add_offline_user', { username })
}

/**
 * Retrieves the default user
 * @return {Promise<UUID | undefined>}
 */
export async function get_default_user() {
	return await invoke('plugin:auth|get_default_user')
}

/**
 * Updates the default user
 * @param {UUID} user
 */
export async function set_default_user(user) {
	return await invoke('plugin:auth|set_default_user', { user })
}

/**
 * Remove a user account from the database
 * @param {UUID} user
 */
export async function remove_user(user) {
	return await invoke('plugin:auth|remove_user', { user })
}

/**
 * Returns a list of users
 * @returns {Promise<Credential[]>}
 */
export async function users() {
	return await invoke('plugin:auth|get_users')
}

/**
 * Retrieves the cracked and offline engine status
 * @returns {Promise<{ authlib_present: boolean, authlib_path: string, auth_endpoint: string, craftrinth_dir: string, skins: Array<{ username: string, filename: string, path: string, modified: number }> }>}
 */
export async function get_cracked_status() {
	return await invoke('plugin:auth|get_cracked_status')
}

/**
 * Saves a skin PNG image for an offline user
 * @param {string} username
 * @param {string} base64Png
 */
export async function save_cracked_skin(username, base64Png) {
	return await invoke('plugin:auth|save_cracked_skin', { username, base64Png })
}

/**
 * Retrieves the skin PNG data URL for an offline user
 * @param {string} username
 * @returns {Promise<string | null>}
 */
export async function get_cracked_skin(username) {
	return await invoke('plugin:auth|get_cracked_skin', { username })
}

/**
 * Deletes custom skin for an offline user
 * @param {string} username
 */
export async function delete_cracked_skin(username) {
	return await invoke('plugin:auth|delete_cracked_skin', { username })
}

/**
 * Sets the authlib-injector server endpoint
 * @param {string} endpoint
 */
export async function set_authlib_endpoint(endpoint) {
	return await invoke('plugin:auth|set_authlib_endpoint', { endpoint })
}

/**
 * Downloads the latest authlib-injector.jar
 */
export async function download_authlib_injector() {
	return await invoke('plugin:auth|download_authlib_injector')
}

/**
 * Synchronizes skins across all instance directories
 * @returns {Promise<number>}
 */
export async function sync_cracked_skins_to_instances() {
	return await invoke('plugin:auth|sync_cracked_skins_to_instances')
}

/**
 * Computes deterministic offline UUID v3 for a username
 * @param {string} username
 * @returns {Promise<string>}
 */
export async function compute_offline_uuid(username) {
	return await invoke('plugin:auth|compute_offline_uuid', { username })
}

/**
 * Fetches skin from Mojang by player username
 * @param {string} username
 * @returns {Promise<{ username: string, base64_png: string, model: string }>}
 */
export async function fetch_skin_by_username(username) {
	return await invoke('plugin:auth|fetch_skin_by_username', { username })
}
