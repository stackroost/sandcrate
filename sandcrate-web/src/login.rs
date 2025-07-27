use crate::auth::{use_auth_context, User};
use leptos::*;
use leptos_router::*;

#[component]
pub fn LoginPage() -> impl IntoView {
    let auth = use_auth_context();
    let navigate = use_navigate();
    
    // Redirect to dashboard if already authenticated
    let auth_clone = auth.clone();
    let navigate_clone = navigate.clone();
    create_effect(move |_| {
        if auth_clone.is_authenticated() {
            navigate_clone("/dashboard", Default::default());
        }
    });
    
    let (username, set_username) = create_signal(String::new());
    let (password, set_password) = create_signal(String::new());
    let (error_message, set_error_message) = create_signal(None::<String>);
    let (is_loading, set_is_loading) = create_signal(false);

    let login_action = create_action(move |_: &()| {
        let username_val = username.get();
        let password_val = password.get();
        let auth = auth.clone();
        let navigate = navigate.clone();
        
        async move {
            set_is_loading.set(true);
            set_error_message.set(None);
            
            // For demo purposes, simulate a successful login
            // In production, this would be a real API call
            if username_val == "admin" && password_val == "password" {
                let user = User {
                    username: username_val,
                    token: "demo_token_123".to_string(),
                    expires_at: "2025-12-31T23:59:59Z".to_string(),
                };
                auth.login(user);
                navigate("/dashboard", Default::default());
            } else {
                set_error_message.set(Some("Invalid credentials. Try admin/password".to_string()));
            }
            
            set_is_loading.set(false);
        }
    });

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        if !username.get().is_empty() && !password.get().is_empty() {
            login_action.dispatch(());
        }
    };

    view! {
        <div class="auth-page">
            <div class="auth-background">
                <div class="auth-particles"></div>
            </div>
            
            <div class="auth-container">
                <div class="auth-card">
                    <div class="auth-header">
                        <div class="logo-container">
                            <div class="logo-icon">"🔒"</div>
                            <h1 class="logo-text">"SandCrate"</h1>
                        </div>
                        <p class="auth-subtitle">"Secure System Access Portal"</p>
                    </div>
                    
                    <form on:submit=on_submit class="auth-form">
                        <div class="form-group">
                            <label for="username" class="form-label">"Username"</label>
                            <div class="input-wrapper">
                                <span class="input-icon">"👤"</span>
                                <input
                                    type="text"
                                    id="username"
                                    class="form-input"
                                    placeholder="Enter your username"
                                    prop:value=username
                                    on:input=move |ev| set_username.set(event_target_value(&ev))
                                    disabled=move || is_loading.get()
                                />
                            </div>
                        </div>
                        
                        <div class="form-group">
                            <label for="password" class="form-label">"Password"</label>
                            <div class="input-wrapper">
                                <span class="input-icon">"🔐"</span>
                                <input
                                    type="password"
                                    id="password"
                                    class="form-input"
                                    placeholder="Enter your password"
                                    prop:value=password
                                    on:input=move |ev| set_password.set(event_target_value(&ev))
                                    disabled=move || is_loading.get()
                                />
                            </div>
                        </div>
                        
                        {move || error_message.get().map(|msg| view! {
                            <div class="error-message">
                                <span class="error-icon">"⚠️"</span>
                                <span class="error-text">{msg}</span>
                            </div>
                        })}
                        
                        <button
                            type="submit"
                            class="auth-button"
                            disabled=move || is_loading.get() || username.get().is_empty() || password.get().is_empty()
                        >
                            {move || if is_loading.get() { 
                                view! {
                                    <span class="button-content">
                                        <span class="spinner-small"></span>
                                        <span>"Signing in..."</span>
                                    </span>
                                }
                            } else { 
                                view! {
                                    <span class="button-content">
                                        <span class="button-icon">"🚀"</span>
                                        <span>"Sign In"</span>
                                    </span>
                                }
                            }}
                        </button>
                    </form>
                    
                    <div class="auth-footer">
                        <div class="demo-info">
                            <p class="demo-text">"Demo Credentials:"</p>
                            <p class="demo-credentials">"Username: admin | Password: password"</p>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}