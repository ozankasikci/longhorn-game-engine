-- Enemy AI Script
-- Example of simple enemy behavior in Longhorn Engine

local state = "idle"
local target = nil
local patrol_speed = 2.0
local chase_speed = 4.0
local attack_range = 2.0
local sight_range = 10.0

function init()
    print("Enemy AI initialized")
    
    -- Set initial state
    state = "patrol"
    
    -- Subscribe to events
    engine.events.subscribe("player_spotted", on_player_spotted)
end

function update(dt)
    local transform = self.entity:get_component("Transform")
    if not transform then return end
    
    -- State machine
    if state == "idle" then
        -- Do nothing
    elseif state == "patrol" then
        patrol(dt, transform)
    elseif state == "chase" then
        chase_player(dt, transform)
    elseif state == "attack" then
        attack_player(dt, transform)
    end
    
    -- Check for player in sight
    check_for_player(transform)
end

function patrol(dt, transform)
    -- Simple patrol behavior - move in a pattern
    local time = engine.time.total_time
    local x = math.sin(time) * patrol_speed * dt
    local z = math.cos(time) * patrol_speed * dt
    
    transform.position = transform.position:add(engine.math.vec3(x, 0, z))
end

function chase_player(dt, transform)
    if not target then
        state = "patrol"
        return
    end
    
    local target_transform = target:get_component("Transform")
    if not target_transform then return end
    
    -- Calculate direction to player
    local direction = target_transform.position:sub(transform.position):normalize()
    
    -- Move towards player
    transform.position = transform.position:add(direction:mul(chase_speed * dt))
    
    -- Check if in attack range
    local distance = target_transform.position:distance(transform.position)
    if distance <= attack_range then
        state = "attack"
    elseif distance > sight_range then
        state = "patrol"
        target = nil
    end
end

function attack_player(dt, transform)
    if not target then
        state = "patrol"
        return
    end
    
    -- Attack logic
    engine.debug.log("info", "Enemy attacking player!")
    
    -- Emit attack event
    engine.events.emit("enemy_attack", {
        enemy = self.entity,
        target = target,
        damage = 10
    })
end

function check_for_player(transform)
    -- Query for entities with Player component within sight range
    for entity, player_transform in engine.world:query("Transform, Player") do
        local distance = player_transform.position:distance(transform.position)
        
        if distance <= sight_range then
            if state ~= "chase" and state ~= "attack" then
                on_player_spotted({ player = entity })
            end
        end
    end
end

function on_player_spotted(event)
    target = event.player
    state = "chase"
    engine.debug.log("info", "Enemy spotted player!")
end