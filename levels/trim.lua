function generateTestCase()
  local inputs, outputs = {}, {}

  local leading_zeros = math.random(0, 5)
  local trailing_zeros = math.random(0, 5)

  for i = 1, 15 do
    if (i <= leading_zeros) or (i > (15 - trailing_zeros)) then
      inputs[i] = 0
    else
      inputs[i] = math.random(-9, 9)
    end
    outputs[i] = inputs[i]
  end


  -- Trim leading zeros
  while outputs[1] == 0 do
    table.remove(outputs, 1)
  end

  -- Trim trailing zeros
  while outputs[#outputs] == 0 do
    table.remove(outputs, #outputs)
  end

  return inputs, outputs
end
