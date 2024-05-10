print("called setup")

local transform = get_component("TransformComponent")

MainEvents.FixedUpdate.connect(function() 
    local direction = Input.get_vector("KeyA", "KeyS", "KeyD", "KeyW")

    -- transform.translation += direction * 10

    transform.translation = Input.mouse_position();
end)
