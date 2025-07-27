use crate::auth::use_auth_context;
use leptos::*;
use leptos_router::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginList {
    pub plugins: Vec<String>,
}

#[component]
pub fn Dashboard() -> impl IntoView {
    let auth = use_auth_context();
    let navigate = use_navigate();
    
    let (plugins, set_plugins) = create_signal(vec![
        "File Manager".to_string(),
        "System Monitor".to_string(),
        "Network Tools".to_string(),
        "Security Scanner".to_string(),
        "Database Browser".to_string(),
        "Log Analyzer".to_string(),
    ]);
    let (is_loading, set_is_loading) = create_signal(false);
    let (active_tab, set_active_tab) = create_signal("dashboard".to_string());

    let auth_clone = auth.clone();
    let logout = move |_| {
        auth_clone.logout();
        navigate("/auth", Default::default());
    };

    let user = move || auth.user.get();

    view! {
        <div class="app-layout">
            // Sidebar
            <aside class="sidebar">
                <div class="sidebar-header">
                    <div class="logo">
                        <span class="logo-icon">"🔒"</span>
                        <span class="logo-text">"SandCrate"</span>
                    </div>
                </div>
                
                <nav class="sidebar-nav">
                    <button 
                        class=move || format!("nav-item {}", if active_tab.get() == "dashboard" { "active" } else { "" })
                        on:click=move |_| set_active_tab.set("dashboard".to_string())
                    >
                        <span class="nav-icon">"📊"</span>
                        <span class="nav-text">"Dashboard"</span>
                    </button>
                    
                    <button 
                        class=move || format!("nav-item {}", if active_tab.get() == "plugins" { "active" } else { "" })
                        on:click=move |_| set_active_tab.set("plugins".to_string())
                    >
                        <span class="nav-icon">"🔌"</span>
                        <span class="nav-text">"Plugins"</span>
                    </button>
                    
                    <button 
                        class=move || format!("nav-item {}", if active_tab.get() == "settings" { "active" } else { "" })
                        on:click=move |_| set_active_tab.set("settings".to_string())
                    >
                        <span class="nav-icon">"⚙️"</span>
                        <span class="nav-text">"Settings"</span>
                    </button>
                </nav>
                
                <div class="sidebar-footer">
                    <div class="user-info">
                        <div class="user-avatar">"👤"</div>
                        <div class="user-details">
                            <span class="user-name">{move || user().map(|u| u.username).unwrap_or_default()}</span>
                            <span class="user-role">"Administrator"</span>
                        </div>
                    </div>
                    <button class="logout-btn" on:click=logout>
                        <span class="logout-icon">"🚪"</span>
                        <span>"Sign Out"</span>
                    </button>
                </div>
            </aside>

            // Main Content
            <main class="main-content">
                <header class="top-header">
                    <div class="header-left">
                        <h1 class="page-title">
                            {move || match active_tab.get().as_str() {
                                "dashboard" => "Dashboard",
                                "plugins" => "Plugin Management",
                                "settings" => "Settings",
                                _ => "Dashboard"
                            }}
                        </h1>
                    </div>
                    <div class="header-right">
                        <div class="status-indicator">
                            <span class="status-dot online"></span>
                            <span class="status-text">"System Online"</span>
                        </div>
                    </div>
                </header>

                <div class="content-area">
                    <Show when=move || active_tab.get() == "dashboard" fallback=|| view! { <div></div> }>
                        <div class="dashboard-tab">
                            <div class="stats-grid">
                                <div class="stat-card">
                                    <div class="stat-icon">"🔌"</div>
                                    <div class="stat-content">
                                        <h3 class="stat-title">"Active Plugins"</h3>
                                        <p class="stat-number">{move || plugins.get().len()}</p>
                                        <p class="stat-label">"Available"</p>
                                    </div>
                                </div>
                                
                                <div class="stat-card">
                                    <div class="stat-icon">"✅"</div>
                                    <div class="stat-content">
                                        <h3 class="stat-title">"System Status"</h3>
                                        <p class="stat-number">"Online"</p>
                                        <p class="stat-label">"All Systems Operational"</p>
                                    </div>
                                </div>
                                
                                <div class="stat-card">
                                    <div class="stat-icon">"🔒"</div>
                                    <div class="stat-content">
                                        <h3 class="stat-title">"Security"</h3>
                                        <p class="stat-number">"PAM"</p>
                                        <p class="stat-label">"Authentication Active"</p>
                                    </div>
                                </div>
                                
                                <div class="stat-card">
                                    <div class="stat-icon">"📈"</div>
                                    <div class="stat-content">
                                        <h3 class="stat-title">"Performance"</h3>
                                        <p class="stat-number">"98%"</p>
                                        <p class="stat-label">"System Efficiency"</p>
                                    </div>
                                </div>
                            </div>

                            <div class="recent-activity">
                                <h2 class="section-title">"Recent Activity"</h2>
                                <div class="activity-list">
                                    <div class="activity-item">
                                        <div class="activity-icon">"🔌"</div>
                                        <div class="activity-content">
                                            <p class="activity-text">"File Manager plugin loaded successfully"</p>
                                            <span class="activity-time">"2 minutes ago"</span>
                                        </div>
                                    </div>
                                    <div class="activity-item">
                                        <div class="activity-icon">"👤"</div>
                                        <div class="activity-content">
                                            <p class="activity-text">"User authentication successful"</p>
                                            <span class="activity-time">"5 minutes ago"</span>
                                        </div>
                                    </div>
                                    <div class="activity-item">
                                        <div class="activity-icon">"🔒"</div>
                                        <div class="activity-content">
                                            <p class="activity-text">"Security scan completed"</p>
                                            <span class="activity-time">"10 minutes ago"</span>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </Show>

                    <Show when=move || active_tab.get() == "plugins" fallback=|| view! { <div></div> }>
                        <div class="plugins-tab">
                            <div class="plugins-header">
                                <h2 class="section-title">"Available Plugins"</h2>
                                <button class="refresh-btn">
                                    <span class="refresh-icon">"🔄"</span>
                                    <span>"Refresh"</span>
                                </button>
                            </div>
                            
                            <div class="plugins-grid">
                                {move || {
                                    let plugin_list = plugins.get();
                                    plugin_list.into_iter().map(|plugin| view! {
                                        <div class="plugin-card">
                                            <div class="plugin-header">
                                                <div class="plugin-icon">"🔌"</div>
                                                <div class="plugin-status online"></div>
                                            </div>
                                            <div class="plugin-info">
                                                <h4 class="plugin-name">{plugin.clone()}</h4>
                                                <p class="plugin-description">"WASM-based system plugin"</p>
                                                <div class="plugin-meta">
                                                    <span class="plugin-version">"v1.0.0"</span>
                                                    <span class="plugin-type">"System Tool"</span>
                                                </div>
                                            </div>
                                            <div class="plugin-actions">
                                                <button class="plugin-btn primary">
                                                    <span class="btn-icon">"▶️"</span>
                                                    <span>"Run"</span>
                                                </button>
                                                <button class="plugin-btn secondary">
                                                    <span class="btn-icon">"ℹ️"</span>
                                                    <span>"Info"</span>
                                                </button>
                                            </div>
                                        </div>
                                    }).collect::<Vec<_>>()
                                }}
                            </div>
                        </div>
                    </Show>

                    <Show when=move || active_tab.get() == "settings" fallback=|| view! { <div></div> }>
                        <div class="settings-tab">
                            <h2 class="section-title">"System Settings"</h2>
                            <div class="settings-grid">
                                <div class="setting-card">
                                    <h3 class="setting-title">"Authentication"</h3>
                                    <p class="setting-description">"Configure system authentication settings"</p>
                                    <button class="setting-btn">"Configure"</button>
                                </div>
                                <div class="setting-card">
                                    <h3 class="setting-title">"Security"</h3>
                                    <p class="setting-description">"Manage security policies and access controls"</p>
                                    <button class="setting-btn">"Configure"</button>
                                </div>
                                <div class="setting-card">
                                    <h3 class="setting-title">"Plugins"</h3>
                                    <p class="setting-description">"Configure plugin management and permissions"</p>
                                    <button class="setting-btn">"Configure"</button>
                                </div>
                            </div>
                        </div>
                    </Show>
                </div>
            </main>
        </div>
    }
}