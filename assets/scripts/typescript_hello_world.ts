

export class HelloWorld {
    private entity: any;
    
    init(): void {
        try {
            console.log("🔥 DIRECT ECS TEST - Init started");
            
            // Get entity reference (Unity-style approach)
            this.entity = Engine.world.getCurrentEntity();
            console.log("🔥 DIRECT ECS TEST - Got entity, ID:", this.entity.id());
            
            // Debug: List all methods on the entity object
            console.log("🔍 Entity object properties:");
            for (let prop in this.entity) {
                console.log("  " + prop + ":", typeof this.entity[prop]);
            }
            
            // Test if getPosition method exists
            if (typeof this.entity.getPosition === 'function') {
                console.log("✅ getPosition method is available");
                const currentPos = this.entity.getPosition();
                console.log("📍 Current position:", currentPos.x, currentPos.y, currentPos.z);
            } else {
                console.log("❌ getPosition method is NOT available");
                console.log("   this.entity.getPosition type:", typeof this.entity.getPosition);
            }
            
            console.log("🔥 DIRECT ECS TEST - Init completed successfully");
        } catch (error) {
            console.error("🚨 ERROR in init():", error.toString());
            throw error;
        }
    }
    
    update(deltaTime: number): void {
        console.log('🚀 DIRECT ECS UPDATE');
        
        // Get current position
        const pos = this.entity.getPosition();
        
        // Calculate new position
        const newX = pos.x + deltaTime * 2.0;
        const newY = pos.y;
        const newZ = pos.z + deltaTime * 5.0;
        
        // Set new position using direct ECS call (Unity-style)
        this.entity.setPosition(newX, newY, newZ);
        
        console.log("🎯 Moved to:", newX, newY, newZ);
    }
    
    destroy(): void {
        console.log("🔥 DIRECT ECS TEST - Destroy called");
    }
}