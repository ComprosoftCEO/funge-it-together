function generateTestCase()
  local inputs, outputs = {}, {}

  for i = 1, 15 do
    inputs[i] = math.random(0, 15)
    outputs[i] = nthFibonacci(inputs[i])
  end

  return inputs, outputs
end

function nthFibonacci(n)
  if n < 2 then
    return n
  else
    return nthFibonacci(n - 1) + nthFibonacci(n - 2)
  end
end
