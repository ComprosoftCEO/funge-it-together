function generateTestCase()
  -- Make sure we pick constants that do not loop
  local multiplier, constant, seed, outputs
  repeat
    multiplier = math.random(2, 999)
    constant = math.random(0, 999)
    seed = math.random(1, 999)
    outputs = generateOutputs(multiplier, constant, seed)
  until not hasRepeatedNumbers(outputs)

  local inputs = { multiplier, constant, seed }
  return inputs, outputs
end

function generateOutputs(multiplier, constant, seed)
  local outputs = {}
  for i = 1, 15 do
    seed = (seed * multiplier + constant) % 1000
    outputs[i] = seed
  end

  return outputs
end

-- Returns "true" if the array has duplicate numbers
function hasRepeatedNumbers(inputs)
  local uniqueNumbers = {}
  for i = 1, #inputs do
    if uniqueNumbers[inputs[i]] then
      return true
    end

    uniqueNumbers[inputs[i]] = true
  end

  return false
end
