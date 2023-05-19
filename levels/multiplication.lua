function generateTestCase()
  local inputs, outputs = {}, {}

  for i = 1, 14, 2 do
    inputs[i] = math.random(-99, 99)
    inputs[i + 1] = math.random(-10, 10)
    outputs[math.floor(i / 2) + 1] = inputs[i] * inputs[i + 1]
  end

  return inputs, outputs
end
