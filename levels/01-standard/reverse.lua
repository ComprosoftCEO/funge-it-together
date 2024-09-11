function generateTestCase()
  local inputs, outputs = {}, {}

  for i = 1, 15 do
    inputs[i] = math.random(-999, 999)
  end
  for i = 1, 15 do
    outputs[i] = inputs[16 - i]
  end

  return inputs, outputs
end
