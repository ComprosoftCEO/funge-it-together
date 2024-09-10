function generateTestCase()
  local inputs, outputs = {}, {}
  local max = math.random(-80, 99)

  for i = 1, 15 do
    inputs[i] = math.random(-99, max - 1)
  end
  inputs[math.random(1, #inputs)] = max
  outputs[1] = max

  return inputs, outputs
end
