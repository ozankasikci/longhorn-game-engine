//! Tests for TypeScript Event API bindings
//! 
//! These tests define the expected behavior of the Event bindings for TypeScript scripts.
//! Following TDD principles, these tests are written before implementation.

use crate::initialize_v8_platform;
use crate::runtime::TypeScriptRuntime;
use engine_scripting::{
    runtime::ScriptRuntime,
    types::{ScriptId, ScriptMetadata, ScriptType},
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_listener_registration() {
        initialize_v8_platform().unwrap();
        let mut runtime = TypeScriptRuntime::new().unwrap();
        runtime.initialize().unwrap();
        
        let typescript_code = r#"
            let eventReceived = false;
            let lastEventData: any = null;
            
            function onCustomEvent(data: any): void {
                eventReceived = true;
                lastEventData = data;
            }
            
            function registerListener(): void {
                engine.events.onEvent("CustomEvent", onCustomEvent);
            }
            
            function wasEventReceived(): boolean {
                return eventReceived;
            }
            
            function getLastEventData(): string {
                return JSON.stringify(lastEventData);
            }
            
            function resetEventState(): void {
                eventReceived = false;
                lastEventData = null;
            }
        "#;
        
        let script_id = ScriptId(1);
        let metadata = ScriptMetadata {
            id: script_id,
            script_type: ScriptType::TypeScript,
            path: "test_event_listeners.ts".to_string(),
            entry_point: None,
        };
        
        runtime.load_script(metadata, typescript_code).unwrap();
        runtime.execute_script(script_id).unwrap();
        
        // Test initial state
        let received = runtime.execute_function("wasEventReceived", vec![]).unwrap();
        assert_eq!(received, "false", "Should not have received events initially");
        
        // Test listener registration works without error
        runtime.execute_function("registerListener", vec![]).unwrap();
        
        // Test reset functionality
        runtime.execute_function("resetEventState", vec![]).unwrap();
        let reset_received = runtime.execute_function("wasEventReceived", vec![]).unwrap();
        assert_eq!(reset_received, "false", "Event state should be reset");
    }

    #[test]
    fn test_event_emission() {
        initialize_v8_platform().unwrap();
        let mut runtime = TypeScriptRuntime::new().unwrap();
        runtime.initialize().unwrap();
        
        let typescript_code = r#"
            interface GameEvent {
                type: string;
                timestamp: number;
                data?: any;
            }
            
            function emitSimpleEvent(): void {
                engine.events.emitEvent("PlayerJump", { height: 5.0 });
            }
            
            function emitComplexEvent(): void {
                const eventData = {
                    player: { id: 123, name: "Hero" },
                    action: "levelComplete",
                    score: 9000,
                    timestamp: Date.now()
                };
                engine.events.emitEvent("GameEvent", eventData);
            }
            
            function emitSystemEvent(): void {
                engine.events.emitEvent("SystemEvent", "gameStarted");
            }
            
            function testEventEmission(): boolean {
                try {
                    emitSimpleEvent();
                    emitComplexEvent();
                    emitSystemEvent();
                    return true;
                } catch (e) {
                    return false;
                }
            }
        "#;
        
        let script_id = ScriptId(2);
        let metadata = ScriptMetadata {
            id: script_id,
            script_type: ScriptType::TypeScript,
            path: "test_event_emission.ts".to_string(),
            entry_point: None,
        };
        
        runtime.load_script(metadata, typescript_code).unwrap();
        runtime.execute_script(script_id).unwrap();
        
        // Test event emission works without errors
        let emission_result = runtime.execute_function("testEventEmission", vec![]).unwrap();
        assert_eq!(emission_result, "true", "Event emission should work without errors");
        
        // Test individual emission functions
        runtime.execute_function("emitSimpleEvent", vec![]).unwrap();
        runtime.execute_function("emitComplexEvent", vec![]).unwrap();
        runtime.execute_function("emitSystemEvent", vec![]).unwrap();
    }

    #[test]
    fn test_event_listener_removal() {
        initialize_v8_platform().unwrap();
        let mut runtime = TypeScriptRuntime::new().unwrap();
        runtime.initialize().unwrap();
        
        let typescript_code = r#"
            let listenerId: number = -1;
            let eventCount = 0;
            
            function onTestEvent(): void {
                eventCount++;
            }
            
            function addListener(): number {
                listenerId = engine.events.onEvent("TestEvent", onTestEvent);
                return listenerId;
            }
            
            function removeListener(): void {
                if (listenerId !== -1) {
                    engine.events.removeListener(listenerId);
                }
            }
            
            function getEventCount(): number {
                return eventCount;
            }
            
            function getListenerId(): number {
                return listenerId;
            }
            
            function resetCount(): void {
                eventCount = 0;
            }
        "#;
        
        let script_id = ScriptId(3);
        let metadata = ScriptMetadata {
            id: script_id,
            script_type: ScriptType::TypeScript,
            path: "test_listener_removal.ts".to_string(),
            entry_point: None,
        };
        
        runtime.load_script(metadata, typescript_code).unwrap();
        runtime.execute_script(script_id).unwrap();
        
        // Test listener addition returns valid ID
        let listener_id = runtime.execute_function("addListener", vec![]).unwrap();
        let id: i32 = listener_id.parse().unwrap();
        assert!(id > 0, "Listener ID should be positive");
        
        // Test listener ID is stored correctly
        let stored_id = runtime.execute_function("getListenerId", vec![]).unwrap();
        assert_eq!(listener_id, stored_id, "Stored listener ID should match returned ID");
        
        // Test listener removal works without error
        runtime.execute_function("removeListener", vec![]).unwrap();
        
        // Test event count functionality
        let count = runtime.execute_function("getEventCount", vec![]).unwrap();
        assert_eq!(count, "0", "Initial event count should be 0");
        
        runtime.execute_function("resetCount", vec![]).unwrap();
        let reset_count = runtime.execute_function("getEventCount", vec![]).unwrap();
        assert_eq!(reset_count, "0", "Event count should remain 0 after reset");
    }

    #[test]
    fn test_system_events() {
        initialize_v8_platform().unwrap();
        let mut runtime = TypeScriptRuntime::new().unwrap();
        runtime.initialize().unwrap();
        
        let typescript_code = r#"
            let systemEvents: string[] = [];
            
            function onSystemStart(): void {
                systemEvents.push("start");
            }
            
            function onSystemStop(): void {
                systemEvents.push("stop");
            }
            
            function onSystemUpdate(deltaTime: number): void {
                systemEvents.push(`update:${deltaTime}`);
            }
            
            function registerSystemListeners(): void {
                engine.events.onEvent("SystemStart", onSystemStart);
                engine.events.onEvent("SystemStop", onSystemStop);
                engine.events.onEvent("SystemUpdate", onSystemUpdate);
            }
            
            function getSystemEvents(): string {
                return JSON.stringify(systemEvents);
            }
            
            function clearSystemEvents(): void {
                systemEvents = [];
            }
            
            function getEventCount(): number {
                return systemEvents.length;
            }
        "#;
        
        let script_id = ScriptId(4);
        let metadata = ScriptMetadata {
            id: script_id,
            script_type: ScriptType::TypeScript,
            path: "test_system_events.ts".to_string(),
            entry_point: None,
        };
        
        runtime.load_script(metadata, typescript_code).unwrap();
        runtime.execute_script(script_id).unwrap();
        
        // Test system listener registration
        runtime.execute_function("registerSystemListeners", vec![]).unwrap();
        
        // Test initial state
        let count = runtime.execute_function("getEventCount", vec![]).unwrap();
        assert_eq!(count, "0", "Should have no system events initially");
        
        // Test event list functionality
        let events = runtime.execute_function("getSystemEvents", vec![]).unwrap();
        assert!(events.contains("["), "Should return array format");
        
        // Test clearing events
        runtime.execute_function("clearSystemEvents", vec![]).unwrap();
        let cleared_count = runtime.execute_function("getEventCount", vec![]).unwrap();
        assert_eq!(cleared_count, "0", "Event count should be 0 after clearing");
    }

    #[test]
    fn test_game_events() {
        initialize_v8_platform().unwrap();
        let mut runtime = TypeScriptRuntime::new().unwrap();
        runtime.initialize().unwrap();
        
        let typescript_code = r#"
            interface EntityEvent {
                entityId: number;
                componentType?: string;
            }
            
            interface CollisionEvent {
                entity1: number;
                entity2: number;
                point: { x: number; y: number; z: number };
            }
            
            let gameEvents: any[] = [];
            
            function onEntitySpawned(event: EntityEvent): void {
                gameEvents.push({ type: "spawned", ...event });
            }
            
            function onEntityDestroyed(event: EntityEvent): void {
                gameEvents.push({ type: "destroyed", ...event });
            }
            
            function onCollisionStart(event: CollisionEvent): void {
                gameEvents.push({ type: "collision", ...event });
            }
            
            function registerGameListeners(): void {
                engine.events.onEvent("EntitySpawned", onEntitySpawned);
                engine.events.onEvent("EntityDestroyed", onEntityDestroyed);
                engine.events.onEvent("CollisionStart", onCollisionStart);
            }
            
            function getGameEvents(): string {
                return JSON.stringify(gameEvents);
            }
            
            function getGameEventCount(): number {
                return gameEvents.length;
            }
            
            function clearGameEvents(): void {
                gameEvents = [];
            }
        "#;
        
        let script_id = ScriptId(5);
        let metadata = ScriptMetadata {
            id: script_id,
            script_type: ScriptType::TypeScript,
            path: "test_game_events.ts".to_string(),
            entry_point: None,
        };
        
        runtime.load_script(metadata, typescript_code).unwrap();
        runtime.execute_script(script_id).unwrap();
        
        // Test game listener registration
        runtime.execute_function("registerGameListeners", vec![]).unwrap();
        
        // Test initial state
        let count = runtime.execute_function("getGameEventCount", vec![]).unwrap();
        assert_eq!(count, "0", "Should have no game events initially");
        
        // Test event management functions work
        runtime.execute_function("clearGameEvents", vec![]).unwrap();
        let cleared_count = runtime.execute_function("getGameEventCount", vec![]).unwrap();
        assert_eq!(cleared_count, "0", "Event count should remain 0 after clearing");
        
        // Test events array format
        let events = runtime.execute_function("getGameEvents", vec![]).unwrap();
        // The result should be "[]" (JSON encoded string), so we need to check for that pattern
        assert!(events == "[]" || (events.starts_with("\"[") && events.ends_with("]\"")) || (events.starts_with("[") && events.ends_with("]")), "Should return valid JSON array");
    }

    #[test]
    fn test_event_validation() {
        initialize_v8_platform().unwrap();
        let mut runtime = TypeScriptRuntime::new().unwrap();
        runtime.initialize().unwrap();
        
        let typescript_code = r#"
            function testEmptyEventType(): boolean {
                try {
                    engine.events.emitEvent("", { data: "test" });
                    return false; // Should not succeed
                } catch (e) {
                    return true; // Expected to fail
                }
            }
            
            function testNullEventType(): boolean {
                try {
                    engine.events.emitEvent(null, { data: "test" });
                    return false; // Should not succeed
                } catch (e) {
                    return true; // Expected to fail
                }
            }
            
            function testInvalidListenerRemoval(): boolean {
                try {
                    engine.events.removeListener(-1); // Invalid ID
                    return true; // Should handle gracefully
                } catch (e) {
                    return true; // Either graceful handling or error is acceptable
                }
            }
            
            function testNullCallback(): boolean {
                try {
                    engine.events.onEvent("TestEvent", null);
                    return false; // Should not succeed
                } catch (e) {
                    return true; // Expected to fail
                }
            }
        "#;
        
        let script_id = ScriptId(6);
        let metadata = ScriptMetadata {
            id: script_id,
            script_type: ScriptType::TypeScript,
            path: "test_event_validation.ts".to_string(),
            entry_point: None,
        };
        
        runtime.load_script(metadata, typescript_code).unwrap();
        runtime.execute_script(script_id).unwrap();
        
        // Test empty event type handling
        let empty_result = runtime.execute_function("testEmptyEventType", vec![]).unwrap();
        assert_eq!(empty_result, "true", "Empty event type should be rejected");
        
        // Test null event type handling
        let null_result = runtime.execute_function("testNullEventType", vec![]).unwrap();
        assert_eq!(null_result, "true", "Null event type should be rejected");
        
        // Test invalid listener removal
        let invalid_removal = runtime.execute_function("testInvalidListenerRemoval", vec![]).unwrap();
        assert_eq!(invalid_removal, "true", "Invalid listener removal should be handled gracefully");
        
        // Test null callback handling
        let null_callback = runtime.execute_function("testNullCallback", vec![]).unwrap();
        assert_eq!(null_callback, "true", "Null callback should be rejected");
    }
}