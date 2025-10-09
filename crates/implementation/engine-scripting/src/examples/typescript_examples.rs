//! TypeScript example scripts for documentation and learning
//! These examples demonstrate how to use the TypeScript scripting API

use super::{ExampleScript, DifficultyLevel, ExampleCategory};

/// Get all available TypeScript example scripts
pub fn get_all_typescript_examples() -> Vec<ExampleScript> {
    vec![
        hello_world_example(),
        entity_controller_example(),
        input_handling_example(),
        physics_basic_example(),
        event_system_example(),
        vector_math_example(),
        transform_manipulation_example(),
        debugging_example(),
    ]
}

/// Basic Hello World example
fn hello_world_example() -> ExampleScript {
    ExampleScript {
        name: "typescript_hello_world".to_string(),
        description: "The classic Hello World example in TypeScript".to_string(),
        code: r#"// Simple Hello World example - no imports needed for basic console operations
export class HelloWorld {
    init(): void {
        console.log("Hello, World!");
        console.log("Welcome to Longhorn Game Engine TypeScript scripting!");
    }
    
    update(deltaTime: number): void {
        // Update logic here
    }
    
    destroy(): void {
        console.log("Goodbye from TypeScript!");
    }
}"#.to_string(),
        expected_outputs: vec!["Hello, World!".to_string()],
        api_features: vec!["console.log".to_string()],
        difficulty_level: DifficultyLevel::Beginner,
        category: ExampleCategory::BasicSyntax,
    }
}

/// Entity controller with transform manipulation
fn entity_controller_example() -> ExampleScript {
    ExampleScript {
        name: "typescript_entity_controller".to_string(),
        description: "Entity controller with transform manipulation".to_string(),
        code: r#"// World, Input, Physics APIs are globally available via globalThis
// No imports needed - APIs are injected by the V8 runtime
export class EntityController {
    private entityId: number;
    private rotationSpeed: number = 1.0;
    
    init(): void {
        this.entityId = globalThis.World.createEntity();
        globalThis.World.addComponent(this.entityId, 'Transform', {
            position: { x: 0.0, y: 0.0, z: 0.0 },
            rotation: { x: 0.0, y: 0.0, z: 0.0, w: 1.0 },
            scale: { x: 1.0, y: 1.0, z: 1.0 }
        });
        console.log("EntityController initialized");
    }
    
    update(deltaTime: number): void {
        const transform = globalThis.World.getComponent(this.entityId, 'Transform');
        if (transform) {
            // Rotate the entity
            transform.rotation.y += this.rotationSpeed * deltaTime;
            
            // Simple movement with input
            if (globalThis.Input.isKeyPressed("W")) {
                transform.position.z -= deltaTime * 5.0;
            }
            if (globalThis.Input.isKeyPressed("S")) {
                transform.position.z += deltaTime * 5.0;
            }
            
            globalThis.World.updateComponent(this.entityId, 'Transform', transform);
        }
    }
    
    destroy(): void {
        console.log("EntityController destroyed");
    }
}"#.to_string(),
        expected_outputs: vec!["EntityController initialized".to_string()],
        api_features: vec![
            "globalThis.World".to_string(),
            "getComponent".to_string(),
            "globalThis.Input".to_string(),
            "isKeyPressed".to_string(),
        ],
        difficulty_level: DifficultyLevel::Intermediate,
        category: ExampleCategory::GameLogic,
    }
}

/// Basic input handling example
fn input_handling_example() -> ExampleScript {
    ExampleScript {
        name: "typescript_input_handling".to_string(),
        description: "Basic input handling with keyboard and mouse".to_string(),
        code: r#"// Input API is available globally via globalThis - no imports required
export class InputHandler {
    init(): void {
        console.log("Input handler ready");
    }
    
    update(deltaTime: number): void {
        // Keyboard input
        if (globalThis.Input.isKeyPressed("Space")) {
            console.log("Space key pressed!");
        }
        
        if (globalThis.Input.isKeyPressed("Escape")) {
            console.log("Escape key pressed!");
        }
        
        // Mouse input
        const mousePos = globalThis.Input.getMousePosition();
        if (globalThis.Input.isMouseButtonPressed(0)) { // Left click
            console.log(`Mouse clicked at: (${mousePos.x}, ${mousePos.y})`);
        }
        
        // Arrow keys for movement
        if (globalThis.Input.isKeyPressed("ArrowUp")) {
            console.log("Moving up");
        }
        if (globalThis.Input.isKeyPressed("ArrowDown")) {
            console.log("Moving down");
        }
    }
    
    destroy(): void {
        console.log("Input handler stopped");
    }
}"#.to_string(),
        expected_outputs: vec!["Input handler ready".to_string()],
        api_features: vec![
            "globalThis.Input".to_string(),
            "isKeyPressed".to_string(),
            "getMousePosition".to_string(),
            "isMouseButtonPressed".to_string(),
        ],
        difficulty_level: DifficultyLevel::Beginner,
        category: ExampleCategory::InputHandling,
    }
}

/// Basic physics simulation
fn physics_basic_example() -> ExampleScript {
    ExampleScript {
        name: "typescript_physics_basic".to_string(),
        description: "Basic physics simulation with Vector3 math".to_string(),
        code: r#"// Physics API is available via globalThis
export class PhysicsObject {
    private entityId: number;
    private velocity = { x: 0, y: 5, z: 0 }; // Initial upward velocity
    private gravity = { x: 0, y: -9.81, z: 0 };
    
    init(): void {
        this.entityId = globalThis.World.createEntity();
        globalThis.World.addComponent(this.entityId, 'Transform', {
            position: { x: 0.0, y: 10.0, z: 0.0 },
            rotation: { x: 0.0, y: 0.0, z: 0.0, w: 1.0 },
            scale: { x: 1.0, y: 1.0, z: 1.0 }
        });
        console.log("Physics object initialized");
    }
    
    update(deltaTime: number): void {
        const transform = globalThis.World.getComponent(this.entityId, 'Transform');
        if (transform) {
            // Apply gravity
            this.velocity.x += this.gravity.x * deltaTime;
            this.velocity.y += this.gravity.y * deltaTime;
            this.velocity.z += this.gravity.z * deltaTime;
            
            // Apply velocity to position
            transform.position.x += this.velocity.x * deltaTime;
            transform.position.y += this.velocity.y * deltaTime;
            transform.position.z += this.velocity.z * deltaTime;
            
            // Simple ground collision
            if (transform.position.y <= 0) {
                transform.position.y = 0;
                this.velocity.y = Math.abs(this.velocity.y) * 0.8; // Bounce with energy loss
            }
            
            // Apply force via Physics API
            globalThis.Physics.applyForce(this.entityId, { x: 0, y: this.gravity.y * 10, z: 0 });
            
            globalThis.World.updateComponent(this.entityId, 'Transform', transform);
        }
    }
    
    destroy(): void {
        console.log("Physics object destroyed");
    }
}"#.to_string(),
        expected_outputs: vec!["Physics object initialized".to_string()],
        api_features: vec![
            "globalThis.Physics".to_string(),
            "globalThis.World".to_string(),
            "Transform".to_string(),
            "getComponent".to_string(),
        ],
        difficulty_level: DifficultyLevel::Intermediate,
        category: ExampleCategory::Physics,
    }
}

/// Advanced event system example
fn event_system_example() -> ExampleScript {
    ExampleScript {
        name: "typescript_event_system".to_string(),
        description: "Advanced event system with custom events".to_string(),
        code: r#"// Custom event interface - Events will be implemented via console logging for now
interface CustomEvent {
    type: string;
    data: any;
}

export class EventManager {
    private eventListeners: Map<string, Function[]> = new Map();
    
    init(): void {
        // Set up custom events
        this.addEventListener("playerDeath", this.onPlayerDeath.bind(this));
        this.addEventListener("powerUpCollected", this.onPowerUpCollected.bind(this));
        
        console.log("Event manager initialized");
    }
    
    update(deltaTime: number): void {
        // Example: Dispatch custom events based on game state
        if (globalThis.Input.isKeyPressed("K")) {
            this.dispatchEvent("playerDeath", { reason: "debug" });
        }
    }
    
    addEventListener(eventType: string, listener: Function): void {
        if (!this.eventListeners.has(eventType)) {
            this.eventListeners.set(eventType, []);
        }
        this.eventListeners.get(eventType)!.push(listener);
    }
    
    dispatchEvent(eventType: string, data?: any): void {
        const listeners = this.eventListeners.get(eventType);
        if (listeners) {
            listeners.forEach(listener => listener({ type: eventType, data }));
        }
        console.log(`Event dispatched: ${eventType}`, data);
    }
    
    private onPlayerDeath(event: CustomEvent): void {
        console.log("Player died:", event.data.reason);
    }
    
    private onPowerUpCollected(event: CustomEvent): void {
        console.log("Power-up collected:", event.data.type);
    }
    
    destroy(): void {
        console.log("Event manager destroyed");
    }
}"#.to_string(),
        expected_outputs: vec!["Event manager initialized".to_string()],
        api_features: vec![
            "globalThis.Input".to_string(),
            "addEventListener".to_string(),
            "dispatchEvent".to_string(),
            "console.log".to_string(),
        ],
        difficulty_level: DifficultyLevel::Advanced,
        category: ExampleCategory::EventSystem,
    }
}

/// Vector math operations example
fn vector_math_example() -> ExampleScript {
    ExampleScript {
        name: "typescript_vector_math".to_string(),
        description: "Vector3 math operations and utilities".to_string(),
        code: r#"// Math operations using plain JavaScript objects
export class VectorMath {
    init(): void {
        console.log("Vector math examples");
        
        // Create vectors as plain objects
        const vectorA = { x: 1, y: 2, z: 3 };
        const vectorB = { x: 4, y: 5, z: 6 };
        
        // Basic operations
        const sum = {
            x: vectorA.x + vectorB.x,
            y: vectorA.y + vectorB.y,
            z: vectorA.z + vectorB.z
        };
        const difference = {
            x: vectorA.x - vectorB.x,
            y: vectorA.y - vectorB.y,
            z: vectorA.z - vectorB.z
        };
        const scaled = {
            x: vectorA.x * 2.0,
            y: vectorA.y * 2.0,
            z: vectorA.z * 2.0
        };
        
        console.log(`Vector A: (${vectorA.x}, ${vectorA.y}, ${vectorA.z})`);
        console.log(`Vector B: (${vectorB.x}, ${vectorB.y}, ${vectorB.z})`);
        console.log(`Sum: (${sum.x}, ${sum.y}, ${sum.z})`);
        console.log(`Difference: (${difference.x}, ${difference.y}, ${difference.z})`);
        console.log(`Scaled: (${scaled.x}, ${scaled.y}, ${scaled.z})`);
        
        // Advanced operations
        const magnitude = Math.sqrt(vectorA.x * vectorA.x + vectorA.y * vectorA.y + vectorA.z * vectorA.z);
        const dotProduct = vectorA.x * vectorB.x + vectorA.y * vectorB.y + vectorA.z * vectorB.z;
        
        console.log(`Magnitude: ${magnitude}`);
        console.log(`Dot product: ${dotProduct}`);
    }
    
    update(deltaTime: number): void {
        // Demonstrate time-based operations with simple timing
        const entityId = globalThis.World.createEntity();
        const transform = globalThis.World.getComponent(entityId, 'Transform');
        
        if (transform) {
            // Create a sine wave motion using Date for timing
            const time = Date.now() / 1000.0;
            const offset = {
                x: Math.sin(time) * 2.0,
                y: Math.cos(time * 0.5) * 1.0,
                z: 0
            };
            
            transform.position.x += offset.x * deltaTime;
            transform.position.y += offset.y * deltaTime;
            
            globalThis.World.updateComponent(entityId, 'Transform', transform);
        }
    }
    
    destroy(): void {
        console.log("Vector math example finished");
    }
}"#.to_string(),
        expected_outputs: vec!["Vector math examples".to_string()],
        api_features: vec![
            "globalThis.World".to_string(),
            "Math".to_string(),
            "Date".to_string(),
        ],
        difficulty_level: DifficultyLevel::Intermediate,
        category: ExampleCategory::BasicSyntax,
    }
}

/// Transform manipulation example
fn transform_manipulation_example() -> ExampleScript {
    ExampleScript {
        name: "typescript_transform_manipulation".to_string(),
        description: "Advanced transform manipulation and parent-child relationships".to_string(),
        code: r#"// Transform manipulation using World API
export class TransformManipulator {
    private entityId: number;
    private originalPosition = { x: 0, y: 0, z: 0 };
    private animationTime: number = 0;
    
    init(): void {
        this.entityId = globalThis.World.createEntity();
        globalThis.World.addComponent(this.entityId, 'Transform', {
            position: { x: 0.0, y: 0.0, z: 0.0 },
            rotation: { x: 0.0, y: 0.0, z: 0.0, w: 1.0 },
            scale: { x: 1.0, y: 1.0, z: 1.0 }
        });
        
        const transform = globalThis.World.getComponent(this.entityId, 'Transform');
        if (transform) {
            this.originalPosition = { ...transform.position };
        }
        
        console.log("Transform manipulator ready");
    }
    
    update(deltaTime: number): void {
        const transform = globalThis.World.getComponent(this.entityId, 'Transform');
        if (!transform) return;
        
        this.animationTime += deltaTime;
        
        // Orbit animation
        const radius = 3.0;
        const speed = 1.0;
        const x = Math.cos(this.animationTime * speed) * radius;
        const z = Math.sin(this.animationTime * speed) * radius;
        
        transform.position = {
            x: this.originalPosition.x + x,
            y: this.originalPosition.y + Math.sin(this.animationTime * 2) * 0.5,
            z: this.originalPosition.z + z
        };
        
        // Rotation animation
        transform.rotation.y = this.animationTime * speed;
        
        // Scale pulsing
        const scaleFactor = 1.0 + Math.sin(this.animationTime * 3) * 0.2;
        transform.scale = { x: scaleFactor, y: scaleFactor, z: scaleFactor };
        
        // Interactive controls
        if (globalThis.Input.isKeyPressed("R")) {
            this.resetTransform();
        }
        
        if (globalThis.Input.isKeyPressed("Q")) {
            transform.rotation.x += deltaTime * 2;
        }
        
        if (globalThis.Input.isKeyPressed("E")) {
            transform.rotation.z += deltaTime * 2;
        }
        
        globalThis.World.updateComponent(this.entityId, 'Transform', transform);
    }
    
    private resetTransform(): void {
        const resetTransform = {
            position: { ...this.originalPosition },
            rotation: { x: 0, y: 0, z: 0, w: 1 },
            scale: { x: 1, y: 1, z: 1 }
        };
        globalThis.World.updateComponent(this.entityId, 'Transform', resetTransform);
        this.animationTime = 0;
        console.log("Transform reset");
    }
    
    destroy(): void {
        this.resetTransform();
        console.log("Transform manipulator destroyed");
    }
}"#.to_string(),
        expected_outputs: vec!["Transform manipulator ready".to_string()],
        api_features: vec![
            "globalThis.World".to_string(),
            "Transform".to_string(),
            "Math".to_string(),
            "globalThis.Input".to_string(),
        ],
        difficulty_level: DifficultyLevel::Intermediate,
        category: ExampleCategory::GameLogic,
    }
}

/// Debugging and logging example
fn debugging_example() -> ExampleScript {
    ExampleScript {
        name: "typescript_debugging".to_string(),
        description: "Debugging techniques and performance monitoring".to_string(),
        code: r#"// World, Input, console, and Math APIs are globally available
export class DebugExample {
    private frameCount: number = 0;
    private lastFPSUpdate: number = 0;
    private fps: number = 0;
    private entityId: number;
    
    init(): void {
        this.entityId = globalThis.World.createEntity();
        globalThis.World.addComponent(this.entityId, 'Transform', {
            position: { x: 0.0, y: 0.0, z: 0.0 },
            rotation: { x: 0.0, y: 0.0, z: 0.0, w: 1.0 },
            scale: { x: 1.0, y: 1.0, z: 1.0 }
        });
        
        console.log("Debug example started");
        console.log("Press D to toggle debug info");
        console.log("Press P to log performance stats");
    }
    
    update(deltaTime: number): void {
        this.frameCount++;
        this.lastFPSUpdate += deltaTime;
        
        // Calculate FPS
        if (this.lastFPSUpdate >= 1.0) {
            this.fps = this.frameCount / this.lastFPSUpdate;
            this.frameCount = 0;
            this.lastFPSUpdate = 0;
        }
        
        // Debug controls
        if (globalThis.Input.isKeyPressed("D")) {
            this.logDebugInfo();
        }
        
        if (globalThis.Input.isKeyPressed("P")) {
            this.logPerformanceStats(deltaTime);
        }
        
        // Error handling example
        try {
            this.riskyOperation();
        } catch (error) {
            console.error("Caught error:", error.message);
        }
    }
    
    private logDebugInfo(): void {
        const transform = globalThis.World.getComponent(this.entityId, 'Transform');
        
        console.log("=== Debug Info ===");
        console.log(`Entity ID: ${this.entityId}`);
        console.log(`FPS: ${this.fps.toFixed(1)}`);
        
        if (transform) {
            console.log(`Position: (${transform.position.x.toFixed(2)}, ${transform.position.y.toFixed(2)}, ${transform.position.z.toFixed(2)})`);
            console.log(`Rotation: (${transform.rotation.x.toFixed(2)}, ${transform.rotation.y.toFixed(2)}, ${transform.rotation.z.toFixed(2)})`);
            console.log(`Scale: (${transform.scale.x.toFixed(2)}, ${transform.scale.y.toFixed(2)}, ${transform.scale.z.toFixed(2)})`);
        }
        
        console.log("==================");
    }
    
    private logPerformanceStats(deltaTime: number): void {
        console.log("=== Performance ===");
        console.log(`Delta Time: ${(deltaTime * 1000).toFixed(2)}ms`);
        console.log(`FPS: ${this.fps.toFixed(1)}`);
        console.log(`Memory: ${this.getMemoryUsage()} MB`);
        console.log("==================");
    }
    
    private getMemoryUsage(): string {
        // Mock memory usage - in real implementation would use actual memory API
        return (Math.random() * 100 + 50).toFixed(1);
    }
    
    private riskyOperation(): void {
        // Simulate potential errors
        if (Math.random() < 0.01) { // 1% chance
            throw new Error("Random error for demonstration");
        }
    }
    
    destroy(): void {
        console.log("Debug example finished");
    }
}"#.to_string(),
        expected_outputs: vec!["Debug example started".to_string()],
        api_features: vec![
            "console.log".to_string(),
            "console.error".to_string(),
            "globalThis.Input".to_string(),
            "globalThis.World".to_string(),
        ],
        difficulty_level: DifficultyLevel::Beginner,
        category: ExampleCategory::Debugging,
    }
}

/// Get examples filtered by category
pub fn get_typescript_examples_by_category(category: ExampleCategory) -> Vec<ExampleScript> {
    get_all_typescript_examples()
        .into_iter()
        .filter(|example| example.category == category)
        .collect()
}

/// Get examples filtered by difficulty
pub fn get_typescript_examples_by_difficulty(difficulty: DifficultyLevel) -> Vec<ExampleScript> {
    get_all_typescript_examples()
        .into_iter()
        .filter(|example| example.difficulty_level == difficulty)
        .collect()
}

/// Get beginner-friendly examples for easy addition to projects
pub fn get_beginner_typescript_examples() -> Vec<ExampleScript> {
    get_typescript_examples_by_difficulty(DifficultyLevel::Beginner)
}

/// Get example by name
pub fn get_typescript_example_by_name(name: &str) -> Option<ExampleScript> {
    get_all_typescript_examples()
        .into_iter()
        .find(|example| example.name == name)
}