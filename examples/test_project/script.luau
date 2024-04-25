print("called setup")

local transform = entity.get_component("TransformComponent")

MainEvents.Update.connect(function() 
    -- if Input.key_pressed("W") then

    -- end

    transform.translation += Vector3.new(1., 1., 0.)
    transform.rotation += Vector3.new(0., 0., 0.5)
end)
