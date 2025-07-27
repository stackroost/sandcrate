import { useState } from 'react';
import { Link, useLocation } from 'react-router-dom';
import { 
  Terminal, 
  Server, 
  Database, 
  Shield, 
  Settings, 
  LogOut, 
  Menu,
  X,
  Activity,
  Zap,
  GitBranch,
  Docker
} from 'lucide-react';

const Sidebar = () => {
  const [isCollapsed, setIsCollapsed] = useState(false);
  const location = useLocation();

  const navigation = [
    { name: 'Dashboard', href: '/', icon: Terminal },
    { name: 'Infrastructure', href: '/infrastructure', icon: Server },
    { name: 'Databases', href: '/databases', icon: Database },
    { name: 'Security', href: '/security', icon: Shield },
    { name: 'Plugins', href: '/plugins', icon: Zap },
    { name: 'Settings', href: '/settings', icon: Settings },
  ];

  const isActive = (path: string) => location.pathname === path;

  return (
    <div className={`bg-terminal-800 border-r border-terminal-700 transition-all duration-300 ${
      isCollapsed ? 'w-16' : 'w-64'
    }`}>
      <div className="flex items-center justify-between p-4 border-b border-terminal-700">
        {!isCollapsed && (
          <div className="flex items-center space-x-2">
            <Terminal className="h-8 w-8 text-devops-green-400" />
            <span className="text-xl font-bold text-devops-green-400">SandCrate</span>
          </div>
        )}
        <button
          onClick={() => setIsCollapsed(!isCollapsed)}
          className="p-2 rounded-lg hover:bg-terminal-700 transition-colors"
        >
          {isCollapsed ? <Menu className="h-5 w-5" /> : <X className="h-5 w-5" />}
        </button>
      </div>

      <nav className="mt-4">
        <div className="px-3">
          <div className="space-y-1">
            {navigation.map((item) => {
              const Icon = item.icon;
              return (
                <Link
                  key={item.name}
                  to={item.href}
                  className={`flex items-center px-3 py-2 text-sm font-medium rounded-lg transition-colors ${
                    isActive(item.href)
                      ? 'bg-devops-green-600 text-white'
                      : 'text-terminal-300 hover:bg-terminal-700 hover:text-terminal-100'
                  }`}
                >
                  <Icon className="h-5 w-5 mr-3" />
                  {!isCollapsed && item.name}
                </Link>
              );
            })}
          </div>
        </div>

        {/* System Status */}
        {!isCollapsed && (
          <div className="mt-8 px-3">
            <div className="card">
              <div className="flex items-center space-x-2 mb-3">
                <Activity className="h-4 w-4 text-devops-green-400" />
                <span className="text-sm font-medium">System Status</span>
              </div>
              <div className="space-y-2">
                <div className="flex items-center justify-between">
                  <span className="text-xs text-terminal-400">API Gateway</span>
                  <div className="flex items-center space-x-1">
                    <div className="w-2 h-2 bg-devops-green-400 rounded-full animate-pulse"></div>
                    <span className="text-xs text-devops-green-400">Online</span>
                  </div>
                </div>
                <div className="flex items-center justify-between">
                  <span className="text-xs text-terminal-400">Database</span>
                  <div className="flex items-center space-x-1">
                    <div className="w-2 h-2 bg-devops-green-400 rounded-full animate-pulse"></div>
                    <span className="text-xs text-devops-green-400">Online</span>
                  </div>
                </div>
                <div className="flex items-center justify-between">
                  <span className="text-xs text-terminal-400">Load Balancer</span>
                  <div className="flex items-center space-x-1">
                    <div className="w-2 h-2 bg-devops-green-400 rounded-full animate-pulse"></div>
                    <span className="text-xs text-devops-green-400">Online</span>
                  </div>
                </div>
              </div>
            </div>
          </div>
        )}

        {/* User Section */}
        <div className="absolute bottom-0 left-0 right-0 p-3 border-t border-terminal-700">
          <div className="flex items-center space-x-3">
            <div className="w-8 h-8 bg-devops-green-600 rounded-full flex items-center justify-center">
              <span className="text-sm font-medium text-white">A</span>
            </div>
            {!isCollapsed && (
              <div className="flex-1">
                <p className="text-sm font-medium text-terminal-100">Admin</p>
                <p className="text-xs text-terminal-400">DevOps Engineer</p>
              </div>
            )}
            <button className="p-2 rounded-lg hover:bg-terminal-700 transition-colors">
              <LogOut className="h-4 w-4 text-terminal-400" />
            </button>
          </div>
        </div>
      </nav>
    </div>
  );
};

export default Sidebar; 