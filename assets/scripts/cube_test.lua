-- Test script for the cube entity
local CubeController = {}

function CubeController:init()
    print("[CubeController] Script initialized!")
    print("[CubeController] Attached to entity:", self.entity)
    
    -- Initialize rotation
    self.total_time = 0
    self.rotation_speed = 45 -- degrees per second
end

function CubeController:update(delta_time)
    -- Update total time
    self.total_time = self.total_time + delta_time
    
    -- Log every second
    if math.floor(self.total_time) > math.floor(self.total_time - delta_time) then
        print(string.format("[CubeController] Time: %.1f seconds", self.total_time))
        
        -- Try to get transform component
        local transform = self.entity:get_component("Transform")
        if transform then
            print(string.format("[CubeController] Position: [%.2f, %.2f, %.2f]", 
                transform.position[1], transform.position[2], transform.position[3]))
            print(string.format("[CubeController] Rotation: [%.2f, %.2f, %.2f]", 
                transform.rotation[1], transform.rotation[2], transform.rotation[3]))
        end
    end
    
    -- Rotate the cube
    local transform = self.entity:get_component("Transform")
    if transform then
        -- Rotate around Y axis
        transform.rotation[2] = transform.rotation[2] + (self.rotation_speed * delta_time)
        
        -- Keep rotation in 0-360 range
        if transform.rotation[2] > 360 then
            transform.rotation[2] = transform.rotation[2] - 360
        end
        
        -- Update the transform
        self.entity:set_component("Transform", transform)
    end
end

function CubeController:destroy()
    print("[CubeController] Script destroyed!")
end

return CubeController