function generateTestCase()
  local multiplier = math.random(2, 999)
  local constant = math.random(0, 999)
  local seed = math.random(1, 999)

  local inputs = { multiplier, constant, seed }

  local outputs = {}
  for i = 1, 15 do
    seed = (seed * constant + multiplier) % 1000
    outputs[i] = seed
  end

  return inputs, outputs
end
