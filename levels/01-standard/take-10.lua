function generateTestCase()
  local inputs, outputs = {}, {}

  for i = 1, math.random(5, 15) do
    inputs[i] = math.random(-999, 999)

    if i <= 10 then
      outputs[i] = inputs[i]
    end
  end

  return inputs, outputs
end
