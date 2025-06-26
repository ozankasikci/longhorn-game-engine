-- Simple test script that just prints
print("=== SIMPLE TEST SCRIPT LOADED ===")
print("This is a test from Lua!")

local SimpleTest = {}

function SimpleTest:init()
    print("[SimpleTest] init() called!")
end

function SimpleTest:update(delta_time)
    -- Print only once per second to avoid spam
    if not self.last_print then
        self.last_print = 0
    end
    
    self.last_print = self.last_print + delta_time
    if self.last_print >= 1.0 then
        print("[SimpleTest] One second passed!")
        self.last_print = 0
    end
end

function SimpleTest:destroy()
    print("[SimpleTest] destroy() called!")
end

return SimpleTest