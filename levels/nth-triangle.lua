function generateTestCase()
  local inputs, outputs = {}, {}

  for i = 1, 15 do
    inputs[i] = math.random(0, 44)
    outputs[i] = nthTriangle(inputs[i])
  end

  return inputs, outputs
end

function nthTriangle(n)
  local sum = 0
  for i = 1, n do
    sum = sum + i
  end
  return sum
end
