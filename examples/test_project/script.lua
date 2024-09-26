print(self)
local transform = self.get("TransformComponent")

local entity = spawn()

MainEvents.Update.connect(function() 
    local direction = Input.get_vector("KeyA", "KeyS", "KeyD", "KeyW")

    transform.translation += direction * 1000 * Time.delta()

    if Input.mouse_just_pressed("Left") then
        print("test")
    end
end)
