use leptos::*;
use leptos_router::*;
use wasm_bindgen::prelude::*;

mod auth;
mod login;
mod dashboard;

use auth::{provide_auth_context, use_auth_context};
use login::LoginPage;
use dashboard::Dashboard;

#[component]
pub fn App() -> impl IntoView {
    log::info!("App component rendering...");
    
    view! {
        <div style="padding: 20px; color: white; background-color: #333;">
            <h1>"SandCrate is working!"</h1>
            <p>"If you can see this, the app is mounted successfully."</p>
            
            <Router>
                <nav style="margin-bottom: 20px;">
                    <A href="/">"Home"</A>
                    <A href="/auth">"Auth"</A>
                    <A href="/dashboard">"Dashboard"</A>
                </nav>
                
                <Routes>
                    <Route path="/" view=|| view! { 
                        <div>
                            <h2>"Home Page"</h2>
                            <p>"Welcome to SandCrate!"</p>
                        </div> 
                    } />
                    <Route path="/auth" view=|| view! { 
                        <div>
                            <h2>"Auth Page"</h2>
                            <p>"This is the authentication page."</p>
                            <button style="padding: 10px; margin: 10px; background-color: #007bff; color: white; border: none; border-radius: 5px;">
                                "Login"
                            </button>
                        </div> 
                    } />
                    <Route path="/dashboard" view=|| view! { 
                        <div>
                            <h2>"Dashboard Page"</h2>
                            <p>"This is the dashboard page."</p>
                            <div style="background-color: #444; padding: 15px; border-radius: 5px;">
                                <h3>"System Status"</h3>
                                <p>"All systems operational"</p>
                            </div>
                        </div> 
                    } />
                </Routes>
            </Router>
        </div>
    }
}

#[wasm_bindgen]
pub fn main() {
    console_log::init_with_level(log::Level::Debug).expect("Failed to initialize logger");
    log::info!("Starting SandCrate application...");
    
    // Add error handling
    std::panic::set_hook(Box::new(|panic_info| {
        log::error!("Panic: {:?}", panic_info);
    }));
    
    // Mount the app to the body
    leptos::mount_to_body(|| {
        log::info!("Mounting App component...");
        App
    });
}

#[component]
fn HomePage() -> impl IntoView {
    let auth = use_auth_context();
    let navigate = use_navigate();
    
    // Redirect based on authentication status
    create_effect(move |_| {
        if auth.is_authenticated() {
            navigate("/dashboard", Default::default());
        } else {
            navigate("/auth", Default::default());
        }
    });
    
    view! {
        <div class="loading-page">
            <div class="loading-spinner"></div>
            <p>"Redirecting..."</p>
        </div>
    }
}

#[component]
fn ProtectedRoute() -> impl IntoView {
    let auth = use_auth_context();
    let navigate = use_navigate();
    
    // Redirect to login if not authenticated
    let auth_clone = auth.clone();
    let navigate_clone = navigate.clone();
    create_effect(move |_| {
        if !auth_clone.is_authenticated() {
            navigate_clone("/auth", Default::default());
        }
    });
    
    view! {
        <Show when=move || auth.is_authenticated() fallback=|| view! {
            <div class="loading-page">
                <div class="loading-spinner"></div>
                <p>"Checking authentication..."</p>
            </div>
        }>
            <Dashboard />
        </Show>
    }
}

