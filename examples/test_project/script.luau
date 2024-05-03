print("called setup")

local transform = entity.get_component("TransformComponent")

MainEvents.Update.connect(function() 
    local direction = Input.get_vector("KeyA", "KeyS", "KeyD", "KeyW")

    transform.translation += Vector3.new(direction.x, direction.y, 0)
    transform.rotation += Vector3.new(0, 0, 0.1)
end)
