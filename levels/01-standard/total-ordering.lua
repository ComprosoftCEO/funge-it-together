function generateTestCase()
  local inputs, outputs = {}, {}

  for i = 1, 14, 2 do
    inputs[i] = math.random(-999, 999)
    inputs[i + 1] = math.random(-999, 999)
    outputs[i] = math.min(inputs[i], inputs[i + 1])
    outputs[i + 1] = math.max(inputs[i], inputs[i + 1])
  end

  return inputs, outputs
end
