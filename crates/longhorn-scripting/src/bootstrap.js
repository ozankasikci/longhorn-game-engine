// Longhorn Runtime Bootstrap
// Injected before any user scripts
// Uses QuickJS via rquickjs

// Component type markers (used for self.get(Transform))
const Transform = { name: "Transform" };
const Sprite = { name: "Sprite" };

// Entity class - passed as 'self' to lifecycle methods
class Entity {
  constructor(id) {
    this.id = id;
  }

  get(componentType) {
    // TODO: Wire to component accessor when implemented
    return null;
  }

  set(componentType, value) {
    // TODO: Wire to component setter when implemented
  }

  has(componentType) {
    // TODO: Wire to component check when implemented
    return false;
  }
}

// Script class registry (populated when scripts are loaded)
const __scripts = {};

// Script instance registry (populated when entities get scripts)
const __instances = {};

// Console override to route through Rust logging
// Uses __longhorn_log registered by js_runtime.rs
const __console_log = (...args) => {
  try {
    __longhorn_log("info", args.map(a => String(a)).join(" "));
  } catch (e) {
    // Fallback if op not available
  }
};

const __console_error = (...args) => {
  try {
    __longhorn_log("error", args.map(a => String(a)).join(" "));
  } catch (e) {}
};

const __console_warn = (...args) => {
  try {
    __longhorn_log("warn", args.map(a => String(a)).join(" "));
  } catch (e) {}
};

const __console_debug = (...args) => {
  try {
    __longhorn_log("debug", args.map(a => String(a)).join(" "));
  } catch (e) {}
};

// Override global console
globalThis.console = {
  log: __console_log,
  info: __console_log,
  error: __console_error,
  warn: __console_warn,
  debug: __console_debug,
};

// Make classes globally available
globalThis.Entity = Entity;
globalThis.Transform = Transform;
globalThis.Sprite = Sprite;
globalThis.__scripts = __scripts;
globalThis.__instances = __instances;

// Engine API for scripts
globalThis.engine = globalThis.engine || {};

// Emit a global event
globalThis.engine.emit = function(eventName, data) {
  const dataJson = JSON.stringify(data || {});
  __longhorn_emit_event(eventName, dataJson);
};

// Send an event to a specific entity
globalThis.engine.sendTo = function(entityId, eventName, data) {
  const dataJson = JSON.stringify(data || {});
  __longhorn_emit_to_entity(entityId, eventName, dataJson);
};

"bootstrap loaded";
