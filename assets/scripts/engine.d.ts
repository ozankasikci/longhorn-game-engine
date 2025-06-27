/**
 * Longhorn Game Engine TypeScript API
 * 
 * This file provides TypeScript type definitions for the engine's scripting API.
 * Types are available globally in TypeScript scripts without imports.
 * 
 * Usage in scripts:
 * - Engine API: Engine.world.getCurrentEntity()
 * - Math: Math.sin(), new Vector3(1, 2, 3)
 * - Console: console.log()
 * 
 * All types are injected by the V8 runtime and available without explicit imports.
 */

// Entity Component System Types
interface Entity {
    id(): number;
    getComponent<T>(): T | null;
    addComponent<T>(component: T): void;
    removeComponent<T>(): boolean;
}

interface Transform {
    position: Vector3;
    rotation: Vector3;
    scale: Vector3;
}

class Vector3 {
    x: number;
    y: number;
    z: number;

    constructor(x: number, y: number, z: number);
    
    add(other: Vector3): Vector3;
    subtract(other: Vector3): Vector3;
    multiply(scalar: number): Vector3;
    divide(scalar: number): Vector3;
    dot(other: Vector3): number;
    cross(other: Vector3): Vector3;
    length(): number;
    normalize(): Vector3;
    clone(): Vector3;
}

// Rendering Components
interface Mesh {
    vertices: number[];
    indices: number[];
    uvs: number[];
    normals: number[];
}

interface Material {
    albedo: Vector3;
    metallic: number;
    roughness: number;
    emissive: Vector3;
}

interface MeshRenderer {
    mesh: Mesh | null;
    material: Material | null;
    visible: boolean;
}

interface Camera {
    fov: number;
    near: number;
    far: number;
    aspect_ratio: number;
}

interface Light {
    light_type: string;
    color: Vector3;
    intensity: number;
    range: number;
}

// Physics Types
interface RaycastHit {
    entity: Entity;
    point: Vector3;
    normal: Vector3;
    distance: number;
}

// Main Engine API
declare const Engine: {
    world: {
        getCurrentEntity(): Entity;
        getEntity(id: number): Entity | null;
        createEntity(): Entity;
        destroyEntity(entity: Entity): void;
    };
    
    input: {
        isKeyDown(key: string): boolean;
        isKeyPressed(key: string): boolean;
        isKeyReleased(key: string): boolean;
        isMouseButtonDown(button: number): boolean;
        isMouseButtonPressed(button: number): boolean;
        isMouseButtonReleased(button: number): boolean;
        getMousePosition(): { x: number; y: number };
        getMouseDelta(): { x: number; y: number };
    };
    
    physics: {
        addRigidBody(entity: Entity, bodyType: string, mass: number): void;
        removeRigidBody(entity: Entity): void;
        applyForce(entity: Entity, force: Vector3): void;
        applyImpulse(entity: Entity, impulse: Vector3): void;
        raycast(origin: Vector3, direction: Vector3, maxDistance: number): RaycastHit | null;
    };
    
    events: {
        addEventListener(eventType: string, callback: (event: any) => void): void;
        removeEventListener(eventType: string, callback: (event: any) => void): void;
        dispatchEvent(eventType: string, data?: any): void;
    };
    
    time: {
        getElapsedTime(): number;
        getDeltaTime(): number;
        getTimeScale(): number;
        setTimeScale(scale: number): void;
    };
    
    debug: {
        log(message: string): void;
        warn(message: string): void;
        error(message: string): void;
        drawLine(start: Vector3, end: Vector3, color: Vector3): void;
        drawSphere(center: Vector3, radius: number, color: Vector3): void;
    };
};

// Standard JavaScript APIs (enhanced for engine context)
declare const console: {
    log(...args: any[]): void;
    warn(...args: any[]): void;
    error(...args: any[]): void;
};

declare const Math: {
    abs(x: number): number;
    sin(x: number): number;
    cos(x: number): number;
    tan(x: number): number;
    sqrt(x: number): number;
    pow(x: number, y: number): number;
    floor(x: number): number;
    ceil(x: number): number;
    round(x: number): number;
    min(...values: number[]): number;
    max(...values: number[]): number;
    random(): number;
    PI: number;
};