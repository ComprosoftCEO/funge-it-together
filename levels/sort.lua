function generateTestCase()
  local inputs, outputs = {}, {}

  for i = 1, math.random(6, 10) do
    inputs[i] = math.random(-99, 99)
    outputs[i] = inputs[i]
  end
  table.sort(outputs)

  return inputs, outputs
end
