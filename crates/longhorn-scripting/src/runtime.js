// Longhorn Runtime API
((globalThis) => {
    const core = Deno.core;

    // Console implementation
    globalThis.console = {
        log: (...args) => core.ops.op_log("info", args.map(String).join(" ")),
        info: (...args) => core.ops.op_log("info", args.map(String).join(" ")),
        warn: (...args) => core.ops.op_log("warn", args.map(String).join(" ")),
        error: (...args) => core.ops.op_log("error", args.map(String).join(" ")),
        debug: (...args) => core.ops.op_log("debug", args.map(String).join(" ")),
    };

    // Entity class
    class Entity {
        constructor(id) {
            this.id = id;
        }

        get(componentType) {
            // Will be implemented with op_get_component
            return null;
        }

        set(componentType, value) {
            // Will be implemented with op_set_component
        }

        has(componentType) {
            // Will be implemented with op_has_component
            return false;
        }
    }

    // Component type markers
    const Transform = { name: "Transform" };
    const Sprite = { name: "Sprite" };
    const Name = { name: "Name" };

    // World API
    const world = {
        spawn(name) {
            // Will be implemented with op_spawn_entity
            return null;
        },
        find(name) {
            // Will be implemented with op_find_entity
            return null;
        },
        despawn(entity) {
            // Will be implemented with op_despawn_entity
        },
    };

    // Input API
    const input = {
        isTouching() {
            // Will be implemented with op_input_is_touching
            return false;
        },
        position() {
            // Will be implemented with op_input_position
            return null;
        },
        justPressed() {
            return false;
        },
        justReleased() {
            return false;
        },
    };

    // Expose to global scope
    globalThis.Entity = Entity;
    globalThis.Transform = Transform;
    globalThis.Sprite = Sprite;
    globalThis.Name = Name;
    globalThis.world = world;
    globalThis.input = input;

})(globalThis);
