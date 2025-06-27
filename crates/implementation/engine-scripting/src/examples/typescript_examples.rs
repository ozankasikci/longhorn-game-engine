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
        code: r#"// Types (Entity, Transform, Engine, etc.) are globally available via engine.d.ts
// No imports needed - types are injected by the V8 runtime
export class EntityController {
    private entity: Entity;
    private transform: Transform;
    private rotationSpeed: number = 1.0;
    
    init(): void {
        this.entity = Engine.world.getCurrentEntity();
        this.transform = this.entity.getComponent<Transform>();
        console.log("EntityController initialized");
    }
    
    update(deltaTime: number): void {
        if (this.transform) {
            // Rotate the entity
            this.transform.rotation.y += this.rotationSpeed * deltaTime;
            
            // Simple movement with input
            if (Engine.input.isKeyDown("W")) {
                this.transform.position.z -= deltaTime * 5.0;
            }
            if (Engine.input.isKeyDown("S")) {
                this.transform.position.z += deltaTime * 5.0;
            }
        }
    }
    
    destroy(): void {
        console.log("EntityController destroyed");
    }
}"#.to_string(),
        expected_outputs: vec!["EntityController initialized".to_string()],
        api_features: vec![
            "Engine.world".to_string(),
            "getComponent".to_string(),
            "Engine.input".to_string(),
            "isKeyDown".to_string(),
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
        code: r#"// Engine input API is available globally - no imports required
export class InputHandler {
    init(): void {
        console.log("Input handler ready");
    }
    
    update(deltaTime: number): void {
        // Keyboard input
        if (Engine.input.isKeyDown("Space")) {
            console.log("Space key pressed!");
        }
        
        if (Engine.input.isKeyDown("Escape")) {
            console.log("Escape key pressed!");
        }
        
        // Mouse input
        const mousePos = Engine.input.getMousePosition();
        if (Engine.input.isMouseButtonDown(0)) { // Left click
            console.log(`Mouse clicked at: (${mousePos.x}, ${mousePos.y})`);
        }
        
        // Arrow keys for movement
        if (Engine.input.isKeyDown("ArrowUp")) {
            console.log("Moving up");
        }
        if (Engine.input.isKeyDown("ArrowDown")) {
            console.log("Moving down");
        }
    }
    
    destroy(): void {
        console.log("Input handler stopped");
    }
}"#.to_string(),
        expected_outputs: vec!["Input handler ready".to_string()],
        api_features: vec![
            "Engine.input".to_string(),
            "isKeyDown".to_string(),
            "getMousePosition".to_string(),
            "isMouseButtonDown".to_string(),
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
        code: r#"// Vector3, Transform, and Engine types are globally available
export class PhysicsObject {
    private velocity: Vector3 = new Vector3(0, 0, 0);
    private gravity: Vector3 = new Vector3(0, -9.81, 0);
    private transform: Transform;
    
    init(): void {
        this.transform = Engine.world.getCurrentEntity().getComponent<Transform>();
        this.velocity = new Vector3(0, 5, 0); // Initial upward velocity
        console.log("Physics object initialized");
    }
    
    update(deltaTime: number): void {
        if (this.transform) {
            // Apply gravity
            this.velocity.add(this.gravity.multiply(deltaTime));
            
            // Apply velocity to position
            const deltaPos = this.velocity.multiply(deltaTime);
            this.transform.position.x += deltaPos.x;
            this.transform.position.y += deltaPos.y;
            this.transform.position.z += deltaPos.z;
            
            // Simple ground collision
            if (this.transform.position.y <= 0) {
                this.transform.position.y = 0;
                this.velocity.y = Math.abs(this.velocity.y) * 0.8; // Bounce with energy loss
            }
        }
    }
    
    destroy(): void {
        console.log("Physics object destroyed");
    }
}"#.to_string(),
        expected_outputs: vec!["Physics object initialized".to_string()],
        api_features: vec![
            "Engine.physics".to_string(),
            "Vector3".to_string(),
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
        code: r#"// Custom event interface - Engine.events is globally available
interface CustomEvent {
    type: string;
    data: any;
}

export class EventManager {
    private eventListeners: Map<string, Function[]> = new Map();
    
    init(): void {
        // Listen for built-in engine events
        Engine.events.addEventListener("entityCollision", this.onEntityCollision.bind(this));
        Engine.events.addEventListener("keyPressed", this.onKeyPressed.bind(this));
        
        // Set up custom events
        this.addEventListener("playerDeath", this.onPlayerDeath.bind(this));
        this.addEventListener("powerUpCollected", this.onPowerUpCollected.bind(this));
        
        console.log("Event manager initialized");
    }
    
    update(deltaTime: number): void {
        // Example: Dispatch custom events based on game state
        if (Engine.input.isKeyDown("K")) {
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
        
        // Also dispatch to engine event system
        Engine.events.dispatchEvent(eventType, data);
    }
    
    private onEntityCollision(event: CustomEvent): void {
        console.log("Entity collision detected:", event.data);
    }
    
    private onKeyPressed(event: CustomEvent): void {
        console.log("Key pressed:", event.data.key);
    }
    
    private onPlayerDeath(event: CustomEvent): void {
        console.log("Player died:", event.data.reason);
    }
    
    private onPowerUpCollected(event: CustomEvent): void {
        console.log("Power-up collected:", event.data.type);
    }
    
    destroy(): void {
        // Clean up event listeners
        Engine.events.removeEventListener("entityCollision", this.onEntityCollision);
        Engine.events.removeEventListener("keyPressed", this.onKeyPressed);
        console.log("Event manager destroyed");
    }
}"#.to_string(),
        expected_outputs: vec!["Event manager initialized".to_string()],
        api_features: vec![
            "Engine.events".to_string(),
            "addEventListener".to_string(),
            "dispatchEvent".to_string(),
            "removeEventListener".to_string(),
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
        code: r#"// Vector3, Math, and Engine types available globally via engine.d.ts
export class VectorMath {
    init(): void {
        console.log("Vector math examples");
        
        // Create vectors
        const vectorA = new Vector3(1, 2, 3);
        const vectorB = new Vector3(4, 5, 6);
        
        // Basic operations
        const sum = vectorA.add(vectorB);
        const difference = vectorA.subtract(vectorB);
        const scaled = vectorA.multiply(2.0);
        
        console.log(`Vector A: (${vectorA.x}, ${vectorA.y}, ${vectorA.z})`);
        console.log(`Vector B: (${vectorB.x}, ${vectorB.y}, ${vectorB.z})`);
        console.log(`Sum: (${sum.x}, ${sum.y}, ${sum.z})`);
        console.log(`Difference: (${difference.x}, ${difference.y}, ${difference.z})`);
        console.log(`Scaled: (${scaled.x}, ${scaled.y}, ${scaled.z})`);
        
        // Advanced operations
        const magnitude = vectorA.length();
        const normalized = vectorA.normalize();
        const dotProduct = vectorA.dot(vectorB);
        
        console.log(`Magnitude: ${magnitude}`);
        console.log(`Normalized: (${normalized.x}, ${normalized.y}, ${normalized.z})`);
        console.log(`Dot product: ${dotProduct}`);
    }
    
    update(deltaTime: number): void {
        // Demonstrate time-based vector operations
        const entity = Engine.world.getCurrentEntity();
        const transform = entity.getComponent<Transform>();
        
        if (transform) {
            // Create a sine wave motion
            const time = Engine.time.getElapsedTime();
            const offset = new Vector3(
                Math.sin(time) * 2.0,
                Math.cos(time * 0.5) * 1.0,
                0
            );
            
            transform.position = transform.position.add(offset.multiply(deltaTime));
        }
    }
    
    destroy(): void {
        console.log("Vector math example finished");
    }
}"#.to_string(),
        expected_outputs: vec!["Vector math examples".to_string()],
        api_features: vec![
            "Vector3".to_string(),
            "Math".to_string(),
            "Engine.time".to_string(),
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
        code: r#"// Transform, Vector3, Math, and Engine are globally available
export class TransformManipulator {
    private transform: Transform;
    private originalPosition: Vector3;
    private animationTime: number = 0;
    
    init(): void {
        this.transform = Engine.world.getCurrentEntity().getComponent<Transform>();
        this.originalPosition = this.transform.position.clone();
        console.log("Transform manipulator ready");
    }
    
    update(deltaTime: number): void {
        if (!this.transform) return;
        
        this.animationTime += deltaTime;
        
        // Orbit animation
        const radius = 3.0;
        const speed = 1.0;
        const x = Math.cos(this.animationTime * speed) * radius;
        const z = Math.sin(this.animationTime * speed) * radius;
        
        this.transform.position = new Vector3(
            this.originalPosition.x + x,
            this.originalPosition.y + Math.sin(this.animationTime * 2) * 0.5,
            this.originalPosition.z + z
        );
        
        // Rotation animation
        this.transform.rotation.y = this.animationTime * speed;
        
        // Scale pulsing
        const scaleFactor = 1.0 + Math.sin(this.animationTime * 3) * 0.2;
        this.transform.scale = new Vector3(scaleFactor, scaleFactor, scaleFactor);
        
        // Interactive controls
        if (Engine.input.isKeyDown("R")) {
            this.resetTransform();
        }
        
        if (Engine.input.isKeyDown("Q")) {
            this.transform.rotation.x += deltaTime * 2;
        }
        
        if (Engine.input.isKeyDown("E")) {
            this.transform.rotation.z += deltaTime * 2;
        }
    }
    
    private resetTransform(): void {
        this.transform.position = this.originalPosition.clone();
        this.transform.rotation = new Vector3(0, 0, 0);
        this.transform.scale = new Vector3(1, 1, 1);
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
            "Transform".to_string(),
            "Vector3".to_string(),
            "Math".to_string(),
            "Engine.input".to_string(),
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
        code: r#"// Engine, console, and Math APIs are globally available
export class DebugExample {
    private frameCount: number = 0;
    private lastFPSUpdate: number = 0;
    private fps: number = 0;
    
    init(): void {
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
        if (Engine.input.isKeyDown("D")) {
            this.logDebugInfo();
        }
        
        if (Engine.input.isKeyDown("P")) {
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
        const entity = Engine.world.getCurrentEntity();
        const transform = entity.getComponent<Transform>();
        
        console.log("=== Debug Info ===");
        console.log(`Entity ID: ${entity.id}`);
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
            "Engine.input".to_string(),
            "Math".to_string(),
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