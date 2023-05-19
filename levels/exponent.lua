function generateTestCase()
  local inputs, outputs = {}, {}

  for i = 1, 14, 2 do
    inputs[i] = math.random(0, 9)

    local minValue = 0
    if inputs[i] <= 1 then
      inputs[i + 1] = math.random(1, 9)
    else
      inputs[i + 1] = math.random(0, math.floor(math.log(999, inputs[i])))
    end
    outputs[math.floor(i / 2) + 1] = inputs[i] ^ inputs[i + 1]
  end

  return inputs, outputs
end
