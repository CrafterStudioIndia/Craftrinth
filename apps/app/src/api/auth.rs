use crate::api::Result;
use anyhow::Context;
use chrono::{Duration, Utc};
use tauri::plugin::TauriPlugin;
use tauri::{Manager, Runtime, UserAttentionType};
use theseus::prelude::*;

pub fn init<R: Runtime>() -> TauriPlugin<R> {
	tauri::plugin::Builder::<R>::new("auth")
		.invoke_handler(tauri::generate_handler![
			check_reachable,
			login,
			add_offline_user,
			remove_user,
			get_default_user,
			set_default_user,
			get_users,
			get_cracked_status,
			save_cracked_skin,
			get_cracked_skin,
			delete_cracked_skin,
			set_authlib_endpoint,
			download_authlib_injector,
			sync_cracked_skins_to_instances,
			compute_offline_uuid,
			fetch_skin_by_username,
		])
		.build()
}

/// Checks if the authentication servers are reachable.
#[tauri::command]
pub async fn check_reachable() -> Result<()> {
    minecraft_auth::check_reachable().await?;
    Ok(())
}

/// Authenticate a user with Hydra - part 1
/// This begins the authentication flow quasi-synchronously, returning a URL to visit (that the user will sign in at)
#[tauri::command]
pub async fn login<R: Runtime>(
    app: tauri::AppHandle<R>,
) -> Result<Option<Credentials>> {
    let flow = minecraft_auth::begin_login().await?;

    let start = Utc::now();

    if let Some(window) = app.get_webview_window("signin") {
        window.close()?;
    }

    let window = tauri::WebviewWindowBuilder::new(
        &app,
        "signin",
        tauri::WebviewUrl::External(
            flow.auth_request_uri
                .parse()
                .context("error parsing auth redirect URL")?,
        ),
    )
    .title("Sign into Modrinth")
    .always_on_top(true)
    .min_inner_size(500.0, 500.0)
    .inner_size(1000.0, 700.0)
    .focused(true)
    .center()
    .build()?;

    window.request_user_attention(Some(UserAttentionType::Critical))?;

    while (Utc::now() - start) < Duration::minutes(10) {
        if window.title().is_err() {
            // user closed window, cancelling flow
            return Ok(None);
        }

        if window
            .url()?
            .as_str()
            .starts_with("https://login.live.com/oauth20_desktop.srf")
            && let Some((_, code)) =
                window.url()?.query_pairs().find(|x| x.0 == "code")
        {
            window.close()?;
            let val = minecraft_auth::finish_login(&code.clone(), flow).await?;

            return Ok(Some(val));
        }

        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }

    window.close()?;
    Ok(None)
}

/// Add an offline user profile
#[tauri::command]
pub async fn add_offline_user(username: String) -> Result<Credentials> {
	Ok(minecraft_auth::add_offline_user(username).await?)
}

#[tauri::command]
pub async fn remove_user(user: uuid::Uuid) -> Result<()> {
    Ok(minecraft_auth::remove_user(user).await?)
}

#[tauri::command]
pub async fn get_default_user() -> Result<Option<uuid::Uuid>> {
    Ok(minecraft_auth::get_default_user().await?)
}

#[tauri::command]
pub async fn set_default_user(user: uuid::Uuid) -> Result<()> {
    Ok(minecraft_auth::set_default_user(user).await?)
}

/// Get a copy of the list of all user credentials
#[tauri::command]
pub async fn get_users() -> Result<Vec<Credentials>> {
	Ok(minecraft_auth::users().await?)
}

#[tauri::command]
pub async fn get_cracked_status() -> Result<theseus::launcher::custom_skin::CrackedStatus> {
	let status = theseus::launcher::custom_skin::get_cracked_status().await?;
	Ok(status)
}

#[tauri::command]
pub async fn save_cracked_skin(username: String, base64_png: String) -> Result<()> {
	theseus::launcher::custom_skin::save_cracked_skin_from_str(&username, &base64_png).await?;
	Ok(())
}

#[tauri::command]
pub async fn get_cracked_skin(username: String) -> Result<Option<String>> {
	let skin = theseus::launcher::custom_skin::get_cracked_skin(&username).await?;
	Ok(skin)
}

#[tauri::command]
pub async fn delete_cracked_skin(username: String) -> Result<()> {
	theseus::launcher::custom_skin::delete_cracked_skin(&username).await?;
	Ok(())
}

#[tauri::command]
pub async fn set_authlib_endpoint(endpoint: String) -> Result<()> {
	theseus::launcher::custom_skin::set_authlib_endpoint(&endpoint).await?;
	Ok(())
}

#[tauri::command]
pub async fn download_authlib_injector() -> Result<()> {
	theseus::launcher::custom_skin::download_authlib_injector().await?;
	Ok(())
}

#[tauri::command]
pub async fn sync_cracked_skins_to_instances() -> Result<usize> {
	let count = theseus::launcher::custom_skin::sync_cracked_skins_to_all_instances().await?;
	Ok(count)
}

#[tauri::command]
pub async fn compute_offline_uuid(username: String) -> Result<String> {
	let uuid = theseus::launcher::custom_skin::compute_offline_uuid(&username);
	Ok(uuid.to_string())
}

#[tauri::command]
pub async fn fetch_skin_by_username(
	username: String,
) -> Result<theseus::launcher::custom_skin::FetchedSkinResult> {
	let res = theseus::launcher::custom_skin::fetch_skin_by_username(&username).await?;
	Ok(res)
}
