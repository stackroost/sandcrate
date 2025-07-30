#!/usr/bin/env python3
import asyncio
import websockets
import json

async def test_websocket():
    uri = "ws://localhost:3000/ws/plugins"
    
    try:
        async with websockets.connect(uri) as websocket:
            print("Connected to WebSocket")
            
            # Send a test message
            test_message = {
                "command": "execute_plugin",
                "plugin_id": "plugin_hello",
                "parameters": {"test": "data"},
                "timeout": 10000
            }
            
            await websocket.send(json.dumps(test_message))
            print(f"Sent: {test_message}")
            
            # Listen for responses
            count = 0
            while count < 10:  # Listen for up to 10 messages
                try:
                    response = await asyncio.wait_for(websocket.recv(), timeout=5.0)
                    print(f"Received: {response}")
                    count += 1
                    
                    # Parse the response
                    data = json.loads(response)
                    if data.get("type") == "result":
                        print("Plugin execution completed!")
                        break
                        
                except asyncio.TimeoutError:
                    print("Timeout waiting for response")
                    break
                    
    except Exception as e:
        print(f"Error: {e}")

if __name__ == "__main__":
    asyncio.run(test_websocket()) 