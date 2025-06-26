-- Basic Lua script template
-- This script will be attached to an entity and receive lifecycle events

-- Called once when the script is first loaded
function init()
    print("Script initialized!")
end

-- Called every frame while the entity is active
-- delta_time: Time since last frame in seconds
function update(delta_time)
    -- Add your update logic here
    -- print("Update called with delta_time:", delta_time)
end

-- Called when the script is about to be destroyed
function destroy()
    print("Script destroyed!")
end

-- You can define your own custom functions here
function my_custom_function()
    print("Custom function called!")
end