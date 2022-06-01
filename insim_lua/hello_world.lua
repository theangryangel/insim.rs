function dump(o)
   if type(o) == 'table' then
      local s = '{ '
      for k,v in pairs(o) do
         if type(k) ~= 'number' then k = '"'..k..'"' end
         s = s .. '['..k..'] = ' .. dump(v) .. ','
      end
      return s .. '} '
   else
      return tostring(o)
   end
end

i = 0

k = insim:on('startup', function()
   tracing.debug('Events Hello World!' .. insim.instance)
end)

insim:on("connected", function()
  tracing.info("CONNECTED to " .. insim.instance)
end)

insim:on("multi_car_info", function(data)
  tracing.info("MULTI_CAR_INFO: " .. dump(data))
  i += 1
  if i >= 10 then
    tracing.info("Dying. We got to 10!")
    insim:shutdown()
  end
end)
