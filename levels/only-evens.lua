function generateTestCase()
  local inputs, outputs = {}, {}

  for i = 1, 15 do
    inputs[i] = math.random(-10, 10)
    if math.abs(inputs[i]) % 2 == 0 then
      outputs[#outputs + 1] = inputs[i]
    end
  end

  return inputs, outputs
end
