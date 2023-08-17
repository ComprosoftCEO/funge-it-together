function generateTestCase()
  local inputs, outputs = {}, {}

  for i = 1, 15 do
    inputs[i] = math.random(0, 999)
    outputs[i] = digitalRoot(inputs[i])
  end

  return inputs, outputs
end

function digitalRoot(x)
  if x < 10 then
    return x
  end

  local hundreds = math.floor(x / 100)
  x = x % 100
  local tens = math.floor(x / 10)
  x = x % 10

  return digitalRoot(x + tens + hundreds)
end
