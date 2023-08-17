function generateTestCase()
  local inputs = { randomNonZero() }

  local flips = 0
  local isPositive = (inputs[1] > 0)
  for i = 2, 15 do
    inputs[i] = randomNonZero()

    if isPositive and inputs[i] < 0 then
      flips = flips + 1
      isPositive = false
    elseif not isPositive and inputs[i] > 0 then
      flips = flips + 1
      isPositive = true
    end
  end

  return inputs, { flips }
end

function randomNonZero()
  local x
  repeat
    x = math.random(-999, 999)
  until x ~= 0
  return x
end
