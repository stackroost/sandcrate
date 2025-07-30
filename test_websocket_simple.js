const WebSocket = require('ws');

async function testWebSocket() {
    const ws = new WebSocket('ws://localhost:3000/ws/plugins');
    
    ws.on('open', function open() {
        console.log('✅ WebSocket connected successfully');
        
        // Send a test message
        const testMessage = {
            command: 'execute_plugin',
            plugin_id: 'plugin_hello',
            parameters: { test: 'data' },
            timeout: 10000
        };
        
        console.log('📤 Sending test message:', JSON.stringify(testMessage));
        ws.send(JSON.stringify(testMessage));
    });
    
    ws.on('message', function message(data) {
        try {
            const parsed = JSON.parse(data.toString());
            console.log('📥 Received:', JSON.stringify(parsed, null, 2));
            
            if (parsed.type === 'result') {
                console.log('✅ Plugin execution completed!');
                ws.close();
            }
        } catch (e) {
            console.log('📥 Received raw:', data.toString());
        }
    });
    
    ws.on('close', function close() {
        console.log('🔌 WebSocket connection closed');
    });
    
    ws.on('error', function error(err) {
        console.error('❌ WebSocket error:', err.message);
    });
    
    // Close after 10 seconds
    setTimeout(() => {
        console.log('⏰ Timeout reached, closing connection');
        ws.close();
    }, 10000);
}

// Check if ws module is available
try {
    require('ws');
    testWebSocket();
} catch (e) {
    console.log('❌ WebSocket module not available. Install with: npm install ws');
    console.log('📝 You can test the WebSocket manually by:');
    console.log('   1. Opening http://localhost:5173 in your browser');
    console.log('   2. Going to the Plugins page');
    console.log('   3. Clicking "Real-time" on any plugin');
    console.log('   4. Clicking "Execute" to start real-time execution');
} 