use gloo_storage::{LocalStorage, Storage};
use leptos::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    pub token: String,
    pub username: String,
    pub expires_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub username: String,
    pub token: String,
    pub expires_at: String,
}

#[derive(Debug, Clone)]
pub struct AuthContext {
    pub user: ReadSignal<Option<User>>,
    pub set_user: WriteSignal<Option<User>>,
}

impl AuthContext {
    pub fn new() -> Self {
        let (user, set_user) = create_signal(None::<User>);
        
        // Try to load user from localStorage on initialization
        if let Ok(stored_user) = LocalStorage::get::<User>("user") {
            set_user.set(Some(stored_user));
        }
        
        Self { user, set_user }
    }
    
    pub fn login(&self, user: User) {
        // Store in localStorage
        let _ = LocalStorage::set("user", &user);
        self.set_user.set(Some(user));
    }
    
    pub fn logout(&self) {
        // Remove from localStorage
        LocalStorage::delete("user");
        self.set_user.set(None);
    }
    
    pub fn is_authenticated(&self) -> bool {
        self.user.get().is_some()
    }
    
    pub fn get_token(&self) -> Option<String> {
        self.user.get().map(|u| u.token)
    }
}

pub fn provide_auth_context() -> AuthContext {
    let auth_context = AuthContext::new();
    provide_context(auth_context.clone());
    auth_context
}

pub fn use_auth_context() -> AuthContext {
    use_context::<AuthContext>().expect("AuthContext should be provided")
}