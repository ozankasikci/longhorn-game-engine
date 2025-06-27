// Simple Hello World example - no imports needed for basic console operations
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
}