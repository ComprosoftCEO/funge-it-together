function generateTestCase()
  -- Limit the numbers available to pick from
  local numbersAvailable = {}
  for i = 1, math.random(4, 10) do
    numbersAvailable[i] = math.random(-99, 99)
  end

  -- Pick inputs from our pool of available numbers
  local inputs = {}
  local uniqueMap = {}
  for i = 1, math.random(6, 10) do
    inputs[i] = numbersAvailable[math.random(1, #numbersAvailable)]
    uniqueMap[inputs[i]] = true
  end

  local outputs = {}
  for k in pairs(uniqueMap) do
    outputs[#outputs + 1] = k
  end
  table.sort(outputs)

  return inputs, outputs
end
