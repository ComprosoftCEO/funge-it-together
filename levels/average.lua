function generateTestCase()
  local inputs, outputs = {}, {}

  local num_inputs = math.random(2, 10)
  local sum = 0
  for i = 1, num_inputs do
    inputs[i] = math.random(1, 99)
    sum = sum + inputs[i]
  end

  outputs[1] = math.floor(sum / num_inputs)

  return inputs, outputs
end
