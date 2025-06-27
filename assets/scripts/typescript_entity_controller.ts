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
}