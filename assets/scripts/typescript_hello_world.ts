// Simple Hello World example - no imports needed for basic console operations
export class HelloWorld {
    init(): void {
        console.log("Hello, World!");
        console.log("Welcome to Longhorn Game Engine TypeScript scripting!");
        console.log('omfg')
    }
    
    update(deltaTime: number): void {
        console.log('tf is this man')
    }
    
    destroy(): void {
        console.log("Goodbye from TypeScript!");
    }
}