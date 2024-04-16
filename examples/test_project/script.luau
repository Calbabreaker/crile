print("called setup")
local position = Vector3.new(0., 0., 0.)

local transform = entity.get_component("TransformComponent")

MainEvents.Update.connect(function() 
    transform.translation.x += 1
end)
