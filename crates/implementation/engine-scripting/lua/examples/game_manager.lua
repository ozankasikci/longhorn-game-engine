-- Game Manager Script
-- Example of managing game state and spawning entities

local score = 0
local game_state = "menu"
local spawn_timer = 0
local spawn_interval = 3.0

function init()
    print("Game Manager initialized")
    
    -- Initialize game
    game_state = "playing"
    score = 0
    
    -- Subscribe to events
    engine.events.subscribe("enemy_defeated", on_enemy_defeated)
    engine.events.subscribe("player_died", on_player_died)
    
    -- Create initial entities
    spawn_player()
end

function update(dt)
    if game_state ~= "playing" then
        return
    end
    
    -- Update spawn timer
    spawn_timer = spawn_timer + dt
    
    -- Spawn enemies periodically
    if spawn_timer >= spawn_interval then
        spawn_enemy()
        spawn_timer = 0
        
        -- Increase difficulty over time
        spawn_interval = math.max(1.0, spawn_interval - 0.1)
    end
    
    -- Update UI
    update_score_display()
end

function spawn_player()
    local player = engine.world:create_entity({
        Transform = {
            position = engine.math.vec3(0, 1, 0),
            rotation = engine.math.quat(0, 0, 0, 1),
            scale = engine.math.vec3(1, 1, 1)
        },
        Player = {},
        Health = {
            current = 100,
            max = 100
        }
    })
    
    -- Attach player controller script
    player:add_component("LuaScript", {
        path = "player_controller.lua"
    })
    
    engine.debug.log("info", "Player spawned")
end

function spawn_enemy()
    -- Random spawn position
    local x = math.random(-20, 20)
    local z = math.random(-20, 20)
    
    local enemy = engine.world:create_entity({
        Transform = {
            position = engine.math.vec3(x, 1, z),
            rotation = engine.math.quat(0, 0, 0, 1),
            scale = engine.math.vec3(1, 1, 1)
        },
        Enemy = {},
        Health = {
            current = 50,
            max = 50
        }
    })
    
    -- Attach AI script
    enemy:add_component("LuaScript", {
        path = "enemy_ai.lua"
    })
    
    engine.debug.log("info", "Enemy spawned at: " .. x .. ", " .. z)
end

function on_enemy_defeated(event)
    score = score + 100
    engine.debug.log("info", "Enemy defeated! Score: " .. score)
    
    -- Spawn particle effect
    spawn_explosion(event.position)
    
    -- Play sound
    engine.assets.load_sound("explosion.ogg"):play()
end

function on_player_died(event)
    game_state = "game_over"
    engine.debug.log("info", "Game Over! Final Score: " .. score)
    
    -- Show game over screen
    show_game_over_screen()
end

function update_score_display()
    -- Update UI text (when UI system is available)
    -- engine.ui.set_text("score_text", "Score: " .. score)
end

function spawn_explosion(position)
    -- Create particle effect entity
    local explosion = engine.world:create_entity({
        Transform = {
            position = position,
            rotation = engine.math.quat(0, 0, 0, 1),
            scale = engine.math.vec3(1, 1, 1)
        },
        ParticleEmitter = {
            texture = "explosion.png",
            emit_rate = 100,
            lifetime = 1.0,
            start_size = 0.5,
            end_size = 2.0
        }
    })
end

function show_game_over_screen()
    -- Display game over UI
    engine.debug.log("info", "=== GAME OVER ===")
    engine.debug.log("info", "Final Score: " .. score)
    engine.debug.log("info", "Press ENTER to restart")
end