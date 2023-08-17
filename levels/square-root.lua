function generateTestCase()
  local inputs, outputs = {}, {}

  for i = 1, 15 do
    inputs[i] = math.random(0, 999)
    outputs[i] = math.floor(math.sqrt(inputs[i]))
  end

  return inputs, outputs
end
