function generateTestCase()
  local inputs, outputs = {}, {}

  for i = 1, 15 do
    inputs[i] = math.random(0, 6)
    outputs[i] = factorial(inputs[i])
  end

  return inputs, outputs
end

function factorial(x)
  if x < 1 then
    return 1
  else
    return x * factorial(x - 1)
  end
end
