local transform = get_component("TransformComponent")

MainEvents.FixedUpdate.connect(function() 
    local direction = Input.get_vector("KeyA", "KeyS", "KeyD", "KeyW")

    transform.translation += direction * 10

    if Input.mouse_just_pressed("Left") then
        print("test")
    end
end)
