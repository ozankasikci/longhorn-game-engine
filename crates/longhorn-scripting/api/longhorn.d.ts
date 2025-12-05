// Longhorn Game Engine TypeScript API

declare module "longhorn" {
    export interface Vec2 {
        x: number;
        y: number;
    }

    export interface Transform {
        position: Vec2;
        rotation: number;
        scale: Vec2;
    }

    export interface Sprite {
        texture: string;
        color: [number, number, number, number];
        flipX: boolean;
        flipY: boolean;
        zIndex: number;
    }

    export interface Entity {
        id: number;
        get<T>(component: ComponentType<T>): T;
        set<T>(component: ComponentType<T>, value: T): void;
        has<T>(component: ComponentType<T>): boolean;
    }

    export interface ComponentType<T> {
        readonly name: string;
    }

    export const Transform: ComponentType<Transform>;
    export const Sprite: ComponentType<Sprite>;

    export interface EntityBuilder {
        with<T>(component: ComponentType<T>, value: Partial<T>): EntityBuilder;
        build(): Entity;
    }

    export interface World {
        spawn(name: string): EntityBuilder;
        find(name: string): Entity | null;
        despawn(entity: Entity): void;
    }

    export interface Input {
        isTouching(): boolean;
        justPressed(): boolean;
        justReleased(): boolean;
        position(): Vec2 | null;
    }

    export const input: Input;
}
