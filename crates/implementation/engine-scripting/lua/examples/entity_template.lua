-- Entity script template with component access
-- This script demonstrates how to interact with entity components

-- Component references (will be set up by the engine)
local transform = nil
local health = nil

function init()
    print("Entity script initialized for entity:", self.entity.id)
    
    -- Get component references
    transform = self.entity:get_component("Transform")
    health = self.entity:get_component("Health")
    
    if transform then
        print("Initial position:", transform.position[1], transform.position[2], transform.position[3])
    end
    
    if health then
        print("Initial health:", health.current, "/", health.max)
    end
end

function update(delta_time)
    -- Example: Move the entity slowly forward
    if transform then
        transform.position[3] = transform.position[3] + delta_time * 1.0
    end
    
    -- Example: Regenerate health slowly
    if health and health.current < health.max then
        health.current = math.min(health.max, health.current + delta_time * 10.0)
    end
end

function destroy()
    print("Entity script destroyed")
end

-- Custom function to take damage
function take_damage(amount)
    if health then
        health.current = math.max(0, health.current - amount)
        print("Took", amount, "damage. Health now:", health.current)
        
        if health.current <= 0 then
            print("Entity died!")
            -- Could trigger death logic here
        end
    end
end

-- Custom function to heal
function heal(amount)
    if health then
        health.current = math.min(health.max, health.current + amount)
        print("Healed", amount, "points. Health now:", health.current)
    end
end