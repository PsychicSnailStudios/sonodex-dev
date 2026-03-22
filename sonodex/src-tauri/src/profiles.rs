use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Profile {
	pub uid: String,
	pub name: String,
	pub avatar_blob: Option<Vec<u8>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProfileRegistry {
	pub active: String,
	pub profiles: Vec<Profile>,
}

pub fn get_profiles_dir() -> PathBuf {
	let mut path = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
	path.push("sonodex");
	fs::create_dir_all(&path).ok();
	path
}

pub fn get_profiles_json_path() -> PathBuf {
	get_profiles_dir().join("profiles.json")
}

pub fn get_profile_dir(uid: &str) -> PathBuf {
	let mut path = get_profiles_dir();
	path.push(uid);
	fs::create_dir_all(&path).ok();
	path
}

pub fn get_settings_db_path(uid: &str) -> PathBuf {
	get_profile_dir(uid).join("settings.db")
}

pub fn get_lib_db_path(uid: &str) -> PathBuf {
	get_profile_dir(uid).join("lib.db")
}

pub fn read_registry() -> ProfileRegistry {
	let path = get_profiles_json_path();
	if !path.exists() {
		return ProfileRegistry {
			active: String::new(),
			profiles: vec![],
		};
	}
	let raw = fs::read_to_string(&path).unwrap_or_default();
	serde_json::from_str(&raw).unwrap_or(ProfileRegistry {
		active: String::new(),
		profiles: vec![],
	})
}

pub fn write_registry(registry: &ProfileRegistry) {
	let path = get_profiles_json_path();
	let raw = serde_json::to_string_pretty(registry).unwrap_or_default();
	fs::write(path, raw).ok();
}

pub fn create_profile(name: &str, avatar_blob: Option<Vec<u8>>) -> Profile {
	let uid = uuid::Uuid::new_v4().to_string();
	get_profile_dir(&uid);
	Profile {
		uid,
		name: name.to_string(),
		avatar_blob,
	}
}

pub fn get_active_profile_uid() -> Option<String> {
	let registry = read_registry();
	if registry.active.is_empty() {
		None
	} else {
		Some(registry.active)
	}
}
