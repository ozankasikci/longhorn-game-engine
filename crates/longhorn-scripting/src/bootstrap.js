// Longhorn Runtime Bootstrap
// Injected before any user scripts

// Component type markers (used for self.get(Transform))
const Transform = { name: "Transform" };
const Sprite = { name: "Sprite" };

// Entity class - passed as 'self' to lifecycle methods
class Entity {
  constructor(id) {
    this.id = id;
  }

  get(componentType) {
    // TODO: Wire to op_get_component when implemented
    return null;
  }

  set(componentType, value) {
    // TODO: Wire to op_set_component when implemented
  }

  has(componentType) {
    // TODO: Wire to op_has_component when implemented
    return false;
  }
}

// Script class registry (populated when scripts are loaded)
const __scripts = {};

// Script instance registry (populated when entities get scripts)
const __instances = {};

// Console override to route through Rust logging
const __console_log = (...args) => {
  try {
    Deno.core.ops.op_log("info", args.map(a => String(a)).join(" "));
  } catch (e) {
    // Fallback if op not available
  }
};

const __console_error = (...args) => {
  try {
    Deno.core.ops.op_log("error", args.map(a => String(a)).join(" "));
  } catch (e) {}
};

const __console_warn = (...args) => {
  try {
    Deno.core.ops.op_log("warn", args.map(a => String(a)).join(" "));
  } catch (e) {}
};

// Override global console
globalThis.console = {
  log: __console_log,
  info: __console_log,
  error: __console_error,
  warn: __console_warn,
  debug: __console_log,
};

// Make classes globally available
globalThis.Entity = Entity;
globalThis.Transform = Transform;
globalThis.Sprite = Sprite;
globalThis.__scripts = __scripts;
globalThis.__instances = __instances;

// Engine API for scripts
globalThis.engine = globalThis.engine || {};
globalThis.engine.emit = function(eventName, data) {
    Deno.core.ops.op_emit_event(eventName, data || {});
};
globalThis.engine.sendTo = function(entityId, eventName, data) {
    Deno.core.ops.op_emit_to_entity(entityId, eventName, data || {});
};

"bootstrap loaded";
