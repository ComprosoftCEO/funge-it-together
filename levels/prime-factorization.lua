function generateTestCase()
  local input = math.random(2, 999)

  local factors = {}
  local currentFactor = 2
  local remainder = input
  while currentFactor < remainder do
    if (remainder % currentFactor) == 0 then
      factors[#factors + 1] = currentFactor
      remainder = remainder / currentFactor
      currentFactor = 2

      goto continue
    end

    currentFactor = currentFactor + 1

    ::continue::
  end

  if currentFactor > 1 then
    factors[#factors + 1] = currentFactor
  end

  return { input }, factors
end
