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


insim.on_hello_world(function()
  print("Hello, world from lua!")
end)

insim.on_tiny(function()
  print("Got a Tiny!")
end)

insim.on_multi_car_info(function(mci)
  print("Got a MultiCarInfo!")
  print(dump(mci))
end)
