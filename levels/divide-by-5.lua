function generateTestCase()
  local inputs, outputs = {}, {}

  for i = 1, 15 do
    inputs[i] = math.random(5, 100)
    outputs[i] = math.floor(inputs[i] / 5)
  end

  return inputs, outputs
end
