function generateTestCase()
  local inputs, outputs = {}, {}

  for i = 1, 15 do
    inputs[i] = math.random(-999, 999)
    outputs[i] = math.abs(inputs[i])
  end

  return inputs, outputs
end
