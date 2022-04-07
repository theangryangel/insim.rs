insim = {}
insim._events = {}

insim._on = function(event, callback)
  if not insim._events[event] then
    insim._events[event] = {}
  end
  table.insert(insim._events[event], callback)
end

insim._emit = function(event, ...)
  if not insim._events[event] then
    return
  end
  for _, callback in ipairs(insim._events[event]) do
    callback(...)
  end
end

-- Dynamically create event handlers for convenience of end users
for _, event_name in ipairs({
  "hello_world",
  "connect",
  "disconnect",
  "new_player",
  "tiny",
  "multi_car_info",
}) do

  insim["on_" .. event_name] = function(callback)
    insim._on(event_name, callback)
  end

end
