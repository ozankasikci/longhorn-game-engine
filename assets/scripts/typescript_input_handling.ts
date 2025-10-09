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
}