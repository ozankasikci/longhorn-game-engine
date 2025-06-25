-- Player Controller Script
-- Example of how to control a player entity in Longhorn Engine

local speed = 5.0
local jump_force = 10.0

function init()
    print("Player controller initialized")
    
    -- Subscribe to input events
    engine.events.subscribe("key_press", on_key_press)
    engine.events.subscribe("key_release", on_key_release)
end

function update(dt)
    -- Get player entity's transform component
    local transform = self.entity:get_component("Transform")
    if not transform then return end
    
    -- Handle movement
    if engine.input.is_key_pressed(engine.input.keys.W) then
        transform.position = transform.position:add(engine.math.vec3(0, 0, -speed * dt))
    end
    if engine.input.is_key_pressed(engine.input.keys.S) then
        transform.position = transform.position:add(engine.math.vec3(0, 0, speed * dt))
    end
    if engine.input.is_key_pressed(engine.input.keys.A) then
        transform.position = transform.position:add(engine.math.vec3(-speed * dt, 0, 0))
    end
    if engine.input.is_key_pressed(engine.input.keys.D) then
        transform.position = transform.position:add(engine.math.vec3(speed * dt, 0, 0))
    end
    
    -- Debug output
    engine.debug.log("debug", "Player position: " .. tostring(transform.position))
end

function on_key_press(event)
    if event.key == engine.input.keys.SPACE then
        -- Jump logic
        engine.debug.log("info", "Player jumped!")
        
        -- Add velocity component for jump
        local velocity = self.entity:get_component("Velocity")
        if velocity then
            velocity.y = jump_force
        end
    end
end

function on_key_release(event)
    -- Handle key release events
end

function on_collision(event)
    engine.debug.log("info", "Player collided with: " .. tostring(event.other))
end