print("called setup")

local transform = entity.get_component("TransformComponent")

MainEvents.Update.connect(function() 
    transform.translation += Vector3.new(1., 1., 0.)
end)
