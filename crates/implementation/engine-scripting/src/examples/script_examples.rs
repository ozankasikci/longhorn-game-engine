//! Collection of example scripts for documentation and testing

use super::{ExampleScript, DifficultyLevel, ExampleCategory};

/// Get all available example scripts
pub fn get_all_examples() -> Vec<ExampleScript> {
    vec![
        // Basic Syntax Examples
        hello_world_example(),
        variables_and_types_example(),
        functions_example(),
        loops_and_conditionals_example(),
        
        // Input Handling Examples
        basic_input_example(),
        advanced_input_example(),
        input_combinations_example(),
        
        // Physics Examples
        basic_physics_example(),
        physics_simulation_example(),
        collision_detection_example(),
        
        // Event System Examples
        basic_events_example(),
        custom_events_example(),
        
        // Debugging Examples
        debugging_basics_example(),
        advanced_debugging_example(),
        
        // Performance Examples
        basic_profiling_example(),
        performance_optimization_example(),
        
        // Game Logic Examples
        simple_game_loop_example(),
        player_controller_example(),
        game_state_management_example(),
        
        // Integration Examples
        complete_game_example(),
        modding_example(),
    ]
}

// ===== BASIC SYNTAX EXAMPLES =====

fn hello_world_example() -> ExampleScript {
    ExampleScript {
        name: "hello_world".to_string(),
        description: "The classic Hello World example".to_string(),
        code: r#"
-- Hello World in Lua
print("Hello, World!")
print("Welcome to Longhorn Game Engine scripting!")

-- Return success status
return "Hello World executed successfully"
        "#.to_string(),
        expected_outputs: vec!["Hello, World!".to_string()],
        api_features: vec!["print".to_string()],
        difficulty_level: DifficultyLevel::Beginner,
        category: ExampleCategory::BasicSyntax,
    }
}

fn variables_and_types_example() -> ExampleScript {
    ExampleScript {
        name: "variables_and_types".to_string(),
        description: "Working with variables and data types".to_string(),
        code: r#"
-- Numbers
local health = 100
local speed = 5.5
local damage = 25

-- Strings
local player_name = "Hero"
local weapon = "Sword"

-- Booleans
local is_alive = true
local has_key = false

-- Tables (arrays and objects)
local position = {x = 10, y = 20, z = 5}
local inventory = {"potion", "key", "scroll"}

-- Print values
print("Player: " .. player_name)
print("Health: " .. health)
print("Position: (" .. position.x .. ", " .. position.y .. ", " .. position.z .. ")")
print("First item: " .. inventory[1])

-- Type checking
print("Type of health: " .. type(health))
print("Type of player_name: " .. type(player_name))
print("Type of position: " .. type(position))

return "Variables demo complete"
        "#.to_string(),
        expected_outputs: vec![
            "Player: Hero".to_string(),
            "Health: 100".to_string(),
            "Variables demo complete".to_string()
        ],
        api_features: vec!["print".to_string()],
        difficulty_level: DifficultyLevel::Beginner,
        category: ExampleCategory::BasicSyntax,
    }
}

fn functions_example() -> ExampleScript {
    ExampleScript {
        name: "functions".to_string(),
        description: "Defining and using functions".to_string(),
        code: r#"
-- Simple function
function greet(name)
    return "Hello, " .. name .. "!"
end

-- Function with multiple parameters
function calculate_damage(base_damage, multiplier, armor)
    local raw_damage = base_damage * multiplier
    local final_damage = math.max(1, raw_damage - armor)
    return final_damage
end

-- Function with multiple return values
function get_player_stats()
    return 100, 50, 25  -- health, mana, stamina
end

-- Local function
local function is_critical_hit(chance)
    return math.random() < chance
end

-- Using functions
local greeting = greet("Player")
print(greeting)

local damage = calculate_damage(50, 1.5, 10)
print("Damage dealt: " .. damage)

local health, mana, stamina = get_player_stats()
print("Stats - Health: " .. health .. ", Mana: " .. mana .. ", Stamina: " .. stamina)

if is_critical_hit(0.2) then
    print("Critical hit!")
else
    print("Normal hit")
end

return "Functions demo complete"
        "#.to_string(),
        expected_outputs: vec![
            "Hello, Player!".to_string(),
            "Functions demo complete".to_string()
        ],
        api_features: vec!["print".to_string()],
        difficulty_level: DifficultyLevel::Beginner,
        category: ExampleCategory::BasicSyntax,
    }
}

fn loops_and_conditionals_example() -> ExampleScript {
    ExampleScript {
        name: "loops_and_conditionals".to_string(),
        description: "Control flow with loops and conditionals".to_string(),
        code: r#"
-- Conditional statements
local health = 75

if health > 80 then
    print("Health is good!")
elseif health > 50 then
    print("Health is okay")
elseif health > 20 then
    print("Health is low!")
else
    print("Critical health!")
end

-- For loops
print("Counting down:")
for i = 5, 1, -1 do
    print(i)
end

-- While loop
local energy = 10
while energy > 0 do
    print("Energy remaining: " .. energy)
    energy = energy - 2
end

-- Iterating over tables
local items = {"sword", "shield", "potion", "key"}
print("Inventory:")
for i, item in ipairs(items) do
    print(i .. ": " .. item)
end

-- Iterating over table keys
local stats = {strength = 15, dexterity = 12, intelligence = 8}
print("Character stats:")
for stat, value in pairs(stats) do
    print(stat .. ": " .. value)
end

return "Control flow demo complete"
        "#.to_string(),
        expected_outputs: vec![
            "Health is okay".to_string(),
            "Counting down:".to_string(),
            "Control flow demo complete".to_string()
        ],
        api_features: vec!["print".to_string()],
        difficulty_level: DifficultyLevel::Beginner,
        category: ExampleCategory::BasicSyntax,
    }
}

// ===== INPUT HANDLING EXAMPLES =====

fn basic_input_example() -> ExampleScript {
    ExampleScript {
        name: "basic_input".to_string(),
        description: "Basic keyboard and mouse input handling".to_string(),
        code: r#"
-- Check if specific keys are pressed
if is_key_pressed("W") then
    print("Moving forward")
end

if is_key_pressed("Space") then
    print("Jumping!")
end

-- Get mouse position
local mouse_x, mouse_y = get_mouse_position()
print("Mouse position: " .. mouse_x .. ", " .. mouse_y)

-- Check mouse buttons
if is_mouse_button_pressed("Left") then
    print("Left mouse button pressed")
end

if is_mouse_button_pressed("Right") then
    print("Right mouse button pressed")
end

return "Basic input check complete"
        "#.to_string(),
        expected_outputs: vec!["Basic input check complete".to_string()],
        api_features: vec![
            "is_key_pressed".to_string(),
            "get_mouse_position".to_string(),
            "is_mouse_button_pressed".to_string()
        ],
        difficulty_level: DifficultyLevel::Beginner,
        category: ExampleCategory::InputHandling,
    }
}

fn advanced_input_example() -> ExampleScript {
    ExampleScript {
        name: "advanced_input".to_string(),
        description: "Key binding and input callbacks".to_string(),
        code: r#"
-- Bind keys to actions
bind_key("Enter", function()
    print("Enter key pressed - confirming action")
end)

bind_key("Escape", function()
    print("Escape key pressed - canceling")
end)

-- Bind mouse buttons
bind_mouse_button("Left", function()
    print("Left click - attacking")
end)

bind_mouse_button("Right", function()
    print("Right click - blocking")
end)

-- Complex key combinations (would need additional API)
print("Advanced input bindings configured")

return "Advanced input setup complete"
        "#.to_string(),
        expected_outputs: vec!["Advanced input setup complete".to_string()],
        api_features: vec![
            "bind_key".to_string(),
            "bind_mouse_button".to_string()
        ],
        difficulty_level: DifficultyLevel::Intermediate,
        category: ExampleCategory::InputHandling,
    }
}

fn input_combinations_example() -> ExampleScript {
    ExampleScript {
        name: "input_combinations".to_string(),
        description: "Handling key combinations and sequences".to_string(),
        code: r#"
-- Check for key combinations
local function check_key_combo()
    local ctrl_pressed = is_key_pressed("Ctrl")
    local s_pressed = is_key_pressed("S")
    
    if ctrl_pressed and s_pressed then
        print("Save command (Ctrl+S)")
        return true
    end
    
    return false
end

-- Movement input handling
local function handle_movement()
    local move_x = 0
    local move_y = 0
    
    if is_key_pressed("W") or is_key_pressed("Up") then
        move_y = move_y + 1
    end
    if is_key_pressed("S") or is_key_pressed("Down") then
        move_y = move_y - 1
    end
    if is_key_pressed("A") or is_key_pressed("Left") then
        move_x = move_x - 1
    end
    if is_key_pressed("D") or is_key_pressed("Right") then
        move_x = move_x + 1
    end
    
    if move_x ~= 0 or move_y ~= 0 then
        print("Moving: " .. move_x .. ", " .. move_y)
    end
end

-- Execute input checks
check_key_combo()
handle_movement()

return "Input combinations demo complete"
        "#.to_string(),
        expected_outputs: vec!["Input combinations demo complete".to_string()],
        api_features: vec!["is_key_pressed".to_string()],
        difficulty_level: DifficultyLevel::Intermediate,
        category: ExampleCategory::InputHandling,
    }
}

// ===== PHYSICS EXAMPLES =====

fn basic_physics_example() -> ExampleScript {
    ExampleScript {
        name: "basic_physics".to_string(),
        description: "Creating and manipulating rigid bodies".to_string(),
        code: r#"
-- Create a dynamic rigid body (a falling object)
local falling_box = add_rigid_body(0, 10, 0, "Dynamic", 1.0)
print("Created falling box with handle: " .. falling_box)

-- Create a static ground
local ground = add_rigid_body(0, 0, 0, "Static", 0.0)
print("Created ground with handle: " .. ground)

-- Apply an upward force to the box
apply_force(falling_box, 0, 50, 0)
print("Applied upward force to box")

-- Apply an impulse (instant velocity change)
apply_impulse(falling_box, 5, 0, 0)
print("Applied horizontal impulse to box")

-- Check gravity settings
local gx, gy, gz = get_gravity()
print("Current gravity: " .. gx .. ", " .. gy .. ", " .. gz)

return "Basic physics setup complete"
        "#.to_string(),
        expected_outputs: vec![
            "Created falling box".to_string(),
            "Applied upward force".to_string(),
            "Basic physics setup complete".to_string()
        ],
        api_features: vec![
            "add_rigid_body".to_string(),
            "apply_force".to_string(),
            "apply_impulse".to_string(),
            "get_gravity".to_string()
        ],
        difficulty_level: DifficultyLevel::Intermediate,
        category: ExampleCategory::Physics,
    }
}

fn physics_simulation_example() -> ExampleScript {
    ExampleScript {
        name: "physics_simulation".to_string(),
        description: "Running a physics simulation with multiple objects".to_string(),
        code: r#"
-- Create multiple objects
local objects = {}

-- Create a stack of boxes
for i = 1, 5 do
    local box = add_rigid_body(0, i * 2, 0, "Dynamic", 1.0)
    table.insert(objects, box)
    print("Created box " .. i .. " at height " .. (i * 2))
end

-- Create ground
local ground = add_rigid_body(0, -1, 0, "Static", 0.0)

-- Apply random forces to objects
for i, obj in ipairs(objects) do
    local force_x = (math.random() - 0.5) * 20
    local force_y = math.random() * 10
    local force_z = (math.random() - 0.5) * 20
    
    apply_force(obj, force_x, force_y, force_z)
    print("Applied force (" .. force_x .. ", " .. force_y .. ", " .. force_z .. ") to box " .. i)
end

-- Simulate explosion effect
local explosion_center = objects[3] -- Middle object
apply_impulse(explosion_center, 0, 100, 0)
print("Applied explosion impulse!")

return "Physics simulation complete"
        "#.to_string(),
        expected_outputs: vec![
            "Created box 1".to_string(),
            "Applied explosion impulse!".to_string(),
            "Physics simulation complete".to_string()
        ],
        api_features: vec![
            "add_rigid_body".to_string(),
            "apply_force".to_string(),
            "apply_impulse".to_string()
        ],
        difficulty_level: DifficultyLevel::Advanced,
        category: ExampleCategory::Physics,
    }
}

fn collision_detection_example() -> ExampleScript {
    ExampleScript {
        name: "collision_detection".to_string(),
        description: "Detecting collisions and raycasting".to_string(),
        code: r#"
-- Create objects for collision testing
local moving_object = add_rigid_body(-5, 5, 0, "Dynamic", 1.0)
local static_object = add_rigid_body(5, 5, 0, "Static", 0.0)

-- Move the object towards the static one
apply_impulse(moving_object, 20, 0, 0)
print("Launched object towards target")

-- Perform a raycast
local hit = raycast(0, 0, 0, 1, 0, 0, 10.0)
if hit then
    print("Raycast hit something!")
    print("Hit point: " .. hit.point.x .. ", " .. hit.point.y .. ", " .. hit.point.z)
    print("Hit distance: " .. hit.distance)
else
    print("Raycast didn't hit anything")
end

-- Set up collision callback (simplified)
print("Collision detection system active")

return "Collision detection setup complete"
        "#.to_string(),
        expected_outputs: vec![
            "Launched object towards target".to_string(),
            "Collision detection setup complete".to_string()
        ],
        api_features: vec![
            "add_rigid_body".to_string(),
            "apply_impulse".to_string(),
            "raycast".to_string()
        ],
        difficulty_level: DifficultyLevel::Advanced,
        category: ExampleCategory::Physics,
    }
}

// ===== EVENT SYSTEM EXAMPLES =====

fn basic_events_example() -> ExampleScript {
    ExampleScript {
        name: "basic_events".to_string(),
        description: "Basic event system usage".to_string(),
        code: r#"
-- Listen for events (simplified API)
print("Setting up event listeners")

-- Emit a custom event
print("Emitting player_spawn event")

-- Emit game events
print("Emitting level_complete event")

return "Basic events demo complete"
        "#.to_string(),
        expected_outputs: vec![
            "Setting up event listeners".to_string(),
            "Basic events demo complete".to_string()
        ],
        api_features: vec!["print".to_string()],
        difficulty_level: DifficultyLevel::Intermediate,
        category: ExampleCategory::EventSystem,
    }
}

fn custom_events_example() -> ExampleScript {
    ExampleScript {
        name: "custom_events".to_string(),
        description: "Creating and handling custom events".to_string(),
        code: r#"
-- Define custom event types
local EVENT_PLAYER_DIED = "player_died"
local EVENT_ITEM_COLLECTED = "item_collected"
local EVENT_LEVEL_UP = "level_up"

-- Event data structures
local function create_player_death_event(cause)
    return {
        type = EVENT_PLAYER_DIED,
        cause = cause,
        timestamp = os.time()
    }
end

local function create_item_event(item_name, rarity)
    return {
        type = EVENT_ITEM_COLLECTED,
        item = item_name,
        rarity = rarity,
        timestamp = os.time()
    }
end

-- Simulate events
local death_event = create_player_death_event("dragon")
print("Player death event: " .. death_event.cause)

local item_event = create_item_event("Magic Sword", "legendary")
print("Item collected: " .. item_event.item .. " (" .. item_event.rarity .. ")")

return "Custom events demo complete"
        "#.to_string(),
        expected_outputs: vec![
            "Player death event: dragon".to_string(),
            "Item collected: Magic Sword (legendary)".to_string(),
            "Custom events demo complete".to_string()
        ],
        api_features: vec!["print".to_string()],
        difficulty_level: DifficultyLevel::Advanced,
        category: ExampleCategory::EventSystem,
    }
}

// ===== DEBUGGING EXAMPLES =====

fn debugging_basics_example() -> ExampleScript {
    ExampleScript {
        name: "debugging_basics".to_string(),
        description: "Basic debugging techniques".to_string(),
        code: r#"
-- Debug print statements
debug_print("Starting calculation sequence")

-- Debug inspect variables
local player_health = 85
local player_mana = 42
local player_level = 7

debug_inspect("player_health", player_health)
debug_inspect("player_mana", player_mana)
debug_inspect("player_level", player_level)

-- Complex data inspection
local player_data = {
    name = "Hero",
    stats = {health = player_health, mana = player_mana},
    level = player_level,
    inventory = {"sword", "potion", "key"}
}

debug_inspect("player_data", player_data)

-- Conditional debugging
if player_health < 50 then
    debug_print("WARNING: Player health is low!")
end

-- Debug breakpoint (for interactive debugging)
debug_break()

debug_print("Debugging sequence complete")

return "Debugging demo complete"
        "#.to_string(),
        expected_outputs: vec![
            "Starting calculation sequence".to_string(),
            "Debugging demo complete".to_string()
        ],
        api_features: vec![
            "debug_print".to_string(),
            "debug_inspect".to_string(),
            "debug_break".to_string()
        ],
        difficulty_level: DifficultyLevel::Intermediate,
        category: ExampleCategory::Debugging,
    }
}

fn advanced_debugging_example() -> ExampleScript {
    ExampleScript {
        name: "advanced_debugging".to_string(),
        description: "Advanced debugging with call stack and error handling".to_string(),
        code: r#"
-- Function with debugging
function calculate_damage(attacker, defender)
    debug_print("Entering calculate_damage function")
    debug_inspect("attacker", attacker)
    debug_inspect("defender", defender)
    
    if not attacker or not defender then
        debug_print("ERROR: Missing attacker or defender data")
        return 0
    end
    
    local base_damage = attacker.attack or 0
    local defense = defender.defense or 0
    local final_damage = math.max(1, base_damage - defense)
    
    debug_inspect("final_damage", final_damage)
    debug_print("Exiting calculate_damage function")
    
    return final_damage
end

-- Test with valid data
local hero = {attack = 50, health = 100}
local monster = {defense = 15, health = 80}

debug_print("=== Testing with valid data ===")
local damage = calculate_damage(hero, monster)
print("Damage dealt: " .. damage)

-- Test with invalid data
debug_print("=== Testing with invalid data ===")
local invalid_damage = calculate_damage(nil, monster)
print("Damage with nil attacker: " .. invalid_damage)

-- Performance debugging
debug_print("=== Performance test ===")
local start_time = os.clock()

for i = 1, 1000 do
    calculate_damage(hero, monster)
end

local end_time = os.clock()
local execution_time = end_time - start_time
debug_print("1000 calculations took: " .. execution_time .. " seconds")

return "Advanced debugging complete"
        "#.to_string(),
        expected_outputs: vec![
            "Testing with valid data".to_string(),
            "Advanced debugging complete".to_string()
        ],
        api_features: vec![
            "debug_print".to_string(),
            "debug_inspect".to_string()
        ],
        difficulty_level: DifficultyLevel::Advanced,
        category: ExampleCategory::Debugging,
    }
}

// ===== PERFORMANCE EXAMPLES =====

fn basic_profiling_example() -> ExampleScript {
    ExampleScript {
        name: "basic_profiling".to_string(),
        description: "Basic performance profiling".to_string(),
        code: r#"
-- Start profiling
profile_start()

-- Mark beginning of computation
profile_mark("computation_start")

-- Simulate some work
local sum = 0
for i = 1, 10000 do
    sum = sum + i * i
end

profile_mark("computation_middle")

-- More work
local product = 1
for i = 1, 100 do
    product = product * (i / 100)
end

profile_mark("computation_end")

-- Stop profiling
profile_stop()

print("Profiling complete. Sum: " .. sum .. ", Product: " .. product)

return "Basic profiling complete"
        "#.to_string(),
        expected_outputs: vec!["Basic profiling complete".to_string()],
        api_features: vec![
            "profile_start".to_string(),
            "profile_mark".to_string(),
            "profile_stop".to_string()
        ],
        difficulty_level: DifficultyLevel::Intermediate,
        category: ExampleCategory::Performance,
    }
}

fn performance_optimization_example() -> ExampleScript {
    ExampleScript {
        name: "performance_optimization".to_string(),
        description: "Performance optimization techniques".to_string(),
        code: r#"
-- Inefficient approach
profile_start()
profile_mark("inefficient_start")

local inefficient_result = 0
for i = 1, 1000 do
    for j = 1, 1000 do
        inefficient_result = inefficient_result + (i * j)
    end
end

profile_mark("inefficient_end")

-- Efficient approach
profile_mark("efficient_start")

local efficient_result = 0
local sum_i = 0
for i = 1, 1000 do
    sum_i = sum_i + i
end

local sum_j = 0
for j = 1, 1000 do
    sum_j = sum_j + j
end

efficient_result = sum_i * sum_j

profile_mark("efficient_end")

profile_stop()

print("Inefficient result: " .. inefficient_result)
print("Efficient result: " .. efficient_result)
print("Results match: " .. tostring(inefficient_result == efficient_result))

return "Performance optimization demo complete"
        "#.to_string(),
        expected_outputs: vec![
            "Results match: true".to_string(),
            "Performance optimization demo complete".to_string()
        ],
        api_features: vec![
            "profile_start".to_string(),
            "profile_mark".to_string(),
            "profile_stop".to_string()
        ],
        difficulty_level: DifficultyLevel::Advanced,
        category: ExampleCategory::Performance,
    }
}

// ===== GAME LOGIC EXAMPLES =====

fn simple_game_loop_example() -> ExampleScript {
    ExampleScript {
        name: "simple_game_loop".to_string(),
        description: "Basic game loop structure".to_string(),
        code: r#"
-- Game state
local game_state = {
    running = true,
    score = 0,
    level = 1,
    player = {
        x = 0,
        y = 0,
        health = 100
    }
}

-- Update function
function update_game(delta_time)
    -- Update player position based on input
    if is_key_pressed("W") then
        game_state.player.y = game_state.player.y + 1
    end
    if is_key_pressed("S") then
        game_state.player.y = game_state.player.y - 1
    end
    if is_key_pressed("A") then
        game_state.player.x = game_state.player.x - 1
    end
    if is_key_pressed("D") then
        game_state.player.x = game_state.player.x + 1
    end
    
    -- Update score
    game_state.score = game_state.score + 1
    
    -- Check for level progression
    if game_state.score > 100 * game_state.level then
        game_state.level = game_state.level + 1
        print("Level up! Now level " .. game_state.level)
    end
end

-- Render function
function render_game()
    print("Score: " .. game_state.score .. " | Level: " .. game_state.level)
    print("Player: (" .. game_state.player.x .. ", " .. game_state.player.y .. ")")
    print("Health: " .. game_state.player.health)
end

-- Simulate a few game ticks
for tick = 1, 5 do
    update_game(0.016) -- 60 FPS
    render_game()
    print("--- Tick " .. tick .. " ---")
end

return "Game loop simulation complete"
        "#.to_string(),
        expected_outputs: vec![
            "Score:".to_string(),
            "Game loop simulation complete".to_string()
        ],
        api_features: vec!["is_key_pressed".to_string(), "print".to_string()],
        difficulty_level: DifficultyLevel::Intermediate,
        category: ExampleCategory::GameLogic,
    }
}

fn player_controller_example() -> ExampleScript {
    ExampleScript {
        name: "player_controller".to_string(),
        description: "Advanced player controller with physics".to_string(),
        code: r#"
-- Player controller class
local PlayerController = {}
PlayerController.__index = PlayerController

function PlayerController.new(x, y, z)
    local self = setmetatable({}, PlayerController)
    
    -- Create physics body
    self.rigid_body = add_rigid_body(x, y, z, "Dynamic", 1.0)
    
    -- Player properties
    self.speed = 10
    self.jump_force = 15
    self.is_grounded = false
    self.health = 100
    self.max_health = 100
    
    return self
end

function PlayerController:update()
    -- Movement input
    local move_x = 0
    local move_z = 0
    
    if is_key_pressed("W") then move_z = move_z + 1 end
    if is_key_pressed("S") then move_z = move_z - 1 end
    if is_key_pressed("A") then move_x = move_x - 1 end
    if is_key_pressed("D") then move_x = move_x + 1 end
    
    -- Apply movement forces
    if move_x ~= 0 or move_z ~= 0 then
        apply_force(self.rigid_body, move_x * self.speed, 0, move_z * self.speed)
    end
    
    -- Jump input
    if is_key_pressed("Space") and self.is_grounded then
        apply_impulse(self.rigid_body, 0, self.jump_force, 0)
        self.is_grounded = false
        print("Player jumped!")
    end
    
    -- Health regeneration
    if self.health < self.max_health then
        self.health = math.min(self.max_health, self.health + 0.1)
    end
end

function PlayerController:take_damage(amount)
    self.health = math.max(0, self.health - amount)
    print("Player took " .. amount .. " damage. Health: " .. self.health)
    
    if self.health <= 0 then
        print("Player died!")
        return true -- Player is dead
    end
    
    return false
end

-- Create and test player controller
local player = PlayerController.new(0, 5, 0)

-- Simulate gameplay
for frame = 1, 10 do
    player:update()
    
    -- Simulate taking damage occasionally
    if frame % 5 == 0 then
        player:take_damage(10)
    end
    
    print("Frame " .. frame .. " - Health: " .. math.floor(player.health))
end

return "Player controller demo complete"
        "#.to_string(),
        expected_outputs: vec![
            "Player took 10 damage".to_string(),
            "Player controller demo complete".to_string()
        ],
        api_features: vec![
            "add_rigid_body".to_string(),
            "apply_force".to_string(),
            "apply_impulse".to_string(),
            "is_key_pressed".to_string()
        ],
        difficulty_level: DifficultyLevel::Advanced,
        category: ExampleCategory::GameLogic,
    }
}

fn game_state_management_example() -> ExampleScript {
    ExampleScript {
        name: "game_state_management".to_string(),
        description: "Managing different game states".to_string(),
        code: r#"
-- Game state enumeration
local GameState = {
    MENU = "menu",
    PLAYING = "playing",
    PAUSED = "paused",
    GAME_OVER = "game_over"
}

-- Game state manager
local StateManager = {}
StateManager.__index = StateManager

function StateManager.new()
    local self = setmetatable({}, StateManager)
    self.current_state = GameState.MENU
    self.previous_state = nil
    self.state_data = {}
    return self
end

function StateManager:change_state(new_state)
    print("Changing state from " .. self.current_state .. " to " .. new_state)
    self.previous_state = self.current_state
    self.current_state = new_state
    
    -- State entry logic
    if new_state == GameState.PLAYING then
        self:enter_playing_state()
    elseif new_state == GameState.PAUSED then
        self:enter_paused_state()
    elseif new_state == GameState.GAME_OVER then
        self:enter_game_over_state()
    end
end

function StateManager:enter_playing_state()
    print("Starting new game...")
    self.state_data = {
        score = 0,
        lives = 3,
        level = 1
    }
end

function StateManager:enter_paused_state()
    print("Game paused")
    -- Save current state data
end

function StateManager:enter_game_over_state()
    print("Game Over! Final score: " .. (self.state_data.score or 0))
end

function StateManager:update()
    if self.current_state == GameState.MENU then
        if is_key_pressed("Enter") then
            self:change_state(GameState.PLAYING)
        end
    elseif self.current_state == GameState.PLAYING then
        if is_key_pressed("Escape") then
            self:change_state(GameState.PAUSED)
        end
        
        -- Simulate gameplay
        self.state_data.score = self.state_data.score + 1
        
        -- Check game over condition
        if self.state_data.lives <= 0 then
            self:change_state(GameState.GAME_OVER)
        end
    elseif self.current_state == GameState.PAUSED then
        if is_key_pressed("Escape") then
            self:change_state(GameState.PLAYING)
        end
    elseif self.current_state == GameState.GAME_OVER then
        if is_key_pressed("R") then
            self:change_state(GameState.MENU)
        end
    end
end

-- Test state manager
local game = StateManager.new()

-- Simulate state transitions
print("=== Starting in MENU state ===")
game:update()

print("=== Simulating Enter press ===")
game:change_state(GameState.PLAYING)

print("=== Playing for a while ===")
for i = 1, 3 do
    game:update()
end

print("=== Simulating pause ===")
game:change_state(GameState.PAUSED)

print("=== Simulating game over ===")
game.state_data.lives = 0
game:change_state(GameState.PLAYING)
game:update()

return "Game state management demo complete"
        "#.to_string(),
        expected_outputs: vec![
            "Starting in MENU state".to_string(),
            "Starting new game...".to_string(),
            "Game state management demo complete".to_string()
        ],
        api_features: vec!["is_key_pressed".to_string(), "print".to_string()],
        difficulty_level: DifficultyLevel::Advanced,
        category: ExampleCategory::GameLogic,
    }
}

// ===== INTEGRATION EXAMPLES =====

fn complete_game_example() -> ExampleScript {
    ExampleScript {
        name: "complete_game_example".to_string(),
        description: "A complete mini-game using all systems".to_string(),
        code: r#"
-- Complete mini-game: "Dodge the Falling Blocks"

-- Game initialization
print("=== Initializing Dodge Game ===")

local game = {
    player = nil,
    blocks = {},
    score = 0,
    game_over = false,
    spawn_timer = 0
}

-- Create player
game.player = add_rigid_body(0, 1, 0, "Kinematic", 1.0)
print("Player created")

-- Game update function
function update_game(delta_time)
    if game.game_over then return end
    
    -- Player movement
    local move_speed = 15
    if is_key_pressed("A") then
        apply_force(game.player, -move_speed, 0, 0)
    end
    if is_key_pressed("D") then
        apply_force(game.player, move_speed, 0, 0)
    end
    
    -- Spawn falling blocks
    game.spawn_timer = game.spawn_timer + delta_time
    if game.spawn_timer > 1.0 then
        local x = (math.random() - 0.5) * 20
        local block = add_rigid_body(x, 15, 0, "Dynamic", 1.0)
        table.insert(game.blocks, block)
        game.spawn_timer = 0
        print("Spawned block at x=" .. x)
    end
    
    -- Update score
    game.score = game.score + 1
    
    -- Check collisions (simplified)
    if #game.blocks > 0 and game.score % 100 == 0 then
        print("Near miss! Score: " .. game.score)
    end
    
    -- Game over condition
    if game.score > 1000 then
        game.game_over = true
        print("You survived! Final score: " .. game.score)
    end
end

-- Input setup
bind_key("R", function()
    if game.game_over then
        -- Restart game
        game.score = 0
        game.game_over = false
        game.blocks = {}
        print("Game restarted!")
    end
end)

-- Debugging
debug_print("Game systems initialized")

-- Profiling
profile_start()

-- Simulate game loop
for tick = 1, 20 do
    update_game(0.05) -- 20 FPS simulation
    
    if tick % 5 == 0 then
        print("Tick " .. tick .. " - Score: " .. game.score)
    end
end

profile_stop()

return "Complete game demo finished"
        "#.to_string(),
        expected_outputs: vec![
            "Initializing Dodge Game".to_string(),
            "Player created".to_string(),
            "Complete game demo finished".to_string()
        ],
        api_features: vec![
            "add_rigid_body".to_string(),
            "apply_force".to_string(),
            "is_key_pressed".to_string(),
            "bind_key".to_string(),
            "debug_print".to_string(),
            "profile_start".to_string(),
            "profile_stop".to_string()
        ],
        difficulty_level: DifficultyLevel::Advanced,
        category: ExampleCategory::Integration,
    }
}

fn modding_example() -> ExampleScript {
    ExampleScript {
        name: "modding_example".to_string(),
        description: "Example of how to create game mods".to_string(),
        code: r#"
-- Mod: Enhanced Player Abilities

print("=== Loading Enhanced Player Mod ===")

-- Mod configuration
local mod_config = {
    name = "Enhanced Player",
    version = "1.0",
    author = "ModAuthor",
    description = "Adds super jump and speed boost abilities"
}

print("Loading mod: " .. mod_config.name .. " v" .. mod_config.version)

-- Enhanced player abilities
local enhanced_player = {
    super_jump_cooldown = 0,
    speed_boost_active = false,
    speed_boost_timer = 0
}

-- Super jump ability
function enhanced_player:super_jump()
    if self.super_jump_cooldown <= 0 then
        print("SUPER JUMP activated!")
        -- Apply strong upward impulse
        local player_body = add_rigid_body(0, 2, 0, "Dynamic", 1.0) -- Mock player
        apply_impulse(player_body, 0, 25, 0)
        self.super_jump_cooldown = 3.0 -- 3 second cooldown
        
        return true
    else
        print("Super jump on cooldown: " .. math.floor(self.super_jump_cooldown) .. "s")
        return false
    end
end

-- Speed boost ability
function enhanced_player:activate_speed_boost()
    if not self.speed_boost_active then
        print("SPEED BOOST activated!")
        self.speed_boost_active = true
        self.speed_boost_timer = 5.0 -- 5 second duration
        return true
    else
        print("Speed boost already active")
        return false
    end
end

function enhanced_player:update(delta_time)
    -- Update cooldowns
    if self.super_jump_cooldown > 0 then
        self.super_jump_cooldown = self.super_jump_cooldown - delta_time
    end
    
    if self.speed_boost_active then
        self.speed_boost_timer = self.speed_boost_timer - delta_time
        if self.speed_boost_timer <= 0 then
            self.speed_boost_active = false
            print("Speed boost expired")
        end
    end
end

-- Key bindings for mod abilities
bind_key("Q", function()
    enhanced_player:super_jump()
end)

bind_key("E", function()
    enhanced_player:activate_speed_boost()
end)

print("Mod keybindings:")
print("Q - Super Jump")
print("E - Speed Boost")

-- Simulate mod usage
print("=== Testing Mod Abilities ===")

-- Test super jump
enhanced_player:super_jump()
enhanced_player:super_jump() -- Should be on cooldown

-- Test speed boost
enhanced_player:activate_speed_boost()
enhanced_player:activate_speed_boost() -- Should already be active

-- Simulate time passing
for i = 1, 5 do
    enhanced_player:update(1.0) -- 1 second per iteration
    print("Time: " .. i .. "s")
end

print("Mod loading complete!")

return "Modding example complete"
        "#.to_string(),
        expected_outputs: vec![
            "Loading Enhanced Player Mod".to_string(),
            "SUPER JUMP activated!".to_string(),
            "Modding example complete".to_string()
        ],
        api_features: vec![
            "add_rigid_body".to_string(),
            "apply_impulse".to_string(),
            "bind_key".to_string(),
            "print".to_string()
        ],
        difficulty_level: DifficultyLevel::Advanced,
        category: ExampleCategory::Integration,
    }
}