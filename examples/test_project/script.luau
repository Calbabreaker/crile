print("called setup")

local transform = entity.get_component("TransformComponent")

MainEvents.FixedUpdate.connect(function() 
    local direction = Input.get_vector("KeyA", "KeyS", "KeyD", "KeyW")

    transform.translation += Vector3.new(direction.x*  10, direction.y * 10, 0)
end)
