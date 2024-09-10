function generateTestCase()
  local inputs, outputs = {}, {}

  for i = 1, 15 do
    inputs[i] = math.random(0, 12)
    outputs[i] = nthTribonacci(inputs[i])
  end

  return inputs, outputs
end

function nthTribonacci(n)
  if n < 2 then
    return 0
  elseif n == 2 then
    return 1
  else
    return nthTribonacci(n - 1) + nthTribonacci(n - 2) + nthTribonacci(n - 3)
  end
end
