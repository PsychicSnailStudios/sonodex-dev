use std::sync::Mutex;

pub struct AppState {
	pub active_profile_uid: Mutex<String>,
}

impl AppState {
	pub fn new(uid: String) -> Self {
		AppState {
			active_profile_uid: Mutex::new(uid),
		}
	}

	pub fn get_uid(&self) -> String {
		self.active_profile_uid.lock().unwrap().clone()
	}

	pub fn set_uid(&self, uid: String) {
		*self.active_profile_uid.lock().unwrap() = uid;
	}
}
